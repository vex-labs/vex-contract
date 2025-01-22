use std::str::FromStr;

use near_sdk::{json_types::{U128, U64}, test_utils::VMContextBuilder, testing_env, AccountId, NearToken};

use crate::Contract;

fn owner() -> AccountId {
    AccountId::from_str("owner.testnet").unwrap()
}

fn admin() -> AccountId {
    AccountId::from_str("admin.testnet").unwrap()
}

fn usdc_account() -> AccountId {
    AccountId::from_str("usdc.testnet").unwrap()
}

fn vex_token_account() -> AccountId {
    AccountId::from_str("vex_token.testnet").unwrap()
}

fn treasury_account() -> AccountId {
    AccountId::from_str("treasury.testnet").unwrap()
}

fn ref_finance_account() -> AccountId {
    AccountId::from_str("ref_finance.testnet").unwrap()
}

const TEST_REF_POOL_ID: U64 = U64(1);
const REWARDS_PERIOD: U64 = U64(100);
const UNSTAKE_TIME_BUFFER: U64 = U64(10);
const MIN_SWAP_AMMOUNT: U128 = U128(500);


pub fn setup(
    contract_owner_account_id: Option<AccountId>,
    contract_predecessor_account_id: Option<AccountId>,
) -> (Contract, VMContextBuilder) {
    let mut context = VMContextBuilder::new();

    let contract_owner_account_id = contract_owner_account_id.unwrap_or(owner());
    context.current_account_id(contract_owner_account_id.clone());

    //setting predecessor to owner to simulate contract deployment.
    //with contract deployment the predecessor will be equal to current account.
    context.predecessor_account_id(contract_owner_account_id.clone());

    context.account_balance(NearToken::from_near(50));

    testing_env!(context.build());

    let contract = Contract::init(admin(), usdc_account(), vex_token_account(), treasury_account(), refinance_account(), TEST_REF_POOL_ID, REWARDS_PERIOD, UNSTAKE_TIME_BUFFER, MIN_SWAP_AMMOUNT);

    //now, after the contract has been deployed we can switch predecessor to whatever our test requires.
    context.predecessor_account_id(contract_predecessor_account_id.unwrap_or(owner()));

    testing_env!(context.build());

    (contract, context)
}