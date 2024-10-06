use near_sdk::{env, near, require, Gas, NearToken};
use uint::construct_uint;

pub use crate::ext::*;
use crate::*;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

#[near]
impl Contract {
    pub(crate) fn deposit(&mut self, sender_id: AccountId, amount: U128) {
        require!(
            env::predecessor_account_id() == self.vex_contract,
            "Only VEX can be staked"
        );

        require!(
            amount >= FIFTY_VEX,
            "You must deposit at least 50 VEX"
        );

        let relevant_account = self
            .users_stake
            .entry(sender_id)
            .or_insert_with(UserStake::default);

        relevant_account.unstaked_balance = U128(relevant_account.unstaked_balance.0 + amount.0);
    }

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

        relevant_account.unstaked_balance = U128(relevant_account.unstaked_balance.0 - charge_amount);
        relevant_account.stake_shares = U128(relevant_account.stake_shares.0 + num_shares);
        relevant_account.unstake_timestamp = U64(env::block_timestamp() + 604_800_000_000_000); // One week in advance

        // The staked amount that will be added to the total to guarantee the "stake" share price
        // doesnt decrease when staking because of rounding. The difference between `stake_amount` and `charge_amount` is paid
        // from the allocated STAKE_SHARE_PRICE_GUARANTEE_FUND.
        let stake_amount = self.staked_amount_from_num_shares_rounded_up(num_shares);

        self.total_staked_balance = U128(self.total_staked_balance.0 + stake_amount);
        self.total_stake_shares = U128(self.total_stake_shares.0 + num_shares);
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
        relevant_account.unstaked_balance = U128(relevant_account.unstaked_balance.0 + receive_amount);

        // The staked amount that will be added to the total to guarantee the "stake" share price
        // doesnt decrease when unstaking because of rounding. The difference between `stake_amount` and `charge_amount` is paid
        // from the allocated STAKE_SHARE_PRICE_GUARANTEE_FUND.
        let stake_amount = self.staked_amount_from_num_shares_rounded_up(num_shares);

        self.total_staked_balance = U128(self.total_staked_balance.0 - stake_amount);
        self.total_stake_shares = U128(self.total_stake_shares.0 - num_shares);
    }

    pub fn withdraw(&mut self, amount: U128) {
        require!(amount.0 > 0, "Withdrawal amount should be positive");

        let account_id = env::predecessor_account_id();

        let mut relevant_account = self
            .users_stake
            .remove(&account_id)
            .unwrap_or_else(|| panic!("You do not have any stake"));

        require!(
            relevant_account.unstaked_balance >= amount,
            "Not enough unstaked balance to withdraw"
        );

        relevant_account.unstaked_balance = U128(relevant_account.unstaked_balance.0 - amount.0);

        if relevant_account.unstaked_balance.0 > 0 || relevant_account.stake_shares.0 > 0 {
            self.users_stake.insert(account_id.clone(), relevant_account);
        }

        ft_contract::ext(self.usdc_contract.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(Gas::from_tgas(30))
            .ft_transfer(account_id, amount);
    }

    pub(crate) fn add_to_insurance_fund(&mut self, amount: U128) {
        require!(
            env::predecessor_account_id() == self.usdc_contract,
            "Only USDC can be added"
        );

        self.insurance_fund = U128(self.insurance_fund.0 + amount.0);
    }

    pub(crate) fn num_shares_from_staked_amount_rounded_down(
        &self,
        amount: u128,
    ) -> u128 {
        require!(
            self.total_staked_balance.0 > 0,
            "The total staked balance can't be 0"
        );

        (U256::from(self.total_stake_shares.0) * U256::from(amount) / U256::from(self.total_staked_balance.0)).as_u128()
    }

    pub(crate) fn staked_amount_from_num_shares_rounded_down(
        &self,
        num_shares: u128,
    ) -> u128 {
        require!(
            self.total_stake_shares.0 > 0,
            "The total number of stake shares can't be 0"
        );
        (U256::from(self.total_staked_balance.0) * U256::from(num_shares) / U256::from(self.total_stake_shares.0)).as_u128()
    }

    pub(crate) fn staked_amount_from_num_shares_rounded_up(
        &self,
        num_shares: u128,
    ) -> u128 {
        require!(
            self.total_stake_shares.0 > 0,
            "The total number of stake shares can't be 0"
        );
        ((U256::from(self.total_staked_balance.0) * U256::from(num_shares)
            + U256::from(self.total_stake_shares.0 - 1))
            / U256::from(self.total_stake_shares.0))
        .as_u128()
    }

    pub(crate) fn num_shares_from_staked_amount_rounded_up(
        &self,
        amount: u128,
    ) -> u128 {
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
