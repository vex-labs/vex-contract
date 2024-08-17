use near_sdk::json_types::{U128, U64};
use vex_contracts::Team;
mod setup;
use crate::setup::*;

#[tokio::test]

async fn test_usual_flow() -> Result<(), Box<dyn std::error::Error>> {
    let TestSetup {
        sandbox,
        alice,
        bob,
        admin,
        vex_contract,
        usdc_contract,
    } = setup::TestSetup::new().await?;

    // Create a new match
    let create_match = admin
        .call(vex_contract.id(), "create_match")
        .args_json(serde_json::json!({"game": "CSGO", "team_1": "RUBY", "team_2": "Nexus", "in_odds_1": 1.2, "in_odds_2": 1.6, "date": "17/08/2024"}))
        .transact()
        .await?;

    assert!(create_match.is_success());

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

    bet = vex_contract.view("get_bet").args_json(serde_json::json!({"bettor": alice.id(), "bet_id": U64(3)})).await;
    assert!(bet.is_err(), "Wrongly managed to get Alice's invalid match bet");

    // end betting 
    

    // users try to place bets

    // finish match

    // users try to place bets

    // users try to claim funds for each others

    // user claims funds

    // user tries to claim funds again

    //

    Ok(())
}





// for error checking 

// if alice_bet.is_success() {
//     println!("Bet was successful!");
// } else {
//     // Directly iterate over the failures vector
//     for failure in alice_bet.failures() {
//         println!("Failure: {:?}", failure);
//     }
//     panic!("Alice's bet failed.");
// }
