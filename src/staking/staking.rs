use near_sdk::{env, near, require, Gas, NearToken};

pub use crate::ext::*;
use crate::*;

#[near]
impl Contract {
    pub fn stake(&mut self, amount: U128) {
        require!(amount.0 > 0, "Staking amount should be positive");

        let account_id = env::predecessor_account_id();

        // Get the rounded down number of stake shares
        let num_shares = self.num_shares_from_staked_amount_rounded_down(amount.0);

        // Get the amount of VEX for the rounded down stake shares
        let charge_amount = self.staked_amount_from_num_shares_rounded_down(num_shares);
        require!(
            charge_amount > 0,
            "Invariant violation. Calculated staked amount must be positive, because \"stake\" share price should be at least 1"
        );

        let relevant_account = self
            .users_stake
            .get_mut(&account_id)
            .unwrap_or_else(|| panic!("You have to deposit before staking"));

        require!(
            relevant_account.unstaked_balance.0 >= charge_amount,
            "Not enough unstaked balance to stake"
        );

        relevant_account.unstaked_balance =
            U128(relevant_account.unstaked_balance.0 - charge_amount);
        relevant_account.stake_shares = U128(relevant_account.stake_shares.0 + num_shares);
        relevant_account.unstake_timestamp = U64(env::block_timestamp() + ONE_WEEK); // One week in advance

        // The staked amount that will be added to the total to guarantee the "stake" share price
        // doesnt decrease when staking because of rounding. The difference between `stake_amount` and `charge_amount` is paid
        // from the allocated STAKE_SHARE_PRICE_GUARANTEE_FUND.
        let stake_amount = self.staked_amount_from_num_shares_rounded_up(num_shares);

        self.total_staked_balance = U128(self.total_staked_balance.0 + stake_amount);
        self.total_stake_shares = U128(self.total_stake_shares.0 + num_shares);
    }

    pub fn stake_all(&mut self) {
        let account_id = env::predecessor_account_id();

        let relevant_account = self
            .users_stake
            .get(&account_id)
            .unwrap_or_else(|| panic!("You have to deposit before staking"));

        let amount = relevant_account.unstaked_balance;
        self.stake(amount);
    }

    pub fn unstake(&mut self, amount: U128) {
        require!(amount.0 > 0, "Unstaking amount should be positive");

        let account_id = env::predecessor_account_id();
        let relevant_account = self
            .users_stake
            .get(&account_id)
            .unwrap_or_else(|| panic!("You do not have any stake"));

        require!(
            relevant_account.stake_shares.0 > 0,
            "You do not have any stake"
        );

        require!(
            env::block_timestamp() >= relevant_account.unstake_timestamp.0,
            "You cannot unstake yet"
        );

        // Calculate the number of shares required to unstake the given amount.
        // NOTE: The number of shares the account will pay is rounded up.
        let num_shares = self.num_shares_from_staked_amount_rounded_up(amount.0);
        require!(
            num_shares > 0,
            "Invariant violation. The calculated number of \"stake\" shares for unstaking should be positive"
        );

        require!(
            relevant_account.stake_shares.0 >= num_shares,
            "Not enough staked balance to unstake"
        );

        // Calculating the amount of tokens the account will receive by unstaking the corresponding
        // number of "stake" shares, rounding up.
        let receive_amount = self.staked_amount_from_num_shares_rounded_up(num_shares);
        require!(
            receive_amount > 0,
            "Invariant violation. Calculated staked amount must be positive, because \"stake\" share price should be at least 1"
        );

        let relevant_account = self
            .users_stake
            .get_mut(&account_id)
            .unwrap_or_else(|| panic!("You do not have any stake"));

        relevant_account.stake_shares = U128(relevant_account.stake_shares.0 - num_shares);
        relevant_account.unstaked_balance =
            U128(relevant_account.unstaked_balance.0 + receive_amount);

        // The staked amount that will be added to the total to guarantee the "stake" share price
        // doesnt decrease when unstaking because of rounding. The difference between `stake_amount` and `charge_amount` is paid
        // from the allocated STAKE_SHARE_PRICE_GUARANTEE_FUND.
        let stake_amount = self.staked_amount_from_num_shares_rounded_down(num_shares);

        self.total_staked_balance = U128(self.total_staked_balance.0 - stake_amount);
        self.total_stake_shares = U128(self.total_stake_shares.0 - num_shares);
    }

    pub fn unstake_all(&mut self) {
        let account_id = env::predecessor_account_id();

        let relevant_account = self
            .users_stake
            .get(&account_id)
            .unwrap_or_else(|| panic!("You do not have any stake"));

        let amount =
            self.staked_amount_from_num_shares_rounded_down(relevant_account.stake_shares.0);
        self.unstake(U128(amount));
    }

    pub fn withdraw(&mut self, amount: U128) {
        require!(amount.0 > 0, "Withdrawal amount should be positive");

        let account_id = env::predecessor_account_id();

        let mut relevant_account = self
            .users_stake
            .remove(&account_id)
            .unwrap_or_else(|| panic!("You do not have any to withdraw"));

        require!(
            relevant_account.unstaked_balance >= amount,
            "Not enough unstaked balance to withdraw"
        );

        relevant_account.unstaked_balance = U128(relevant_account.unstaked_balance.0 - amount.0);

        if relevant_account.unstaked_balance.0 > 0 || relevant_account.stake_shares.0 > 0 {
            self.users_stake
                .insert(account_id.clone(), relevant_account);
        }

        ft_contract::ext(self.usdc_contract.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(Gas::from_tgas(30))
            .ft_transfer(account_id, amount);
    }

    pub fn withdraw_all(&mut self) {
        let account_id = env::predecessor_account_id();

        let relevant_account = self
            .users_stake
            .get(&account_id)
            .unwrap_or_else(|| panic!("You do not have any to withdraw"));

        let amount = relevant_account.unstaked_balance;
        self.withdraw(amount);
    }

    pub fn perform_stake_swap(&mut self) {
        self.last_stake_swap_timestamp = U64(env::block_timestamp());
        if self.staking_rewards_queue.is_empty() {
            return;
        }

        let time_passed = env::block_timestamp() - self.last_stake_swap_timestamp.0;

        // Check if the first item in staking rewards queue has expired
        // repeat until the first item in the queue has not expired
        // if expired remove and calculate rewards
        let mut finished_matches_rewards: u128 = 0;
        while let Some(first) = self.staking_rewards_queue.front() {
            if first.stake_end_time.0 < env::block_timestamp() {
                let finished_match_time_passed =
                    first.stake_end_time.0 - self.last_stake_swap_timestamp.0;

                let passed_match_reward = (U256::from(finished_match_time_passed)
                    * U256::from(first.staking_rewards.0)
                    / U256::from(ONE_MONTH))
                .as_u128();
            
                finished_matches_rewards += passed_match_reward;
                self.usdc_staking_rewards =
                    U128(self.usdc_staking_rewards.0 - first.staking_rewards.0);

                self.staking_rewards_queue.pop_front();
            } else {
                break;
            }
        }

        // Calculate the rewards for the matches that have not expired
        let active_match_rewards = (U256::from(time_passed)
            * U256::from(self.usdc_staking_rewards.0)
            / U256::from(ONE_MONTH))
        .as_u128();

        let total_rewards_to_swap = finished_matches_rewards + active_match_rewards;

        // Swap these via ref finance then add a callback to add the output to total vex
        // WIP
    }
}
