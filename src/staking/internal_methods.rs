use near_sdk::{env, near, require};

use crate::*;

#[near]
impl Contract {
    pub(crate) fn deposit(&mut self, sender_id: AccountId, amount: U128) {
        require!(
            env::predecessor_account_id() == self.vex_token_contract,
            "Only VEX can be staked"
        );

        require!(amount >= FIFTY_VEX, "You must deposit at least 50 VEX");

        let relevant_account = self
            .users_stake
            .entry(sender_id)
            .or_insert_with(UserStake::default);

        relevant_account.unstaked_balance = U128(relevant_account.unstaked_balance.0 + amount.0);
    }

    pub(crate) fn add_to_insurance_fund(&mut self, amount: U128) {
        require!(
            env::predecessor_account_id() == self.usdc_contract,
            "Only USDC can be added"
        );

        self.insurance_fund = U128(self.insurance_fund.0 + amount.0);
    }

    pub(crate) fn num_shares_from_staked_amount_rounded_down(&self, amount: u128) -> u128 {
        require!(
            self.total_staked_balance.0 > 0,
            "The total staked balance can't be 0"
        );

        (U256::from(self.total_stake_shares.0) * U256::from(amount)
            / U256::from(self.total_staked_balance.0))
        .as_u128()
    }

    pub(crate) fn staked_amount_from_num_shares_rounded_down(&self, num_shares: u128) -> u128 {
        require!(
            self.total_stake_shares.0 > 0,
            "The total number of stake shares can't be 0"
        );
        (U256::from(self.total_staked_balance.0) * U256::from(num_shares)
            / U256::from(self.total_stake_shares.0))
        .as_u128()
    }

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
}
