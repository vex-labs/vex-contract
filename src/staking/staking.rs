use near_sdk::{env, near, require, Gas, NearToken};

pub use crate::ext::*;
use crate::*;

#[near]
impl Contract {
    // User stakes a certain amount of VEX from their unstaked balance
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

        // Subtract the staked amount from the unstaked balance and add the stake shares
        relevant_account.unstaked_balance =
            U128(relevant_account.unstaked_balance.0 - charge_amount);
        self.total_unstaked_balance = U128(self.total_unstaked_balance.0 - charge_amount);
        relevant_account.stake_shares = U128(relevant_account.stake_shares.0 + num_shares);

        // Sets the time when they can next unstake to 1 week from now
        relevant_account.unstake_timestamp = U64(env::block_timestamp() + ONE_WEEK);

        // The staked amount that will be added to the total to guarantee the "stake" share price
        // doesnt decrease when staking because of rounding. The difference between `stake_amount` and `charge_amount` is paid
        // from the allocated STAKE_SHARE_PRICE_GUARANTEE_FUND.
        let stake_amount = self.staked_amount_from_num_shares_rounded_up(num_shares);

        self.total_staked_balance = U128(self.total_staked_balance.0 + stake_amount);
        self.total_stake_shares = U128(self.total_stake_shares.0 + num_shares);
    }

    // Stake all VEX from the unstaked balance
    pub fn stake_all(&mut self) {
        let account_id = env::predecessor_account_id();

        let relevant_account = self
            .users_stake
            .get(&account_id)
            .unwrap_or_else(|| panic!("You have to deposit before staking"));

        // Call stake with the full amount from unstaked balance
        let amount = relevant_account.unstaked_balance;
        self.stake(amount);
    }

    // Unstake a given amount of VEX to their unstaked balance
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

        // Cannot unstake before the unstake timestamp
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

        // Subtract the number of shares from the stake shares and add the receive amount to the unstaked balance
        relevant_account.stake_shares = U128(relevant_account.stake_shares.0 - num_shares);
        relevant_account.unstaked_balance =
            U128(relevant_account.unstaked_balance.0 + receive_amount);
        self.total_unstaked_balance = U128(self.total_unstaked_balance.0 + receive_amount);

        // The staked amount that will be added to the total to guarantee the "stake" share price
        // doesnt decrease when unstaking because of rounding. The difference between `stake_amount` and `charge_amount` is paid
        // from the allocated STAKE_SHARE_PRICE_GUARANTEE_FUND.
        let stake_amount = self.staked_amount_from_num_shares_rounded_down(num_shares);

        self.total_staked_balance = U128(self.total_staked_balance.0 - stake_amount);
        self.total_stake_shares = U128(self.total_stake_shares.0 - num_shares);
    }

    // Unstake all VEX to their unstaked balance
    pub fn unstake_all(&mut self) {
        let account_id = env::predecessor_account_id();

        let relevant_account = self
            .users_stake
            .get(&account_id)
            .unwrap_or_else(|| panic!("You do not have any stake"));

        // Get the total amount of VEX the account will receive by unstaking all the "stake" shares
        let amount =
            self.staked_amount_from_num_shares_rounded_down(relevant_account.stake_shares.0);

        // Call unstake with the full amount from staked balance
        self.unstake(U128(amount));
    }

    // Withdraw a given amount of VEX to their account
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

        // The user must keep at least 50 VEX in their unstaked balance or staked
        let staked_balance =
            self.staked_amount_from_num_shares_rounded_up(relevant_account.stake_shares.0);
        require!(
            relevant_account.unstaked_balance.0 + staked_balance - amount.0 >= 50
                || amount.0 == relevant_account.unstaked_balance.0,
            "You must keep at least 50 VEX staked or deposited or be withdrawing all"
        );

        // Subtract the amount from the unstaked balance
        relevant_account.unstaked_balance = U128(relevant_account.unstaked_balance.0 - amount.0);
        self.total_unstaked_balance = U128(self.total_unstaked_balance.0 - amount.0);

        // If the user still has stake shares or staked balance, add them back to the users_stake map
        if relevant_account.unstaked_balance.0 > 0 || relevant_account.stake_shares.0 > 0 {
            self.users_stake
                .insert(account_id.clone(), relevant_account);
        }

        // Transfer the amount of VEX to the user
        ft_contract::ext(self.usdc_token_contract.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(Gas::from_tgas(30))
            .ft_transfer(account_id, amount);
    }

    // Withdraw all VEX to their account
    pub fn withdraw_all(&mut self) {
        let account_id = env::predecessor_account_id();

        let relevant_account = self
            .users_stake
            .get(&account_id)
            .unwrap_or_else(|| panic!("You do not have any to withdraw"));

        let amount = relevant_account.unstaked_balance;

        // Call withdraw with the full amount
        self.withdraw(amount);
    }

    pub fn perform_stake_swap(&mut self) {
        // If the staking queue is empty then skip and update the last stake swap timestamp
        if self.staking_rewards_queue.is_empty() {
            self.last_stake_swap_timestamp = U64(env::block_timestamp());
            return;
        }

        // Get time passed since last stake swap
        let time_passed = env::block_timestamp() - self.last_stake_swap_timestamp.0;

        // Check if the first item in staking rewards queue has expired
        // repeat until the first item in the queue has not expired
        // if expired remove and calculate rewards
        // save matches to remove and rewards to remove for later use in callback
        let mut finished_matches_rewards: u128 = 0;
        let mut new_usdc_staking_rewards: u128 = self.usdc_staking_rewards.0;
        let mut num_to_pop: u16 = 0;
        while let Some(first) = self.staking_rewards_queue.front() {
            if first.stake_end_time.0 < env::block_timestamp() {
                let finished_match_time_passed =
                    first.stake_end_time.0 - self.last_stake_swap_timestamp.0;

                // Get rewards for this match has passed
                let passed_match_reward = (U256::from(finished_match_time_passed)
                    * U256::from(first.staking_rewards.0)
                    / U256::from(ONE_MONTH))
                .as_u128();

                finished_matches_rewards += passed_match_reward;

                new_usdc_staking_rewards -= first.staking_rewards.0;
                num_to_pop += 1;
            } else {
                break;
            }
        }

        // Calculate the rewards for the matches that have not expired
        let active_match_rewards = (U256::from(time_passed) * U256::from(new_usdc_staking_rewards)
            / U256::from(ONE_MONTH))
        .as_u128();

        let total_rewards_to_swap = finished_matches_rewards + active_match_rewards;

        // Swap these via ref finance then add a callback to add the output to total vex
        let msg = create_ref_message(
            self.ref_pool_id,
            self.usdc_token_contract.clone(),
            self.vex_token_contract.clone(),
            total_rewards_to_swap,
            0,
        );

        // Make call to ref finance to swap the USDC amount for VEX
        // callback to peform_stake_swap_callback
        ft_contract::ext(self.usdc_token_contract.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(Gas::from_tgas(100))
            .ft_transfer_call(self.ref_contract.clone(), U128(total_rewards_to_swap), msg)
            .then(
                Self::ext(env::current_account_id())
                    .perform_stake_swap_callback(new_usdc_staking_rewards, num_to_pop),
            );
    }


    // WIP improve callbacks

    // Callback after swapping USDC for VEX
    #[private]
    pub fn perform_stake_swap_callback(&mut self, new_usdc_staking_rewards: u128, num_to_pop: u16) {
        // Callback returns the amount of inputted token not outputted so we don't use this result
        // Make a call to get the amount of VEX the contract has
        // callback to balance_callback
        ft_contract::ext("token.betvex.testnet".parse().unwrap())
            .with_static_gas(Gas::from_tgas(30))
            .ft_balance_of(env::current_account_id())
            .then(
                Self::ext(env::current_account_id())
                    .balance_callback(new_usdc_staking_rewards, num_to_pop),
            );
    }

    // Callback after getting the VEX balance
    #[private]
    pub fn balance_callback(
        &mut self,
        #[callback_unwrap] balance: U128,
        new_usdc_staking_rewards: u128,
        num_to_pop: u16,
    ) {
        // I do not like this solution
        // Plus this we do not STAKE_SHARE_PRICE_GUARANTEE_FUND is not a constant really
        self.total_staked_balance =
            U128(balance.0 - self.total_unstaked_balance.0 - STAKE_SHARE_PRICE_GUARANTEE_FUND);

        // Set the new staking rewards since some matches have expired
        self.usdc_staking_rewards = U128(new_usdc_staking_rewards);

        // Remove the finished matches from the queue
        for _ in 0..num_to_pop {
            self.staking_rewards_queue.pop_front();
        }
    }
}
