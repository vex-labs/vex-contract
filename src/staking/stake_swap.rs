use near_sdk::{env, near, require, Gas, NearToken, PromiseError};

pub use crate::ext::*;
use crate::*;

#[near]
impl Contract {
    // Swap the USDC staking rewards for VEX
    pub fn perform_stake_swap(&mut self) {
        require!(
            env::prepaid_gas() >= Gas::from_tgas(300),
            "You need to attach 300 TGas"
        );

        let previous_timestamp = self.last_stake_swap_timestamp;
        self.last_stake_swap_timestamp = U64(env::block_timestamp());

        // If the staking queue is empty then skip and update the last stake swap timestamp
        if self.staking_rewards_queue.is_empty() {
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

        require!(
            total_rewards_to_swap > 100 * ONE_USDC,
            "Rewards to swap must be greather than 100"
        );

        let caller = env::predecessor_account_id();

        // Call to ref finance to deposit the USDC rewards
        // Callback to ref_profit_deposit_callback
        // If this function fails we can call the function again
        // as state is reversed in the callback
        ft_contract::ext(self.usdc_token_contract.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(Gas::from_tgas(30))
            .ft_transfer_call(
                self.ref_contract.clone(),
                U128(total_rewards_to_swap),
                "".to_string(),
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::from_tgas(220))
                    .ref_profit_deposit_callback(
                        num_to_pop,
                        U128(new_usdc_staking_rewards),
                        previous_timestamp,
                        caller,
                    ),
            );
    }

    // Callback after the deposit to ref finance
    #[private]
    pub fn ref_profit_deposit_callback(
        &mut self,
        #[callback_result] call_result: Result<U128, PromiseError>,
        num_to_pop: u16,
        new_usdc_staking_rewards: U128,
        previous_timestamp: U64,
        caller: AccountId,
    ) {
        // If the call to ref finance failed then revert the state
        if call_result.is_err() {
            self.last_stake_swap_timestamp = previous_timestamp;
            return;
        }

        let amount_deposited = call_result.unwrap();

        // Set the new staking rewards since some matches have expired
        self.usdc_staking_rewards = new_usdc_staking_rewards;

        // Remove the finished matches from the queue
        for _ in 0..num_to_pop {
            self.staking_rewards_queue.pop_front();
        }

        let action = create_swap_args(
            self.ref_pool_id,
            self.usdc_token_contract.clone(),
            self.usdc_token_contract.clone(),
            amount_deposited,
            U128(0),
        );

        // Call to ref finance to swap the deposited USDC for VEX
        // Callback to ref_profit_swap_callback
        // If this call fails there will be USDC funds locked in ref finance
        // we will implement a function to carry on the swap from this point
        ref_contract::ext(self.ref_contract.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(Gas::from_tgas(30))
            .swap(action)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::from_tgas(150))
                    .ref_profit_swap_callback(caller),
            );
    }

    // Callback after the swap in ref finance
    #[private]
    pub fn ref_profit_swap_callback(
        &mut self,
        #[callback_result] call_result: Result<U128, PromiseError>,
        caller: AccountId,
    ) {
        let amount_swapped = call_result.unwrap_or_else(|_| panic!("Swap in ref finance failed"));

        // Call to ref finance to withdraw the VEX that was swapped into
        // Callback to ref_profit_withdraw_callback
        // If this call fails there will be VEX funds locked in ref finance
        // we will implement a function to carry on the withdraw from this point
        ref_contract::ext(self.ref_contract.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(Gas::from_tgas(30))
            .withdraw(self.vex_token_contract.clone(), amount_swapped)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::from_tgas(100))
                    .ref_profit_withdraw_callback(caller),
            );
    }

    // Callback after withdrawing the VEX from ref finance
    #[private]
    pub fn ref_profit_withdraw_callback(
        &mut self,
        #[callback_result] call_result: Result<U128, PromiseError>,
        caller: AccountId,
    ) {
        let amount_withdrawn =
            call_result.unwrap_or_else(|_| panic!("Withdraw from ref finance failed"));

        // Reward the initial caller for some amount of VEX
        let passed_match_reward = (U256::from(amount_withdrawn.0) / U256::from(100)).as_u128();

        ft_contract::ext(self.usdc_token_contract.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(Gas::from_tgas(30))
            .ft_transfer(caller, U128(passed_match_reward));

        let left_over_rewards = amount_withdrawn.0 - passed_match_reward;

        // Add the withdrawn VEX to the total staked balance
        self.total_staked_balance = U128(self.total_staked_balance.0 + left_over_rewards);
    }
}