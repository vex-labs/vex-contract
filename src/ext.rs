use near_sdk::json_types::U128;
use near_sdk::{ext_contract, near, serde_json, AccountId, PromiseOrValue};

#[near(serializers = [json])]
struct RefInnerMsg {
    pool_id: u64,
    token_in: AccountId,
    token_out: AccountId,
    amount_in: U128,
    amount_out: String,
    min_amount_out: String,
}

#[near(serializers = [json])]
struct RefSwapMsg {
    force: u8,
    actions: Vec<RefInnerMsg>,
}

// fn create_ref_message(
//     pool_id: u64,
//     token_in: &str,
//     token_out: &str,
//     amount_in: &str,
//     amount_out: &str,
//     min_amount_out: &str,
// ) -> String {
//     // Create the RefInnerMsg instance
//     let action = RefInnerMsg {
//         pool_id,
//         token_in: token_in.to_string(),
//         token_out: token_out.to_string(),
//         amount_in: amount_in.to_string(),
//         amount_out: amount_out.to_string(),
//         min_amount_out: min_amount_out.to_string(),
//     };

//     // Create the RefSwapMsg instance with force set to 0
//     let message = RefSwapMsg {
//         force: 0,
//         actions: vec![action],
//     };

//     // Serialize the RefSwapMsg to JSON
//     serde_json::to_string(&message).unwrap()
// }

// WIP

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
