use near_sdk::json_types::U128;
use near_sdk::{ext_contract, near, AccountId, PromiseOrValue};

// FT transfer interface
#[allow(dead_code)]
#[ext_contract(ft_contract)]
trait FT {
    fn ft_transfer(&self, receiver_id: AccountId, amount: U128);

    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: String,
        msg: String,
    ) -> PromiseOrValue<U128>;
}
