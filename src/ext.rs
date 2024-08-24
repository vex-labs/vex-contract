use near_sdk::json_types::U128;
use near_sdk::{ext_contract, AccountId};

// FT transfer interface
#[allow(dead_code)]
#[ext_contract(ft_contract)]
trait FT {
    fn ft_transfer(&self, receiver_id: AccountId, amount: U128);
}
