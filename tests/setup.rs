// This file contains the setup for all tests 

use near_workspaces::network::Sandbox;
use near_workspaces::result::ExecutionFinalResult;
use near_workspaces::{Account, Contract, Worker, Result};
use near_workspaces::error::Error;
use near_sdk::NearToken;
use near_sdk::json_types::U128;

const TEN_NEAR: NearToken = NearToken::from_near(10);


pub struct TestSetup {
    pub sandbox: Worker<Sandbox>,
    pub alice: Account,
    pub bob: Account,
}

impl TestSetup {
    pub async fn new() -> Result<Self, Error> {
        let sandbox = near_workspaces::sandbox().await?;

        let root: near_workspaces::Account = sandbox.root_account()?;

        let alice = create_account(&root, "alice").await?;
        let bob = create_account(&root, "bob").await?;

        Ok(TestSetup { sandbox, alice, bob })
    }
}

async fn create_account(
    root: &near_workspaces::Account,
    name: &str,
) -> Result<Account, Error> {
    let subaccount = root
        .create_subaccount(name)
        .initial_balance(TEN_NEAR)
        .transact()
        .await?
        .unwrap();

    Ok(subaccount)
}