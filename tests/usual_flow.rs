use near_sdk::json_types::{U128, U64};
use vex_contracts::Team;
mod setup;
use crate::setup::*;

#[tokio::test]

async fn test_usual_flow() -> Result<(), Box<dyn std::error::Error>> {
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

    // Alice places a bet of 10 USDC on the winning team
    let mut alice_bet = ft_transfer_call(
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

    // TODO Check odds

    // Bob places a bet of 5 USDC on the losing team
    let mut bob_bet = ft_transfer_call(
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
    assert!(bet.is_ok(), "Failed to get Bob's first bet");
    
    // TODO Check odds

    // Bob places a bet of 2 USDC on the winning team
    bob_bet = ft_transfer_call(
        bob.clone(),
        usdc_contract.id(),
        vex_contract.id(),
        U128(2 * ONE_USDC),
        serde_json::json!({"match_id": "RUBY-Nexus-17/08/2024", "team": Team::Team1}).to_string(),
    )
    .await?;

    assert!(bob_bet.is_success(), "ft_transfer_call failed on Bob's second bet");

    vex_contract_balance = ft_balance_of(&usdc_contract, vex_contract.id()).await?;
    assert_eq!(vex_contract_balance, U128(117 * ONE_USDC), "Vex contract balance is not correct after Bob's second bet");
    bob_balance = ft_balance_of(&usdc_contract, bob.id()).await?;
    assert_eq!(bob_balance, U128(93 * ONE_USDC), "Bob's balance is not correct after his second bet");

    bet = vex_contract.view("get_bet").args_json(serde_json::json!({"bettor": bob.id(), "bet_id": U64(3)})).await;
    assert!(bet.is_ok(), "Failed to get Bob's second bet");

    // TODO Check odds

    // Alice places a bet on an invalid match
    alice_bet = ft_transfer_call(
        alice.clone(),
        usdc_contract.id(),
        vex_contract.id(),
        U128(10 * ONE_USDC),
        serde_json::json!({"match_id": "Furia-Nexus-17/08/2024", "team": Team::Team1}).to_string(),
    )
    .await?;

    assert!(alice_bet.is_success(), "ft_transfer_call failed on Alice's invalid match bet");

    vex_contract_balance = ft_balance_of(&usdc_contract, vex_contract.id()).await?;
    assert_eq!(vex_contract_balance, U128(117 * ONE_USDC), "Vex contract balance is not correct after Alice's invalid match bet");
    alice_balance = ft_balance_of(&usdc_contract, alice.id()).await?;
    assert_eq!(alice_balance, U128(90 * ONE_USDC), "Alice's balance is not correct after her invalid match bet");

    bet = vex_contract.view("get_bet").args_json(serde_json::json!({"bettor": alice.id(), "bet_id": U64(4)})).await;
    assert!(bet.is_err(), "Wrongly managed to get Alice's invalid match bet");

    // Alice tries to make a bet less than 1 USDC
    alice_bet = ft_transfer_call(
        alice.clone(),
        usdc_contract.id(),
        vex_contract.id(),
        U128((0.5 * ONE_USDC as f64) as u128),
        serde_json::json!({"match_id": "RUBY-Nexus-17/08/2024", "team": Team::Team1}).to_string(),
    )
    .await?;

    assert!(alice_bet.is_success(), "ft_transfer_call failed on Alice's bet less than 1 USDC");

    vex_contract_balance = ft_balance_of(&usdc_contract, vex_contract.id()).await?;
    assert_eq!(vex_contract_balance, U128(117 * ONE_USDC), "Vex contract balance is not correct after Alice's bet less than 1 USDC");
    alice_balance = ft_balance_of(&usdc_contract, alice.id()).await?;
    assert_eq!(alice_balance, U128(90 * ONE_USDC), "Alice's balance is not correct after her bet less than 1 USDC");

    bet = vex_contract.view("get_bet").args_json(serde_json::json!({"bettor": alice.id(), "bet_id": U64(4)})).await;
    assert!(bet.is_err(), "Wrongly managed to get Alice's bet less than 1 USDC");

    // Alice tries to claim her funds before the match ends
    let mut alice_claim = claim(
        alice.clone(),
        vex_contract.id(),
        U64(1),
    )
    .await?;

    assert!(alice_claim.is_failure(), "Alice managed to claim her funds before betting ended");

    // Finish match attempted
    let mut complete_match = finish_match(
        admin.clone(),
        vex_contract.id(),
        "RUBY-Nexus-17/08/2024",
        Team::Team1,
    )
    .await?;

    assert!(complete_match.is_failure(), "Admin managed to finish the match in Future stage");

    // End betting 
    let mut complete_betting = end_betting(
        admin.clone(),
        vex_contract.id(),
        "RUBY-Nexus-17/08/2024",
    )
    .await?;

    assert!(complete_betting.is_success(), "Admin failed to end betting");

    // Alice tries to place a bet after betting ended
    alice_bet = ft_transfer_call(
        alice.clone(),
        usdc_contract.id(),
        vex_contract.id(),
        U128(10 * ONE_USDC),
        serde_json::json!({"match_id": "RUBY-Nexus-17/08/2024", "team": Team::Team1}).to_string(),
    )
    .await?;

    assert!(alice_bet.is_success(), "ft_transfer_call failed on Alice's bet after betting ended");

    vex_contract_balance = ft_balance_of(&usdc_contract, vex_contract.id()).await?;
    assert_eq!(vex_contract_balance, U128(117 * ONE_USDC), "Vex contract balance is not correct after Alice's bet after betting ended");
    alice_balance = ft_balance_of(&usdc_contract, alice.id()).await?;
    assert_eq!(alice_balance, U128(90 * ONE_USDC), "Alice's balance is not correct after her bet after betting ended");

    bet = vex_contract.view("get_bet").args_json(serde_json::json!({"bettor": alice.id(), "bet_id": U64(4)})).await;
    assert!(bet.is_err(), "Wrongly managed to get Alice's invalid match bet");

    // Alice tries to claim her funds before the match is finished
    alice_claim = claim(
        alice.clone(),
        vex_contract.id(),
        U64(1),
    )
    .await?;

    assert!(alice_claim.is_failure(), "Alice managed to claim her funds before the match was finished");

    // Finish match
    complete_match = finish_match(
        admin.clone(),
        vex_contract.id(),
        "RUBY-Nexus-17/08/2024",
        Team::Team1,
    )
    .await?;

    assert!(complete_match.is_success(), "Admin failed to finish the match");

    // Admin tries to end betting after match is finished
    complete_betting = end_betting(
        admin.clone(),
        vex_contract.id(),
        "RUBY-Nexus-17/08/2024",
    )
    .await?;

    assert!(complete_betting.is_failure(), "Admin was able to end betting after match was finished");

    // Admin tries to cancel match after match is finished
    let try_cancel_match = cancel_match(
        admin.clone(),
        vex_contract.id(),
        "RUBY-Nexus-17/08/2024",
    )
    .await?;

    assert!(try_cancel_match.is_failure(), "Admin was able to cancel match after match was finished");

    // Alice tries to place a bet after the match is finished
    alice_bet = ft_transfer_call(
        alice.clone(),
        usdc_contract.id(),
        vex_contract.id(),
        U128(10 * ONE_USDC),
        serde_json::json!({"match_id": "RUBY-Nexus-17/08/2024", "team": Team::Team1}).to_string(),
    )
    .await?;

    assert!(alice_bet.is_success(), "ft_transfer_call failed on Alice's bet after the match was finished");

    vex_contract_balance = ft_balance_of(&usdc_contract, vex_contract.id()).await?;
    assert_eq!(vex_contract_balance, U128(117 * ONE_USDC), "Vex contract balance is not correct after Alice's bet after the match was finished");
    alice_balance = ft_balance_of(&usdc_contract, alice.id()).await?;
    assert_eq!(alice_balance, U128(90 * ONE_USDC), "Alice's balance is not correct after her bet after the match was finished");

    bet = vex_contract.view("get_bet").args_json(serde_json::json!({"bettor": alice.id(), "bet_id": U64(4)})).await;
    assert!(bet.is_err(), "Wrongly managed to get Alice's invalid match bet");

    // Bob tries to claim Alice's bet
    let mut bob_claim = claim(
        bob.clone(),
        vex_contract.id(),
        U64(1),
    )
    .await?;

    assert!(bob_claim.is_failure(), "Bob managed to claim Alice's bet");

    // Bob tries to claim the bet he lost
    bob_claim = claim(
        bob.clone(),
        vex_contract.id(),
        U64(2),
    )
    .await?;

    assert!(bob_claim.is_failure(), "Bob managed to claim a bet he lost");

    // Alice tries to claim a bet that does not exist
    alice_claim = claim(
        alice.clone(),
        vex_contract.id(),
        U64(3),
    )
    .await?;

    assert!(alice_claim.is_failure(), "Alice managed to claim a bet that does not exist");

    // Alice claims the bet he won
    alice_claim = claim(
        alice.clone(),
        vex_contract.id(),
        U64(1),
    )
    .await?;

    assert!(alice_claim.is_success(), "Alice failed to claim her bet");

    // TODO 
    // vex_contract_balance = ft_balance_of(&usdc_contract, vex_contract.id()).await?;
    // assert_eq!(vex_contract_balance, U128( * ONE_USDC), "Vex contract balance is not correct after Alice claimed her bet");
    // alice_balance = ft_balance_of(&usdc_contract, alice.id()).await?;
    // assert_eq!(alice_balance, U128( * ONE_USDC), "Alice's balance is not correct after she claimed her bet");

    // Bob claims the bet he won
    bob_claim = claim(
        bob.clone(),
        vex_contract.id(),
        U64(3),
    )
    .await?;

    assert!(bob_claim.is_success(), "Bob failed to claim his bet");

    // TODO 
    // vex_contract_balance = ft_balance_of(&usdc_contract, vex_contract.id()).await?;
    // assert_eq!(vex_contract_balance, U128( * ONE_USDC), "Vex contract balance is not correct after Bob claimed his bet");
    // bob_balance = ft_balance_of(&usdc_contract, bob.id()).await?;
    // assert_eq!(bob_balance, U128( * ONE_USDC), "Bob's balance is not correct after he claimed his bet");

    // Alice tries to claim her bet for a second time
    alice_claim = claim(
        alice.clone(),
        vex_contract.id(),
        U64(1),
    )
    .await?;

    assert!(alice_claim.is_failure(), "Alice managed to claim her bet for a second time");

    Ok(())
}