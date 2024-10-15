use crate::*;

use near_sdk::require;

#[near]
impl Contract {
    // Get $VEX staking balance for a user if they were to unstake now
    pub fn get_user_staked_bal(&self, account_id: AccountId) -> U128 {
        let relevant_account = self
            .users_stake
            .get(&account_id)
            .unwrap_or_else(|| panic!("You do not have any stake"));

        require!(
            relevant_account.stake_shares.0 > 0,
            "You do not have any stake"
        );

        U128(self.staked_amount_from_num_shares_rounded_down(relevant_account.stake_shares.0))
    }

    // Get a user's stake info
    pub fn get_user_stake_info(&self, account_id: AccountId) -> &UserStake {
        self.users_stake
            .get(&account_id)
            .unwrap_or_else(|| panic!("You do not have any stake"))
    }

    // Get USDC staking rewards queue
    pub fn get_staking_rewards_queue(&self) -> &VecDeque<MatchStakeInfo> {
        &self.staking_rewards_queue
    }

    // Get total USDC staking rewards
    pub fn get_usdc_staking_rewards(&self) -> U128 {
        self.usdc_staking_rewards
    }

    // Get last stake swap timestamp
    pub fn get_last_stake_swap_timestamp(&self) -> U64 {
        self.last_stake_swap_timestamp
    }

    // Get total $VEX staked amount
    pub fn get_total_staked_balance(&self) -> U128 {
        self.total_staked_balance
    }

    // Get total $VEX stake shares
    pub fn get_total_stake_shares(&self) -> U128 {
        self.total_stake_shares
    }

    // Get fees fund balance
    pub fn get_fees_fund(&self) -> U128 {
        self.fees_fund
    }

    // Get insurance fund balance
    pub fn get_insurance_fund(&self) -> U128 {
        self.insurance_fund
    }

    // Get VEX deposits paused status
    pub fn get_vex_deposits_paused(&self) -> bool {
        self.vex_deposits_paused
    }
}

