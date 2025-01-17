use near_sdk::{env, near, require, Gas, NearToken, Promise};

use crate::events::Event;
pub use crate::ext::*;
use crate::*;

#[near]
impl Contract {
    // Unstake a given amount of VEX to their unstaked balance
    pub fn unstake(&mut self, amount: U128) -> Promise {
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

        // The amount tokens that will be unstaked from the total to guarantee the "stake" share
        // price never decreases. The difference between `receive_amount` and `unstake_amount` is
        // paid from the allocated STAKE_SHARE_PRICE_GUARANTEE_FUND.
        let unstake_amount = self.staked_amount_from_num_shares_rounded_down(num_shares);

        // Get the staked balance of the user
        let staked_balance =
            self.staked_amount_from_num_shares_rounded_up(relevant_account.stake_shares.0);

        require!(
            staked_balance - unstake_amount >= 50
                || relevant_account.stake_shares.0 - num_shares == 0,
            "You must keep at least 50 VEX staked or withdraw all"
        );

        let relevant_account = self
            .users_stake
            .get_mut(&account_id)
            .unwrap_or_else(|| panic!("You do not have any stake"));

        // Subtract the number of shares from the stake shares and add the receive amount to the unstaked balance
        relevant_account.stake_shares = U128(relevant_account.stake_shares.0 - num_shares);

        self.total_staked_balance = U128(self.total_staked_balance.0 - unstake_amount);
        self.total_stake_shares = U128(self.total_stake_shares.0 - num_shares);

        // If the user has no stake shares, remove them from the map
        if relevant_account.stake_shares.0 == 0 {
            self.users_stake.remove(&account_id);
        }

        Event::UnstakeVex {
            account_id: &account_id,
            amount,
            new_total_staked: self.total_staked_balance,
        }
        .emit();

        // Transfer the amount of VEX to the user
        ft_contract::ext(self.vex_token_contract.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(Gas::from_tgas(30))
            .ft_transfer(account_id, U128(receive_amount))
    }

    // Unstake all VEX to their unstaked balance
    pub fn unstake_all(&mut self) -> Promise {
        let account_id = env::predecessor_account_id();

        let relevant_account = self
            .users_stake
            .get(&account_id)
            .unwrap_or_else(|| panic!("You do not have any stake"));

        // Get the total amount of VEX the account will receive by unstaking all the "stake" shares
        let amount =
            self.staked_amount_from_num_shares_rounded_down(relevant_account.stake_shares.0);

        // Call unstake with the full amount from staked balance
        self.unstake(U128(amount))
    }
}
