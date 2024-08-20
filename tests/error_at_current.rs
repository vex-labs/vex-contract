use near_sdk::json_types::{U128, U64};
use vex_contracts::Team;
mod setup;
use crate::setup::*;


#[tokio::test]

async fn test_error_at_current() -> Result<(), Box<dyn std::error::Error>> {
    let TestSetup {
        alice,
        bob,
        admin,
        vex_contract,
        usdc_contract,
        ..
    } = setup::TestSetup::new().await?;

    // Create a new match
    let create_match = admin
    .call(vex_contract.id(), "create_match")
    .args_json(serde_json::json!({"game": "CSGO", "team_1": "RUBY", "team_2": "Nexus", "in_odds_1": 1.2, "in_odds_2": 1.6, "date": "17/08/2024"}))
    .transact()
    .await?;

    assert!(create_match.is_success(), "Admin failed to create a match");

    // Alice places a bet of 10 USDC on team 1
    let alice_bet = ft_transfer_call(
        alice.clone(),
        usdc_contract.id(),
        vex_contract.id(),
        U128(10 * ONE_USDC),
        serde_json::json!({"match_id": "RUBY-Nexus-17/08/2024", "team": Team::Team1}).to_string(),
    )
    .await?;

    assert!(alice_bet.is_success(), "ft_transfer_call failed on Alice's first bet");

    let mut vex_contract_balance: U128 = ft_balance_of(&usdc_contract, vex_contract.id()).await?;
    assert_eq!(vex_contract_balance, U128(110 * ONE_USDC), "Vex contract balance is not correct after Alice's first bet");
    let mut alice_balance: U128 = ft_balance_of(&usdc_contract, alice.id()).await?;
    assert_eq!(alice_balance, U128(90 * ONE_USDC), "Alice's balance is not correct after her first bet");

    let mut bet = vex_contract.view("get_bet").args_json(serde_json::json!({"bettor": alice.id(), "bet_id": U64(1)})).await;
    assert!(bet.is_ok(), "Failed to get Alice's bet");

    // Bob places a bet of 5 USDC on the losing team
    let bob_bet = ft_transfer_call(
        bob.clone(),
        usdc_contract.id(),
        vex_contract.id(),
        U128(5 * ONE_USDC),
        serde_json::json!({"match_id": "RUBY-Nexus-17/08/2024", "team": Team::Team2}).to_string(),
    )
    .await?;

    assert!(bob_bet.is_success(), "ft_transfer_call failed on Bob's first bet");

    vex_contract_balance = ft_balance_of(&usdc_contract, vex_contract.id()).await?;
    assert_eq!(vex_contract_balance, U128(115 * ONE_USDC), "Vex contract balance is not correct after Bob's first bet");
    let mut bob_balance: U128 = ft_balance_of(&usdc_contract, bob.id()).await?;
    assert_eq!(bob_balance, U128(95 * ONE_USDC), "Bob's balance is not correct after his first bet");

    bet = vex_contract.view("get_bet").args_json(serde_json::json!({"bettor": bob.id(), "bet_id": U64(2)})).await;
    assert!(bet.is_ok(), "Failed to get Bob's bet");

    // End betting 
    let complete_betting = end_betting(
        admin.clone(),
        vex_contract.id(),
        "RUBY-Nexus-17/08/2024",
    )
    .await?;

    assert!(complete_betting.is_success(), "Admin failed to end betting");

    // Cancel match
    let cancel_match_res = cancel_match(
        admin.clone(),
        vex_contract.id(),
        "RUBY-Nexus-17/08/2024",
    )
    .await?;

    assert!(cancel_match_res.is_success(), "Failed to cancel match");

    // Alice tries to claim a bet that does not exist
    let mut alice_claim = claim(
        alice.clone(),
        vex_contract.id(),
        U64(3),
    )
    .await?;

    assert!(alice_claim.is_failure(), "Alice managed to claim a bet that does not exist");

    // Bob tries to claim Alice's bet
    let mut bob_claim = claim(
        bob.clone(),
        vex_contract.id(),
        U64(1),
    )
    .await?;

    assert!(bob_claim.is_failure(), "Bob managed to claim Alice's bet");

    // Alice claims her refunded bet
    alice_claim = claim(
        alice.clone(),
        vex_contract.id(),
        U64(1),
    )
    .await?;

    assert!(alice_claim.is_success(), "Alice failed to claim her bet");

    // TODO 
    // vex_contract_balance = ft_balance_of(&usdc_contract, vex_contract.id()).await?;
    // assert_eq!(vex_contract_balance, U128( * ONE_USDC), "Vex contract balance is not correct after Bob claimed his bet");
    // bob_balance = ft_balance_of(&usdc_contract, bob.id()).await?;
    // assert_eq!(bob_balance, U128( * ONE_USDC), "Bob's balance is not correct after he claimed his bet");

    // Bob claims his refunded bet
    bob_claim = claim(
        bob.clone(),
        vex_contract.id(),
        U64(2),
    )
    .await?;

    assert!(bob_claim.is_success(), "Bob failed to claim his bet");

    // TODO 
    // vex_contract_balance = ft_balance_of(&usdc_contract, vex_contract.id()).await?;
    // assert_eq!(vex_contract_balance, U128( * ONE_USDC), "Vex contract balance is not correct after Bob claimed his bet");
    // bob_balance = ft_balance_of(&usdc_contract, bob.id()).await?;
    // assert_eq!(bob_balance, U128( * ONE_USDC), "Bob's balance is not correct after he claimed his bet");

    // Alice tries to claim her refunded bet again
    alice_claim = claim(
        alice.clone(),
        vex_contract.id(),
        U64(1),
    )
    .await?;

    assert!(alice_claim.is_failure(), "Alice managed to claim her refunded bet again");


    Ok(())
}
