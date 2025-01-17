use near_sdk::{env, near, require};

use crate::events::Event;
use crate::*;

#[near]
impl Contract {
    // Staking VEX tokens
    pub(crate) fn stake(&mut self, sender_id: AccountId, amount: U128) {
        require!(
            env::predecessor_account_id() == self.vex_token_contract,
            "Only VEX can be staked"
        );

        // Get the rounded down number of stake shares
        let num_shares = self.num_shares_from_staked_amount_rounded_down(amount.0);

        // Get the amount of VEX for the rounded down stake shares
        let charge_amount = self.staked_amount_from_num_shares_rounded_down(num_shares);
        require!(
            charge_amount > 0,
            "Invariant violation. Calculated staked amount must be positive, because \"stake\" share price should be at least 1"
        );

        // Check if the user's staked VEX + the amount they are staking is at least 50
        let stake_shares = self
            .users_stake
            .get(&sender_id)
            .map_or(0, |account| account.stake_shares.0);
        let staked_balance = self.staked_amount_from_num_shares_rounded_down(stake_shares);
        require!(
            staked_balance + amount.0 >= FIFTY_VEX,
            "You must stake at least 50 VEX"
        );

        // Get the user's stake account or create a new one if it doesn't exist
        let relevant_account = self
            .users_stake
            .entry(sender_id.clone())
            .or_insert_with(UserStake::default);

        // Set the unstake timestamp to 1 week from now
        relevant_account.unstake_timestamp = U64(env::block_timestamp() + self.unstake_time_buffer);

        // Update the user's staked shares balance
        relevant_account.stake_shares = U128(relevant_account.stake_shares.0 + num_shares);

        // The staked amount that will be added to the total to guarantee the "stake" share price
        // doesnt decrease when staking because of rounding. The difference between `stake_amount` and `charge_amount` is paid
        // from the allocated STAKE_SHARE_PRICE_GUARANTEE_FUND.
        let stake_amount = self.staked_amount_from_num_shares_rounded_up(num_shares);

        // Update aggregate values
        self.total_staked_balance = U128(self.total_staked_balance.0 + stake_amount);
        self.total_stake_shares = U128(self.total_stake_shares.0 + num_shares);

        Event::StakeVex {
            account_id: &sender_id,
            amount,
            new_total_staked: self.total_staked_balance,
        }
        .emit();
    }

    // Add USDC to the contract
    pub(crate) fn add_usdc(&mut self, amount: U128) {
        require!(
            env::predecessor_account_id() == self.usdc_token_contract,
            "Only USDC can be added"
        );

        // First check if USDC needs to be added to funds_to_add
        // send the rest to the insurance fund
        if self.funds_to_add != U128(0) {
            if amount >= self.funds_to_add {
                let left_over = amount.0 - self.funds_to_add.0;
                self.funds_to_add = U128(0);
                self.insurance_fund = U128(self.insurance_fund.0 + left_over);
            } else {
                self.funds_to_add = U128(self.funds_to_add.0 - amount.0);
                return;
            }
        } else {
            self.insurance_fund = U128(self.insurance_fund.0 + amount.0);
        }
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
