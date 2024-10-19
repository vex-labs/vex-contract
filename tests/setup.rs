// This file contains the setup for all tests
// TODO: Make it optional to set up tokens and ref contracts

use near_sdk::json_types::{U128, U64};
use near_sdk::{AccountId, Gas, NearToken};
use near_workspaces::error::Error;
use near_workspaces::result::ExecutionFinalResult;
use near_workspaces::{Account, Contract, Result};
use serde_json::json;
use vex_contracts::Team;

const FIFTY_NEAR: NearToken = NearToken::from_near(50);
const FT_WASM_FILEPATH: &str = "./tests/fungible_token.wasm";
const REF_WASM_FILEPATH: &str = "./tests/ref_exchange_release.wasm";
pub const ONE_USDC: u128 = 1_000_000;
pub const ONE_VEX: u128 = 1_000_000_000_000_000_000;

pub struct TestSetup {
    pub alice: Account,
    pub bob: Account,
    pub admin: Account,
    pub main_contract: Contract,
    pub usdc_token_contract: Contract,
    pub vex_token_contract: Contract,
    pub ref_contract: Contract,
}

impl TestSetup {
    pub async fn new(incorrect_ft: bool) -> Result<Self, Box<dyn std::error::Error>> {
        // Create sandbox
        let sandbox = near_workspaces::sandbox().await?;

        // Create accounts
        let root: near_workspaces::Account = sandbox.root_account()?;

        let alice = create_account(&root, "alice").await?;
        let bob = create_account(&root, "bob").await?;
        let admin = create_account(&root, "admin").await?;

        // Deploy contract
        let contract_wasm = near_workspaces::compile_project("./").await?;
        // let contract_wasm = std::fs::read("./target/wasm32-unknown-unknown/release/main_contracts.wasm")?;
        let main_contract = sandbox.dev_deploy(&contract_wasm).await?;

        // Deploy USDC token contract
        let ft_wasm = std::fs::read(FT_WASM_FILEPATH)?;
        let usdc_token_contract = sandbox.dev_deploy(&ft_wasm).await?;

        // Deploy VEX token contract
        let vex_token_contract = sandbox.dev_deploy(&ft_wasm).await?;

        // Initialize USDC FT contract
        let mut res = usdc_token_contract
            .call("new_default_meta")
            .args_json(serde_json::json!({
                "owner_id": admin.id(),
                "total_supply": U128(1_000_000_000 * ONE_USDC), // 1 billion USDC
            }))
            .transact()
            .await?;

        assert!(res.is_success(), "Failed to initialize USDC token contract");

        // Initialize VEX FT contract
        res = vex_token_contract
            .call("new_default_meta")
            .args_json(serde_json::json!({
                "owner_id": admin.id(),
                "total_supply": U128(500_000_000 * ONE_VEX), // 500 million VEX
            }))
            .transact()
            .await?;

        assert!(res.is_success(), "Failed to initialize VEX token contract");

        // Deploy ref finance contract
        let ref_wasm = std::fs::read(REF_WASM_FILEPATH);
        let ref_contract = sandbox.dev_deploy(&ref_wasm?).await?;

        // Initialize ref contract
        res = ref_contract
            .call("new")
            .args_json(serde_json::json!({
                "owner_id": admin.id(),
                "exchange_fee": 4, // TODO: Check this is the same as the actual ref contract on mainnet and testnet
                "referral_fee": 1,
            }))
            .transact()
            .await?;

        assert!(
            res.is_success(),
            "Failed to initialize ref finance contract"
        );

        // Register accounts in FT contracts and send 1000 of each
        for account in [
            alice.clone(),
            bob.clone(),
            main_contract.as_account().clone(),
            ref_contract.as_account().clone(),
        ]
        .iter()
        {
            let mut register = account
                .call(usdc_token_contract.id(), "storage_deposit")
                .args_json(serde_json::json!({ "account_id": account.id() }))
                .deposit(NearToken::from_millinear(8))
                .transact()
                .await?;

            assert!(
                register.is_success(),
                "Failed to register account in USDC token contract"
            );

            register = account
                .call(vex_token_contract.id(), "storage_deposit")
                .args_json(serde_json::json!({ "account_id": account.id() }))
                .deposit(NearToken::from_millinear(8))
                .transact()
                .await?;

            assert!(
                register.is_success(),
                "Failed to register account in VEX token contract"
            );

            // Transfer 1000 USDC to accounts
            let transfer = ft_transfer(
                &admin,
                account.clone(),
                usdc_token_contract.clone(),
                U128(1000 * ONE_USDC),
            )
            .await?;
            assert!(
                transfer.is_success(),
                "Failed to transfer 100 FTs to account"
            );

            // Transfer 1000 VEX to accounts
            let transfer = ft_transfer(
                &admin,
                account.clone(),
                vex_token_contract.clone(),
                U128(1000 * ONE_USDC),
            )
            .await?;
            assert!(
                transfer.is_success(),
                "Failed to transfer 100 FTs to account"
            );
        }

        // Initialize main contract
        let ft_contract_id = if incorrect_ft {
            "incorrect_ft".parse::<AccountId>().unwrap()
        } else {
            usdc_token_contract.as_account().id().clone()
        };

        res = main_contract
            .call("init")
            .args_json(json!({"admin": admin.id(), "usdc_token_contract": ft_contract_id}))
            .transact()
            .await?;

        assert!(res.is_success(), "Failed to initialize main contract");

        // Set up pools in ref contract
        res = ref_contract
            .call("add_simple_pool")
            .args_json(serde_json::json!({
                "tokens": vec![usdc_token_contract.id(), vex_token_contract.id()],
                "fee": "25", // Check this fee is the same as our pools
            }))
            .deposit(NearToken::from_millinear(100))
            .transact()
            .await?;

        assert!(res.is_success(), "Failed to initialize ref contract");

        Ok(TestSetup {
            alice,
            bob,
            admin,
            main_contract,
            usdc_token_contract,
            vex_token_contract,
            ref_contract,
        })
    }
}

async fn create_account(root: &near_workspaces::Account, name: &str) -> Result<Account, Error> {
    let subaccount = root
        .create_subaccount(name)
        .initial_balance(FIFTY_NEAR)
        .transact()
        .await?
        .unwrap();

    Ok(subaccount)
}

pub async fn ft_transfer(
    root: &near_workspaces::Account,
    account: Account,
    usdc_token_contract: Contract,
    transfer_amount: U128,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let transfer = root
        .call(usdc_token_contract.id(), "ft_transfer")
        .args_json(serde_json::json!({
            "receiver_id": account.id(),
            "amount": transfer_amount
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    Ok(transfer)
}

#[allow(dead_code)]
pub async fn ft_balance_of(
    usdc_token_contract: &Contract,
    account_id: &AccountId,
) -> Result<U128, Box<dyn std::error::Error>> {
    let result = usdc_token_contract
        .view("ft_balance_of")
        .args_json(json!({"account_id": account_id}))
        .await?
        .json()?;

    Ok(result)
}

#[allow(dead_code)]
pub async fn ft_transfer_call(
    account: Account,
    usdc_token_contract_id: &AccountId,
    receiver_id: &AccountId,
    amount: U128,
    msg: String,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let transfer = account
        .call(usdc_token_contract_id, "ft_transfer_call")
        .args_json(serde_json::json!({"receiver_id": receiver_id, "amount": amount, "msg": msg }))
        .deposit(NearToken::from_yoctonear(1))
        .gas(Gas::from_tgas(50))
        .transact()
        .await?;

    Ok(transfer)
}

#[allow(dead_code)]
pub async fn claim(
    account: Account,
    main_contract_id: &AccountId,
    bet_id: U64,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let claim = account
        .call(main_contract_id, "claim")
        .args_json(serde_json::json!({"bet_id": bet_id}))
        .gas(Gas::from_tgas(50))
        .transact()
        .await?;

    Ok(claim)
}

#[allow(dead_code)]
pub async fn finish_match(
    account: Account,
    main_contract_id: &AccountId,
    match_id: &str,
    winner: Team,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let finish_match = account
        .call(main_contract_id, "finish_match")
        .args_json(serde_json::json!({"match_id": match_id, "winner": winner}))
        .gas(Gas::from_tgas(50))
        .transact()
        .await?;

    Ok(finish_match)
}

#[allow(dead_code)]
pub async fn end_betting(
    account: Account,
    main_contract_id: &AccountId,
    match_id: &str,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let end_betting = account
        .call(main_contract_id, "end_betting")
        .args_json(serde_json::json!({"match_id": match_id}))
        .gas(Gas::from_tgas(50))
        .transact()
        .await?;

    Ok(end_betting)
}

#[allow(dead_code)]
pub async fn cancel_match(
    account: Account,
    main_contract_id: &AccountId,
    match_id: &str,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let cancel_match = account
        .call(main_contract_id, "cancel_match")
        .args_json(serde_json::json!({"match_id": match_id}))
        .gas(Gas::from_tgas(50))
        .transact()
        .await?;

    Ok(cancel_match)
}

#[allow(dead_code)]
pub async fn change_admin(
    account: Account,
    main_contract_id: &AccountId,
    new_admin: AccountId,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let change_admin = account
        .call(main_contract_id, "change_admin")
        .args_json(serde_json::json!({"new_admin": new_admin}))
        .gas(Gas::from_tgas(50))
        .transact()
        .await?;

    Ok(change_admin)
}
