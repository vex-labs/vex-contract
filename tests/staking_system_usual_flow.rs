// TODO: Make these tests acutually good and comprehensive, calculate and check exact or at least rounded balances
// Maybe merge with other tests

use near_sdk::json_types::{U128, U64};
use vex_contracts::{MatchStakeInfo, Team};
use vex_contracts::ft_on_transfer::FtTransferAction;
mod setup;
use crate::setup::*;

#[tokio::test]

async fn test_staking_system_usual_flow() -> Result<(), Box<dyn std::error::Error>> {
    let TestSetup {
        alice,
        bob,
        admin,
        main_contract,
        usdc_token_contract,
        vex_token_contract,
        sandbox
    } = setup::TestSetup::new(false).await?;

    // Add 25 USDC to the insurance pool
    let mut result = ft_transfer_call(
        admin.clone(),
        usdc_token_contract.id(),
        main_contract.id(),
        U128(25 * ONE_USDC),
        serde_json::json!(FtTransferAction::AddUSDC).to_string(),
    )
    .await?;

    assert!(
        result.is_success(),
        "ft_transfer_call failed on adding to insurance pool"
    );

    let mut balance: U128 = ft_balance_of(&usdc_token_contract, main_contract.id()).await?;
    assert_eq!(
        balance,
        U128(125 * ONE_USDC),
        "Vex contract balance is not correct after adding to insurance pool first bet"
    );

    // Check insurance pool
    let insurance_pool: U128 = main_contract.view("get_insurance_fund").await?.json()?;
    assert_eq!(
        insurance_pool,
        U128(25 * ONE_USDC),
        "Insurance pool is not correct after adding to insurance pool"
    );

    // Alice stakes 50 $VEX
    result = ft_transfer_call(
        alice.clone(),
        vex_token_contract.id(),
        main_contract.id(),
        U128(50 * ONE_VEX),
        serde_json::json!(FtTransferAction::Stake).to_string(),
    )
    .await?;

    assert!(
        result.is_success(),
        "ft_transfer_call failed on Alice's first stake"
    );

    balance = ft_balance_of(&vex_token_contract, main_contract.id()).await?;
    assert_eq!(
        balance,
        U128(150 * ONE_VEX),
        "Vex contract balance is not correct after Alice's first stake"
    );
    balance = ft_balance_of(&vex_token_contract, alice.id()).await?;
    assert_eq!(
        balance,
        U128(50 * ONE_VEX),
        "Alice's balance is not correct after her first stake"
    );

    result = stake_all(alice.clone(), main_contract.id()).await?;

    assert!(
        result.is_success(),
        "stake_all failed on Alice's first stake"
    );

    // Bob stakes 100 $VEX
    result = ft_transfer_call(
        bob.clone(),
        vex_token_contract.id(),
        main_contract.id(),
        U128(100 * ONE_VEX),
        serde_json::json!(FtTransferAction::Stake).to_string(),
    )
    .await?;

    assert!(
        result.is_success(),
        "ft_transfer_call failed on Bob's first stake"
    );

    balance = ft_balance_of(&vex_token_contract, main_contract.id()).await?;
    assert_eq!(
        balance,
        U128(250 * ONE_VEX),
        "Vex contract balance is not correct after Bob's first stake"
    );
    balance = ft_balance_of(&vex_token_contract, bob.id()).await?;
    assert_eq!(
        balance,
        U128(0),
        "Bob's balance is not correct after his first stake"
    );

    result = stake_all(bob.clone(), main_contract.id()).await?;

    assert!(
        result.is_success(),
        "stake_all failed on Bob's first stake stake"
    );

    // Check total staked amount
    let total_staked_balance: U128 = main_contract.view("get_total_staked_balance").await?.json()?;
    assert_eq!(
        total_staked_balance,
        U128(249 * ONE_VEX),
        "Total staked amount is not correct after Alice and Bob's first stakes"
    );

    // Create a new match
    result = admin
        .call(main_contract.id(), "create_match")
        .args_json(serde_json::json!({"game": "CSGO", "team_1": "RUBY", "team_2": "Nexus", "in_odds_1": 1.2, "in_odds_2": 1.6, "date": "17/08/2024"}))
        .transact()
        .await?;

    assert!(result.is_success(), "Admin failed to create a match");

    // Alice places a bet of 50 USDC on the losing team
    result = ft_transfer_call(
        alice.clone(),
        usdc_token_contract.id(),
        main_contract.id(),
        U128(50 * ONE_USDC),
        serde_json::json!({"Bet" : {"match_id": "RUBY-Nexus-17/08/2024", "team": Team::Team2}}).to_string(),
    )
    .await?;

    assert!(
        result.is_success(),
        "ft_transfer_call failed on Alice's first bet"
    );

    balance = ft_balance_of(&usdc_token_contract, main_contract.id()).await?;
    assert_eq!(
        balance,
        U128(175 * ONE_USDC),
        "Vex contract balance is not correct after Alice's first bet"
    );

    // Bob places a bet of 10 USDC on the winning team
    result = ft_transfer_call(
        bob.clone(),
        usdc_token_contract.id(),
        main_contract.id(),
        U128(10 * ONE_USDC),
        serde_json::json!({"Bet" : {"match_id": "RUBY-Nexus-17/08/2024", "team": Team::Team1}}).to_string(),
    )
    .await?;

    assert!(
        result.is_success(),
        "ft_transfer_call failed on Bob's first bet"
    );

    balance = ft_balance_of(&usdc_token_contract, main_contract.id()).await?;
    assert_eq!(
        balance,
        U128(185 * ONE_USDC),
        "Vex contract balance is not correct after Bob's first bet"
    );

    // End betting
    result = end_betting(admin.clone(), main_contract.id(), "RUBY-Nexus-17/08/2024").await?;

    assert!(result.is_success(), "Admin failed to end betting");

    // Treasury balance before
    let balance_before = ft_balance_of(&usdc_token_contract, admin.id()).await?;

    // Finish match
    result = finish_match(
        admin.clone(),
        main_contract.id(),
        "RUBY-Nexus-17/08/2024",
        Team::Team1,
    )
    .await?;

    assert!(result.is_success(), "Admin failed to finish the match");

    // Check that the insurance fund balance has increased
    let insurance_pool: U128 = main_contract.view("get_insurance_fund").await?.json()?;
    assert!(
        insurance_pool.0 > 25 * ONE_USDC,
        "Insurance pool is not correct after the first match has finished"
    );
    println!("Insurance pool increased by : {}", insurance_pool.0 - 25 * ONE_USDC);

    // Check that the fees fund balance has increased
    let fees_fund: U128 = main_contract.view("get_fees_fund").await?.json()?;
    assert!(
        fees_fund.0 > 
        2 * ONE_USDC,
        "Fees fund is not correct after the first match has finished"
    );
    println!("Fees fund: {}", fees_fund.0);
    
    // Check that the treasury balance has increased
    balance = ft_balance_of(&usdc_token_contract, admin.id()).await?;
    assert!(
        balance.0 > balance_before.0,
        "Treasury balance is not correct after the first match has finished"
    );
    println!("Treasury balance increased by: {}", balance.0 - balance_before.0);

    // Check rewards have been added to staking rewards queue
    let staking_rewards_queue: Vec<MatchStakeInfo> = main_contract.view("get_staking_rewards_queue").await?.json()?;
    assert_eq!(
        staking_rewards_queue.len(),
        1,
        "Staking rewards queue is not correct after the first match has finished"
    );
    println!("Staking rewards end time: {}", staking_rewards_queue[0].stake_end_time.0);
    println!("Staking rewards amount: {}", staking_rewards_queue[0].staking_rewards.0);

    // Advance some amount of blocks less than the staking rewards end time
    sandbox.fast_forward(50).await?;

    // Save the total staked balance before the swap
    let total_staked_balance_before: U128 = main_contract.view("get_total_staked_balance").await?.json()?;

    // Save the balance of swap caller before the swap
    let balance_before = ft_balance_of(&usdc_token_contract, alice.id()).await?;

    // Call peform_stake_swap
    result = perform_stake_swap(alice.clone(), main_contract.id()).await?;

    dbg!(&result);

    assert!(
        result.is_success(),
        "perform_stake_swap failed"
    );

    // Check that the rewards are still in the queue 
    let staking_rewards_queue: Vec<MatchStakeInfo> = main_contract.view("get_staking_rewards_queue").await?.json()?;
    assert_eq!(
        staking_rewards_queue.len(),
        1,
        "Staking rewards queue is not correct after fast forwarding"
    );

    // Check that the total staked balance has increased
    let total_staked_balance: U128 = main_contract.view("get_total_staked_balance").await?.json()?;
    assert!(
        total_staked_balance.0 > total_staked_balance_before.0,
        "Total staked amount is not correct after fast forwarding"
    );
    println!("Total staked balance increased by: {}", total_staked_balance.0 - total_staked_balance_before.0);

    // Check that the balance of the swap caller has increased
    let balance = ft_balance_of(&vex_token_contract, alice.id()).await?;
    assert!(
        balance.0 > balance_before.0,
        "Balance of the swap caller is not correct after fast forwarding"
    );
    println!("Balance of the swap caller increased by: {}", balance.0 - balance_before.0);

    

    // // Alice places a bet of 20 USDC on the winning team
    // result = ft_transfer_call(
    //     alice.clone(),
    //     usdc_token_contract.id(),
    //     main_contract.id(),
    //     U128(20 * ONE_USDC),
    //     serde_json::json!({"Bet" : {"match_id": "RUBY-Nexus-17/08/2024", "team": Team::Team1}}).to_string(),
    // )
    // .await?;

    // assert!(
    //     result.is_success(),
    //     "ft_transfer_call failed on Alice's first bet"
    // );

    // balance = ft_balance_of(&usdc_token_contract, main_contract.id()).await?;
    // assert_eq!(
    //     balance,
    //     U128(175 * ONE_USDC),
    //     "Vex contract balance is not correct after Alice's first bet"
    // );

    Ok(())
}
