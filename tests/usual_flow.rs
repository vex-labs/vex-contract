use near_sdk::json_types::{U128, U64};
use vex_contracts::betting::view_betting::DisplayMatch;
use vex_contracts::Team;
mod setup;
use crate::setup::*;

#[tokio::test]

async fn test_usual_flow() -> Result<(), Box<dyn std::error::Error>> {
    let TestSetup {
        alice,
        bob,
        admin,
        main_contract,
        usdc_token_contract,
        ..
    } = setup::TestSetup::new(false).await?;

    // Create a new match
    let mut result = admin
        .call(main_contract.id(), "create_match")
        .args_json(serde_json::json!({"game": "CSGO", "team_1": "RUBY", "team_2": "Nexus", "in_odds_1": 1.2, "in_odds_2": 1.6, "date": "17/08/2024"}))
        .transact()
        .await?;

    assert!(result.is_success(), "Admin failed to create a match");

    let mut match_view: DisplayMatch = main_contract
        .view("get_match")
        .args_json(serde_json::json!({"match_id": "RUBY-Nexus-17/08/2024"}))
        .await?
        .json()?;
    assert_eq!(
        (match_view.team_1_odds * 100.0).round() / 100.0,
        1.67,
        "Team 1 odds are incorrect after match is created"
    );
    assert_eq!(
        (match_view.team_2_odds * 100.0).round() / 100.0,
        2.22,
        "Team 2 odds are incorrect after match is created"
    );

    // Alice places a bet of 10 USDC on the winning team
    result = ft_transfer_call(
        alice.clone(),
        usdc_token_contract.id(),
        main_contract.id(),
        U128(10 * ONE_USDC),
        serde_json::json!({"Bet" : {"match_id": "RUBY-Nexus-17/08/2024", "team": Team::Team1}})
            .to_string(),
    )
    .await?;

    assert!(
        result.is_success(),
        "ft_transfer_call failed on Alice's first bet"
    );

    let mut balance: U128 = ft_balance_of(&usdc_token_contract, main_contract.id()).await?;
    assert_eq!(
        balance,
        U128(110 * ONE_USDC),
        "Vex contract balance is not correct after Alice's first bet"
    );
    balance = ft_balance_of(&usdc_token_contract, alice.id()).await?;
    assert_eq!(
        balance,
        U128(90 * ONE_USDC),
        "Alice's balance is not correct after her first bet"
    );

    let mut bet = main_contract
        .view("get_bet")
        .args_json(serde_json::json!({"bettor": alice.id(), "bet_id": U64(1)}))
        .await;
    assert!(bet.is_ok(), "Failed to get Alice's bet");

    match_view = main_contract
        .view("get_match")
        .args_json(serde_json::json!({"match_id": "RUBY-Nexus-17/08/2024"}))
        .await?
        .json()?;
    assert_eq!(
        (match_view.team_1_odds * 100.0).round() / 100.0,
        1.66,
        "Team 1 odds are incorrect after match is created"
    );
    assert_eq!(
        (match_view.team_2_odds * 100.0).round() / 100.0,
        2.24,
        "Team 2 odds are incorrect after match is created"
    );

    // Bob places a bet of 5 USDC on the losing team
    result = ft_transfer_call(
        bob.clone(),
        usdc_token_contract.id(),
        main_contract.id(),
        U128(5 * ONE_USDC),
        serde_json::json!({"Bet" : {"match_id": "RUBY-Nexus-17/08/2024", "team": Team::Team2}})
            .to_string(),
    )
    .await?;

    assert!(
        result.is_success(),
        "ft_transfer_call failed on Bob's first bet"
    );

    balance = ft_balance_of(&usdc_token_contract, main_contract.id()).await?;
    assert_eq!(
        balance,
        U128(115 * ONE_USDC),
        "Vex contract balance is not correct after Bob's first bet"
    );
    balance = ft_balance_of(&usdc_token_contract, bob.id()).await?;
    assert_eq!(
        balance,
        U128(95 * ONE_USDC),
        "Bob's balance is not correct after his first bet"
    );

    bet = main_contract
        .view("get_bet")
        .args_json(serde_json::json!({"bettor": bob.id(), "bet_id": U64(2)}))
        .await;
    assert!(bet.is_ok(), "Failed to get Bob's first bet");

    match_view = main_contract
        .view("get_match")
        .args_json(serde_json::json!({"match_id": "RUBY-Nexus-17/08/2024"}))
        .await?
        .json()?;
    assert_eq!(
        (match_view.team_1_odds * 100.0).round() / 100.0,
        1.66,
        "Team 1 odds are incorrect after match is created"
    );
    assert_eq!(
        (match_view.team_2_odds * 100.0).round() / 100.0,
        2.23,
        "Team 2 odds are incorrect after match is created"
    );

    // Bob places a bet of 2 USDC on the winning team
    result = ft_transfer_call(
        bob.clone(),
        usdc_token_contract.id(),
        main_contract.id(),
        U128(2 * ONE_USDC),
        serde_json::json!({"Bet" : {"match_id": "RUBY-Nexus-17/08/2024", "team": Team::Team1}})
            .to_string(),
    )
    .await?;

    assert!(
        result.is_success(),
        "ft_transfer_call failed on Bob's second bet"
    );

    balance = ft_balance_of(&usdc_token_contract, main_contract.id()).await?;
    assert_eq!(
        balance,
        U128(117 * ONE_USDC),
        "Vex contract balance is not correct after Bob's second bet"
    );
    balance = ft_balance_of(&usdc_token_contract, bob.id()).await?;
    assert_eq!(
        balance,
        U128(93 * ONE_USDC),
        "Bob's balance is not correct after his second bet"
    );

    bet = main_contract
        .view("get_bet")
        .args_json(serde_json::json!({"bettor": bob.id(), "bet_id": U64(3)}))
        .await;
    assert!(bet.is_ok(), "Failed to get Bob's second bet");

    // Alice places a bet on an invalid match
    result = ft_transfer_call(
        alice.clone(),
        usdc_token_contract.id(),
        main_contract.id(),
        U128(10 * ONE_USDC),
        serde_json::json!({"Bet" : {"match_id": "Furia-Nexus-17/08/2024", "team": Team::Team1}})
            .to_string(),
    )
    .await?;

    assert!(
        result.is_success(),
        "ft_transfer_call failed on Alice's invalid match bet"
    );

    balance = ft_balance_of(&usdc_token_contract, main_contract.id()).await?;
    assert_eq!(
        balance,
        U128(117 * ONE_USDC),
        "Vex contract balance is not correct after Alice's invalid match bet"
    );
    balance = ft_balance_of(&usdc_token_contract, alice.id()).await?;
    assert_eq!(
        balance,
        U128(90 * ONE_USDC),
        "Alice's balance is not correct after her invalid match bet"
    );

    bet = main_contract
        .view("get_bet")
        .args_json(serde_json::json!({"bettor": alice.id(), "bet_id": U64(4)}))
        .await;
    assert!(
        bet.is_err(),
        "Wrongly managed to get Alice's invalid match bet"
    );

    // Alice tries to make a bet less than 1 USDC
    result = ft_transfer_call(
        alice.clone(),
        usdc_token_contract.id(),
        main_contract.id(),
        U128((0.5 * ONE_USDC as f64) as u128),
        serde_json::json!({"Bet" : {"match_id": "RUBY-Nexus-17/08/2024", "team": Team::Team1}})
            .to_string(),
    )
    .await?;

    assert!(
        result.is_success(),
        "ft_transfer_call failed on Alice's bet less than 1 USDC"
    );

    balance = ft_balance_of(&usdc_token_contract, main_contract.id()).await?;
    assert_eq!(
        balance,
        U128(117 * ONE_USDC),
        "Vex contract balance is not correct after Alice's bet less than 1 USDC"
    );
    balance = ft_balance_of(&usdc_token_contract, alice.id()).await?;
    assert_eq!(
        balance,
        U128(90 * ONE_USDC),
        "Alice's balance is not correct after her bet less than 1 USDC"
    );

    bet = main_contract
        .view("get_bet")
        .args_json(serde_json::json!({"bettor": alice.id(), "bet_id": U64(4)}))
        .await;
    assert!(
        bet.is_err(),
        "Wrongly managed to get Alice's bet less than 1 USDC"
    );

    // Alice tries to claim her funds before the match ends
    result = claim(alice.clone(), main_contract.id(), U64(1)).await?;

    assert!(
        result.is_failure(),
        "Alice managed to claim her funds before betting ended"
    );

    // Finish match attempted
    result = finish_match(
        admin.clone(),
        main_contract.id(),
        "RUBY-Nexus-17/08/2024",
        Team::Team1,
    )
    .await?;

    assert!(
        result.is_failure(),
        "Admin managed to finish the match in Future stage"
    );

    // End betting
    result = end_betting(admin.clone(), main_contract.id(), "RUBY-Nexus-17/08/2024").await?;

    assert!(result.is_success(), "Admin failed to end betting");

    // Alice tries to place a bet after betting ended
    result = ft_transfer_call(
        alice.clone(),
        usdc_token_contract.id(),
        main_contract.id(),
        U128(10 * ONE_USDC),
        serde_json::json!({"Bet" : {"match_id": "RUBY-Nexus-17/08/2024", "team": Team::Team1}})
            .to_string(),
    )
    .await?;

    assert!(
        result.is_success(),
        "ft_transfer_call failed on Alice's bet after betting ended"
    );

    balance = ft_balance_of(&usdc_token_contract, main_contract.id()).await?;
    assert_eq!(
        balance,
        U128(117 * ONE_USDC),
        "Vex contract balance is not correct after Alice's bet after betting ended"
    );
    balance = ft_balance_of(&usdc_token_contract, alice.id()).await?;
    assert_eq!(
        balance,
        U128(90 * ONE_USDC),
        "Alice's balance is not correct after her bet after betting ended"
    );

    bet = main_contract
        .view("get_bet")
        .args_json(serde_json::json!({"bettor": alice.id(), "bet_id": U64(4)}))
        .await;
    assert!(
        bet.is_err(),
        "Wrongly managed to get Alice's invalid match bet"
    );

    // Alice tries to claim her funds before the match is finished
    result = claim(alice.clone(), main_contract.id(), U64(1)).await?;

    assert!(
        result.is_failure(),
        "Alice managed to claim her funds before the match was finished"
    );

    // Finish match
    result = finish_match(
        admin.clone(),
        main_contract.id(),
        "RUBY-Nexus-17/08/2024",
        Team::Team1,
    )
    .await?;

    assert!(result.is_success(), "Admin failed to finish the match");

    // Admin tries to end betting after match is finished
    result = end_betting(admin.clone(), main_contract.id(), "RUBY-Nexus-17/08/2024").await?;

    assert!(
        result.is_failure(),
        "Admin was able to end betting after match was finished"
    );

    // Admin tries to cancel match after match is finished
    result = cancel_match(admin.clone(), main_contract.id(), "RUBY-Nexus-17/08/2024").await?;

    assert!(
        result.is_failure(),
        "Admin was able to cancel match after match was finished"
    );

    // Alice tries to place a bet after the match is finished
    result = ft_transfer_call(
        alice.clone(),
        usdc_token_contract.id(),
        main_contract.id(),
        U128(10 * ONE_USDC),
        serde_json::json!({"Bet" : {"match_id": "RUBY-Nexus-17/08/2024", "team": Team::Team1}})
            .to_string(),
    )
    .await?;

    assert!(
        result.is_success(),
        "ft_transfer_call failed on Alice's bet after the match was finished"
    );

    balance = ft_balance_of(&usdc_token_contract, main_contract.id()).await?;
    assert_eq!(
        balance,
        U128(117 * ONE_USDC),
        "Vex contract balance is not correct after Alice's bet after the match was finished"
    );
    balance = ft_balance_of(&usdc_token_contract, alice.id()).await?;
    assert_eq!(
        balance,
        U128(90 * ONE_USDC),
        "Alice's balance is not correct after her bet after the match was finished"
    );

    bet = main_contract
        .view("get_bet")
        .args_json(serde_json::json!({"bettor": alice.id(), "bet_id": U64(4)}))
        .await;
    assert!(
        bet.is_err(),
        "Wrongly managed to get Alice's invalid match bet"
    );

    // Bob tries to claim Alice's bet
    result = claim(bob.clone(), main_contract.id(), U64(1)).await?;

    assert!(result.is_failure(), "Bob managed to claim Alice's bet");

    // Bob tries to claim the bet he lost
    result = claim(bob.clone(), main_contract.id(), U64(2)).await?;

    assert!(result.is_failure(), "Bob managed to claim a bet he lost");

    // Alice tries to claim a bet that does not exist
    result = claim(alice.clone(), main_contract.id(), U64(3)).await?;

    assert!(
        result.is_failure(),
        "Alice managed to claim a bet that does not exist"
    );

    // Alice claims the bet he won
    result = claim(alice.clone(), main_contract.id(), U64(1)).await?;

    assert!(result.is_success(), "Alice failed to claim her bet");

    let mut winnings: u128 = 16617241;
    let new_contract_bal = 117 * ONE_USDC - winnings;
    balance = ft_balance_of(&usdc_token_contract, main_contract.id()).await?;
    assert_eq!(
        balance,
        U128(new_contract_bal),
        "Vex contract balance is not correct after Alice claimed her bet"
    );
    balance = ft_balance_of(&usdc_token_contract, alice.id()).await?;
    assert_eq!(
        balance,
        U128(90 * ONE_USDC + winnings),
        "Alice's balance is not correct after she claimed her bet"
    );

    // Bob claims the bet he won
    result = claim(bob.clone(), main_contract.id(), U64(3)).await?;

    assert!(result.is_success(), "Bob failed to claim his bet");

    winnings = 3325152;
    balance = ft_balance_of(&usdc_token_contract, main_contract.id()).await?;
    assert_eq!(
        balance,
        U128(new_contract_bal - winnings),
        "Vex contract balance is not correct after Bob claimed his bet"
    );
    balance = ft_balance_of(&usdc_token_contract, bob.id()).await?;
    assert_eq!(
        balance,
        U128(93 * ONE_USDC + winnings),
        "Bob's balance is not correct after he claimed his bet"
    );

    // Alice tries to claim her bet for a second time
    result = claim(alice.clone(), main_contract.id(), U64(1)).await?;

    assert!(
        result.is_failure(),
        "Alice managed to claim her bet for a second time"
    );

    Ok(())
}
