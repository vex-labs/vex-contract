use near_sdk::env;

use crate::*;

#[near]
impl Contract {
    // Get $VEX staking balance for a user if they were to unstake now
    pub fn get_user_staked_bal(&self, account_id: AccountId) -> Option<U128> {
        let relevant_account = match self.users_stake.get(&account_id) {
            Some(account) => account,
            None => return None,
        };

        Some(U128(self.staked_amount_from_num_shares_rounded_down(
            relevant_account.stake_shares.0,
        )))
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

    // Get amount of USDC that needs to be added to the contract
    pub fn get_funds_to_add(&self) -> U128 {
        self.funds_to_add
    }

    pub fn can_stake_swap_happen(&self) -> bool {
        if self.staking_rewards_queue.is_empty() {
            return false;
        }

        let time_passed = env::block_timestamp() - self.last_stake_swap_timestamp.0;
        // Check if the first item in staking rewards queue has expired
        // repeat until the first item in the queue has not expired
        let mut finished_matches_rewards: u128 = 0;
        let mut new_usdc_staking_rewards = self.usdc_staking_rewards.0;
        for i in self.staking_rewards_queue.iter() {
            if i.stake_end_time.0 < env::block_timestamp() {
                let finished_match_time_passed =
                    i.stake_end_time.0 - self.last_stake_swap_timestamp.0;

                // Get rewards for this match has passed
                let passed_match_reward = (U256::from(finished_match_time_passed)
                    * U256::from(i.staking_rewards.0)
                    / U256::from(self.rewards_period))
                .as_u128();

                finished_matches_rewards += passed_match_reward;
                new_usdc_staking_rewards -= i.staking_rewards.0;
            } else {
                break;
            }
        }

        // Calculate the rewards for the matches that have not expired
        let active_match_rewards = (U256::from(time_passed) * U256::from(new_usdc_staking_rewards)
            / U256::from(self.rewards_period))
        .as_u128();

        let total_rewards_to_swap = finished_matches_rewards + active_match_rewards;

        if total_rewards_to_swap > self.min_swap_amount {
            return true;
        }

        false
    }
}
