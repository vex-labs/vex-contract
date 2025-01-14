use near_sdk::{env, log, near, Gas, NearToken, PromiseError};

pub use crate::ext::*;
use crate::*;

#[near]
impl Contract {
    // Handles the case when a match finishes and there is a loss
    pub(crate) fn handle_loss(&mut self, loss: u128) {
        // If the loss can be covered by the insurance fund then do so
        if loss < self.insurance_fund.0 {
            self.insurance_fund = U128(self.insurance_fund.0 - loss);
            return;
        }

        // Calculate how much more USDC is needed to cover the loss
        let difference = U128(loss - self.insurance_fund.0);

        // Cross contract call to check how much VEX is needed to be sold to cover the difference
        // callback to ref_loss_view_callback
        // if this call fails we can call the function again
        // as no state was changed
        ref_contract::ext(self.ref_contract.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(Gas::from_tgas(30))
            .get_return_by_output(
                self.ref_pool_id,
                self.vex_token_contract.clone(),
                difference,
                self.usdc_token_contract.clone(),
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::from_tgas(250))
                    .ref_loss_view_callback(difference),
            );
    }

    // Callback after getting the return from ref finance
    #[private]
    pub fn ref_loss_view_callback(
        &mut self,
        #[callback_result] call_result: Result<U128, PromiseError>,
        difference: U128,
    ) {
        let amount_in = call_result.unwrap_or_else(|_| panic!("View in ref finance failed"));

        // Add an extra 5% to the amount to swap to account for price change between blocks
        let amount_to_swap =
            (U256::from(105) * U256::from(amount_in.0) / U256::from(100)).as_u128();

        // Call to ref finance to deposit the USDC rewards
        // Callback to ref_loss_deposit_callback
        // If this call fails we can call the function again
        // as no state is changed
        ft_contract::ext(self.vex_token_contract.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(Gas::from_tgas(30))
            .ft_transfer_call(
                self.ref_contract.clone(),
                U128(amount_to_swap),
                "".to_string(),
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::from_tgas(200))
                    .ref_loss_deposit_callback(difference),
            );
    }

    // Callback after depositing the VEX to swap
    #[private]
    pub fn ref_loss_deposit_callback(
        &mut self,
        #[callback_result] call_result: Result<U128, PromiseError>,
        difference: U128,
    ) {
        let amount_deposited =
            call_result.unwrap_or_else(|_| panic!("Deposit to ref finance failed"));

        // Set the insurance fund to zero as it will all be used
        self.insurance_fund = U128(0);

        // Remove the amount of VEX deposited from the total staked balance
        self.total_staked_balance = U128(self.total_staked_balance.0 - amount_deposited.0);

        let action = create_swap_args(
            self.ref_pool_id,
            self.vex_token_contract.clone(),
            self.usdc_token_contract.clone(),
            amount_deposited,
            U128(0),
        );

        // Call to ref finance to swap the deposited VEX for USDC
        // Callback to ref_loss_swap_callback
        // If this call fails there will be VEX funds locked in ref finance
        // also state will have been changed so
        // we will implement a function to carry on the swap from this point
        ref_contract::ext(self.ref_contract.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(Gas::from_tgas(30))
            .swap(action)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::from_tgas(150))
                    .ref_loss_swap_callback(difference),
            );
    }

    // Callback after the swap in ref finance
    #[private]
    pub fn ref_loss_swap_callback(
        &mut self,
        #[callback_result] call_result: Result<U128, PromiseError>,
        difference: U128,
    ) {
        let amount_swapped_for =
            call_result.unwrap_or_else(|_| panic!("Swap in ref finance failed"));

        // Call to ref finance to withdraw the USDC that was swapped into
        // Callback to ref_loss_withdraw_callback
        // If this call fails there will be USDC funds locked in ref finance
        // we will implement a function to carry on the withdraw from this point
        ref_contract::ext(self.ref_contract.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(Gas::from_tgas(30))
            .withdraw(self.usdc_token_contract.clone(), amount_swapped_for)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::from_tgas(100))
                    .ref_loss_withdraw_callback(difference),
            );
    }

    // Callback after withdrawing the USDC from ref finance
    #[private]
    pub fn ref_loss_withdraw_callback(
        &mut self,
        #[callback_result] call_result: Result<U128, PromiseError>,
        difference: U128,
    ) {
        let amount_withdrawn =
            call_result.unwrap_or_else(|_| panic!("Withdraw from ref finance failed"));

        // Check if the amount received is indeed greater than
        // the difference that is needed to be covered
        if amount_withdrawn >= difference {
            // If we have excess USDC then we can add it to the insurance fund
            let excess = U128(amount_withdrawn.0 - difference.0);
            self.insurance_fund = excess;
        } else {
            // In the very rare case that the amount received is less than the difference
            // we will log this and add to the state some amount needs to be added
            log!("URGENT: Need to add funds to the contract!");
            self.funds_to_add = U128(difference.0 - amount_withdrawn.0);
        }
    }
}
