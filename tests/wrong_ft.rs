use near_sdk::json_types::{U128, U64};
use near_sdk::NearToken;
use vex_contracts::Team;
mod setup;
use crate::setup::*;

const FT_WASM_FILEPATH: &str = "./tests/fungible_token.wasm";

#[tokio::test]

async fn test_wrong_ft() -> Result<(), Box<dyn std::error::Error>> {
    let TestSetup {
        alice,
        admin,
        vex_contract,
        sandbox,
        ..
    } = setup::TestSetup::new().await?;

    // Create another FT contract 
    let ft_wasm = std::fs::read(FT_WASM_FILEPATH)?;
    let ft_contract = sandbox.dev_deploy(&ft_wasm).await?;

    let res = ft_contract
        .call("new_default_meta")
        .args_json(serde_json::json!({
        "owner_id": admin.id(),
        "total_supply": U128(1_000_000_000 * ONE_USDC),
    }))
    .transact()
    .await?;

    assert!(res.is_success(), "Failed to initialize FT contract");

    // Register accounts in FT contract and send 100 FTs
    for account in [
        alice.clone(),
        vex_contract.as_account().clone(),
    ]
    .iter()
    {
        let register = account
            .call(ft_contract.id(), "storage_deposit")
            .args_json(serde_json::json!({ "account_id": account.id() }))
            .deposit(NearToken::from_millinear(8))
            .transact()
            .await?;

        assert!(register.is_success(), "Failed to register account in USDC FT contract");

        // Transfer 100 FTs to accounts
        let transfer = ft_transfer(
            &admin,
            account.clone(),
            ft_contract.clone(),
            U128(100 * ONE_USDC),
        )
        .await?;
        assert!(transfer.is_success(), "Failed to transfer 100 FTs to account");
    }

    // Create a new match
    let create_match = admin
    .call(vex_contract.id(), "create_match")
    .args_json(serde_json::json!({"game": "CSGO", "team_1": "RUBY", "team_2": "Nexus", "in_odds_1": 1.2, "in_odds_2": 1.6, "date": "17/08/2024"}))
    .transact()
    .await?;

    assert!(create_match.is_success());

    // Alices attempts to place a bet of 10 FTs on team 1
    let alice_bet = ft_transfer_call(
        alice.clone(),
        ft_contract.id(),
        vex_contract.id(),
        U128(10 * ONE_USDC),
        serde_json::json!({"match_id": "RUBY-Nexus-17/08/2024", "team": Team::Team1}).to_string(),
    )
    .await?;

    assert!(alice_bet.is_success(), "ft_transfer_call failed on Alice's bet");

    let vex_contract_balance: U128 = ft_balance_of(&ft_contract, vex_contract.id()).await?;
    assert_eq!(vex_contract_balance, U128(100 * ONE_USDC), "Vex contract balance is not correct after Alice's first bet");
    let alice_balance: U128 = ft_balance_of(&ft_contract, alice.id()).await?;
    assert_eq!(alice_balance, U128(100 * ONE_USDC), "Alice's balance is not correct after her first bet");

    let bet = vex_contract.view("get_bet").args_json(serde_json::json!({"bettor": alice.id(), "bet_id": U64(1)})).await;
    assert!(bet.is_err(), "Managed to get Alice's bet");

    // TDOD Check odds haven't changed

    Ok(())
}