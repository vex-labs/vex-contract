use near_sdk::{env, near, Gas, NearToken};

pub use crate::ext::*;
use crate::*;

#[near]
impl Contract {
    // Handles the case when a match finishes and there is a profit
    pub(crate) fn handle_profit(&mut self, profit: u128) {
        // Calculate how profit is distributed
        let usdc_for_staking = (U256::from(60) * U256::from(profit) / U256::from(100)).as_u128();
        let treasury_rewards = (U256::from(30) * U256::from(profit) / U256::from(100)).as_u128();
        let insurace_rewards = (U256::from(5) * U256::from(profit) / U256::from(100)).as_u128();
        let fees_rewards = profit - usdc_for_staking - treasury_rewards - insurace_rewards;

        // Increase fees and insurance funds
        self.fees_fund = U128(self.fees_fund.0 + fees_rewards);
        self.insurance_fund = U128(self.insurance_fund.0 + insurace_rewards);

        // Send funds to treasury
        ft_contract::ext(self.usdc_token_contract.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(Gas::from_tgas(30))
            .ft_transfer(self.treasury.clone(), U128(treasury_rewards));

        // Peform stake swap so rewards are distributed at the timestamp of the
        // match being added to the list so extra rewards are not distributed
        self.perform_stake_swap();

        self.usdc_staking_rewards = U128(self.usdc_staking_rewards.0 + usdc_for_staking);

        // Add to staking rewards queue
        let match_stake_info = MatchStakeInfo {
            staking_rewards: U128(usdc_for_staking),
            stake_end_time: U64(env::block_timestamp() + ONE_MONTH),
        };
        self.staking_rewards_queue.push_back(match_stake_info);
    }
}
