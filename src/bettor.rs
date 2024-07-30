use crate::*;
use near_sdk::near;

#[near]
impl Contract {
    pub fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
    }

    pub fn claim_winnings(&mut self, match_id: MatchID, bet_id: BetID) -> U128 {}

    pub fn claim_refund(&mut self, match_id: MatchID, bet_id: BetID) -> U128 {}
}
