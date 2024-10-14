use near_sdk::{env, near, require};

use crate::*;

#[near]
impl Contract {
    // Depositing VEX tokens for staking
    pub(crate) fn deposit(&mut self, sender_id: AccountId, amount: U128) {
        require!(
            env::predecessor_account_id() == self.vex_token_contract,
            "Only VEX can be staked"
        );

        let (stake_shares, unstaked_balance) =
            self.users_stake.get(&sender_id).map_or((0, 0), |account| {
                (account.stake_shares.0, account.unstaked_balance.0)
            });

        // Require that the user has at least 50 VEX staked or deposited after the deposit
        let staked_balance = self.staked_amount_from_num_shares_rounded_up(stake_shares);
        require!(
            unstaked_balance + staked_balance + amount.0 >= FIFTY_VEX,
            "You must keep at least 50 VEX staked or deposited or be withdrawing all"
        );

        // Get the user's stake account or create a new one if it doesn't exist
        let relevant_account = self
            .users_stake
            .entry(sender_id)
            .or_insert_with(UserStake::default);

        // Update the user's unstaked_balance
        relevant_account.unstaked_balance = U128(relevant_account.unstaked_balance.0 + amount.0);
    }

    // Add USDC to the insurance fund
    pub(crate) fn add_to_insurance_fund(&mut self, amount: U128) {
        require!(
            env::predecessor_account_id() == self.usdc_token_contract,
            "Only USDC can be added"
        );

        self.insurance_fund = U128(self.insurance_fund.0 + amount.0);
    }

    // Helper function to calculate the number of stake shares from a staked amount
    // rounded down
    pub(crate) fn num_shares_from_staked_amount_rounded_down(&self, amount: u128) -> u128 {
        require!(
            self.total_staked_balance.0 > 0,
            "The total staked balance can't be 0"
        );

        (U256::from(self.total_stake_shares.0) * U256::from(amount)
            / U256::from(self.total_staked_balance.0))
        .as_u128()
    }

    // Helper function to calculate the number of stake shares from a staked amount
    // rounded up
    pub(crate) fn num_shares_from_staked_amount_rounded_up(&self, amount: u128) -> u128 {
        require!(
            self.total_staked_balance.0 > 0,
            "The total staked balance can't be 0"
        );
        ((U256::from(self.total_stake_shares.0) * U256::from(amount)
            + U256::from(self.total_staked_balance.0 - 1))
            / U256::from(self.total_staked_balance.0))
        .as_u128()
    }

    // Helper function to calculate the staked amount from the number of stake shares
    // rounded down
    pub(crate) fn staked_amount_from_num_shares_rounded_down(&self, num_shares: u128) -> u128 {
        require!(
            self.total_stake_shares.0 > 0,
            "The total number of stake shares can't be 0"
        );
        (U256::from(self.total_staked_balance.0) * U256::from(num_shares)
            / U256::from(self.total_stake_shares.0))
        .as_u128()
    }

    // Helper function to calculate the staked amount from the number of stake shares
    // rounded up
    pub(crate) fn staked_amount_from_num_shares_rounded_up(&self, num_shares: u128) -> u128 {
        require!(
            self.total_stake_shares.0 > 0,
            "The total number of stake shares can't be 0"
        );
        ((U256::from(self.total_staked_balance.0) * U256::from(num_shares)
            + U256::from(self.total_stake_shares.0 - 1))
            / U256::from(self.total_stake_shares.0))
        .as_u128()
    }
}
