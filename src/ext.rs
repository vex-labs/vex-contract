use near_sdk::json_types::U128;
use near_sdk::{ext_contract, near, AccountId, PromiseOrValue};

#[near(serializers = [json])]
pub struct Action {
    pool_id: u64,
    token_in: AccountId,
    token_out: AccountId,
    amount_in: U128,
    min_amount_out: U128,
}

pub fn create_swap_args(
    pool_id: u64,
    token_in: AccountId,
    token_out: AccountId,
    amount_in: U128,
    min_amount_out: U128,
) -> Vec<Action> {
    let action = Action {
        pool_id,
        token_in,
        token_out,
        amount_in: amount_in,
        min_amount_out: min_amount_out,
    };

    vec![action]
}

// FT transfer interface
#[allow(dead_code)]
#[ext_contract(ft_contract)]
trait FT {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128);

    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;
}

#[allow(dead_code)]
#[ext_contract(ref_contract)]
trait Ref {
    fn swap(&mut self, actions: Vec<Action>) -> U128;

    fn withdraw(&mut self, token_id: AccountId, amount: U128) -> U128;
}
