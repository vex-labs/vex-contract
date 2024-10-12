use near_sdk::json_types::U128;
use near_sdk::{ext_contract, near, serde_json, AccountId, PromiseOrValue};

#[near(serializers = [json])]
struct RefInnerMsg {
    pool_id: u64,
    token_in: AccountId,
    token_out: AccountId,
    amount_in: U128,
    min_amount_out: U128,
}

#[near(serializers = [json])]
struct RefSwapMsg {
    force: u8,
    actions: Vec<RefInnerMsg>,
}

pub fn create_ref_message(
    pool_id: u64,
    token_in: AccountId,
    token_out: AccountId,
    amount_in: u128,
    min_amount_out: u128,
) -> String {
    // Create the RefInnerMsg instance
    let action = RefInnerMsg {
        pool_id,
        token_in,
        token_out,
        amount_in: U128(amount_in),
        min_amount_out: U128(min_amount_out),
    };

    // Create the RefSwapMsg instance with force set to 0
    let message = RefSwapMsg {
        force: 0,
        actions: vec![action],
    };

    // Serialize the RefSwapMsg to JSON
    serde_json::to_string(&message).unwrap()
}

// FT transfer interface
#[allow(dead_code)]
#[ext_contract(ft_contract)]
trait FT {
    fn ft_transfer(&self, receiver_id: AccountId, amount: U128);

    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;

    fn ft_balance_of(&self, account_id: AccountId) -> U128;
}
