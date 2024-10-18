use near_sdk::{env, near, Gas, NearToken, PromiseError};

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
        let difference = loss - self.insurance_fund.0;

        // Cross contract call to check how much vex is needed to be sold to cover the difference + 10%
        // the input amount can change between blocks so add this 10% buffer

        // swap this amount of vex to usdc

        // Check that the amount of returned usdc + insurance fund is enough to cover the loss
        // If not emit some log that we gotta add money to the contract (this should hopefully never happen), have a variable to keep track of amount owed
        // add left over usdc to insurance fund
    }
}
