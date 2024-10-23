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
        relevant_account.stake_shares = U128(relevant_account.stake_shares.0 + num_shares);

        // Sets the time when they can next unstake to 1 week from now
        relevant_account.unstake_timestamp = U64(env::block_timestamp() + self.unstake_time_buffer);

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

        // If the user still has stake shares or staked balance, add them back to the users_stake map
        if relevant_account.unstaked_balance.0 > 0 || relevant_account.stake_shares.0 > 0 {
            self.users_stake
                .insert(account_id.clone(), relevant_account);
        }

        // Transfer the amount of VEX to the user
        ft_contract::ext(self.vex_token_contract.clone())
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
}
