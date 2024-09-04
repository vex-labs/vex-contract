use near_sdk::json_types::{U128, U64};
use vex_contracts::Team;
mod setup;
use crate::setup::*;

#[tokio::test]

async fn test_wrong_ft() -> Result<(), Box<dyn std::error::Error>> {
    let TestSetup {
        alice,
        admin,
        vex_contract,
        usdc_contract,
        ..
    } = setup::TestSetup::new(true).await?;

    // Create a new match
    let mut result = admin
    .call(vex_contract.id(), "create_match")
    .args_json(serde_json::json!({"game": "CSGO", "team_1": "RUBY", "team_2": "Nexus", "in_odds_1": 1.2, "in_odds_2": 1.6, "date": "17/08/2024"}))
    .transact()
    .await?;

    assert!(result.is_success());

    // Alices attempts to place a bet of 10 FTs on team 1
    result = ft_transfer_call(
        alice.clone(),
        usdc_contract.id(),
        vex_contract.id(),
        U128(10 * ONE_USDC),
        serde_json::json!({"match_id": "RUBY-Nexus-17/08/2024", "team": Team::Team1}).to_string(),
    )
    .await?;

    assert!(
        result.is_success(),
        "ft_transfer_call failed on Alice's bet"
    );

    let mut balance = ft_balance_of(&usdc_contract, vex_contract.id()).await?;
    assert_eq!(
        balance,
        U128(100 * ONE_USDC),
        "Vex contract balance is not correct after Alice's first bet"
    );
    balance = ft_balance_of(&usdc_contract, alice.id()).await?;
    assert_eq!(
        balance,
        U128(100 * ONE_USDC),
        "Alice's balance is not correct after her first bet"
    );

    let bet = vex_contract
        .view("get_bet")
        .args_json(serde_json::json!({"bettor": alice.id(), "bet_id": U64(1)}))
        .await;
    assert!(bet.is_err(), "Managed to get Alice's bet");

    Ok(())
}
