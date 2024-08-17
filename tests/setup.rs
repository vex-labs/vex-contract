// This file contains the setup for all tests

use near_sdk::json_types::U128;
use near_sdk::{AccountId, Gas, NearToken};
use near_workspaces::error::Error;
use near_workspaces::network::Sandbox;
use near_workspaces::result::ExecutionFinalResult;
use near_workspaces::{Account, Contract, Result, Worker};
use serde_json::json;

const FIFTY_NEAR: NearToken = NearToken::from_near(50);
const FT_WASM_FILEPATH: &str = "./tests/fungible_token.wasm";
pub const ONE_USDC: u128 = 1_000_000_000_000_000_000_000_000;

pub struct TestSetup {
    pub sandbox: Worker<Sandbox>,
    pub alice: Account,
    pub bob: Account,
    pub admin: Account,
    pub vex_contract: Contract,
    pub usdc_contract: Contract,
}

impl TestSetup {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create sandbox
        let sandbox = near_workspaces::sandbox().await?;

        // Create accounts
        let root: near_workspaces::Account = sandbox.root_account()?;

        let alice = create_account(&root, "alice").await?;
        let bob = create_account(&root, "bob").await?;
        let admin = create_account(&root, "admin").await?;

        // Deploy VEX contract
        // let contract_wasm = near_workspaces::compile_project("./").await?;
        let contract_wasm = std::fs::read("./target/wasm32-unknown-unknown/release/vex_contracts.wasm")?;
        let vex_contract = sandbox.dev_deploy(&contract_wasm).await?;

        // Deploy FT contract
        let ft_wasm = std::fs::read(FT_WASM_FILEPATH)?;
        let usdc_contract = sandbox.dev_deploy(&ft_wasm).await?;

        // Initialize USDC FT contract
        let res = usdc_contract
            .call("new_default_meta")
            .args_json(serde_json::json!({
            "owner_id": admin.id(),
            "total_supply": U128(1_000_000_000 * ONE_USDC), // 1 billion USDC
            }))
            .transact()
            .await?;

        assert!(res.is_success(), "Failed to initialize USDC FT contract");

        // Register accounts in FT contract and send 100 FTs
        for account in [
            alice.clone(),
            bob.clone(),
            vex_contract.as_account().clone(),
        ]
        .iter()
        {
            let register = account
                .call(usdc_contract.id(), "storage_deposit")
                .args_json(serde_json::json!({ "account_id": account.id() }))
                .deposit(NearToken::from_millinear(8))
                .transact()
                .await?;

            assert!(register.is_success(), "Failed to register account in USDC FT contract");

            // Transfer 100 FTs to accounts
            let transfer = ft_transfer(
                &admin,
                account.clone(),
                usdc_contract.clone(),
                U128(100 * ONE_USDC),
            )
            .await?;
            assert!(transfer.is_success(), "Failed to transfer 100 FTs to account");
        }

        // Initialize VEX contract
        let init: ExecutionFinalResult = vex_contract
            .call("init")
            .args_json(
                json!({"admin": admin.id(), "usdc_contract": usdc_contract.as_account().id()}),
            )
            .transact()
            .await?;

        assert!(init.is_success(), "Failed to initialize VEX contract");

        Ok(TestSetup {
            sandbox,
            alice,
            bob,
            admin,
            vex_contract,
            usdc_contract,
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

async fn ft_transfer(
    root: &near_workspaces::Account,
    account: Account,
    usdc_contract: Contract,
    transfer_amount: U128,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let transfer = root
        .call(usdc_contract.id(), "ft_transfer")
        .args_json(serde_json::json!({
            "receiver_id": account.id(),
            "amount": transfer_amount
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    Ok(transfer)
}

pub async fn ft_balance_of(
    usdc_contract: &Contract,
    account_id: &AccountId,
) -> Result<U128, Box<dyn std::error::Error>> {
    let result = usdc_contract
        .view("ft_balance_of")
        .args_json(json!({"account_id": account_id}))
        .await?
        .json()?;

    Ok(result)
}

pub async fn ft_transfer_call(
    account: Account,
    usdc_contract_id: &AccountId,
    receiver_id: &AccountId,
    amount: U128,
    msg: String,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let transfer = account
        .call(usdc_contract_id, "ft_transfer_call")
        .args_json(serde_json::json!({"receiver_id": receiver_id, "amount": amount, "msg": msg }))
        .deposit(NearToken::from_yoctonear(1))
        .gas(Gas::from_tgas(50))
        .transact()
        .await?;

    Ok(transfer)
}
