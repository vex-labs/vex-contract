use near_sdk::{json_types::{U128, U64}, test_utils::test_env::alice};
use near_workspaces::rpc::query;
use vex_contracts::Team;
mod setup;
use crate::setup::*;

#[tokio::test]

async fn test_usual_flow() -> Result<(), Box<dyn std::error::Error>> {
    let TestSetup {
        alice,
        admin,
        vex_contract,
        ..
    } = setup::TestSetup::new().await?;

    // Non admin tries to create a match
    let create_match = alice
        .call(vex_contract.id(), "create_match")
        .args_json(serde_json::json!({"game": "CSGO", "team_1": "RUBY", "team_2": "Nexus", "in_odds_1": 1.2, "in_odds_2": 1.6, "date": "17/08/2024"}))
        .transact()
        .await?;

    assert!(create_match.is_failure(), "Non admin was able to create a match");

    // Admin creates a match
    let create_match = admin
        .call(vex_contract.id(), "create_match")
        .args_json(serde_json::json!({"game": "CSGO", "team_1": "RUBY", "team_2": "Nexus", "in_odds_1": 1.2, "in_odds_2": 1.6, "date": "17/08/2024"}))
        .transact()
        .await?;

    assert!(create_match.is_success(), "Admin failed to create a match");

    // Non admin tries to end betting
    let mut complete_betting = end_betting(
        alice.clone(),
        vex_contract.id(),
        "RUBY-Nexus-17/08/2024",
    )
    .await?;

    assert!(complete_betting.is_failure(), "Non admin was able to end betting");

    // Non admin tries to cancel match
    let cancel_match_res = cancel_match(
        alice.clone(),
        vex_contract.id(),
        "RUBY-Nexus-17/08/2024",
    )
    .await?;

    assert!(cancel_match_res.is_failure(), "Non admin was able to cancel match");

    // Admin ends betting
    complete_betting = end_betting(
        admin.clone(),
        vex_contract.id(),
        "RUBY-Nexus-17/08/2024",
    )
    .await?;

    assert!(complete_betting.is_success(), "Admin failed to end betting");

    // Non admin tries to finish match
    let finish_match_res = finish_match(
        alice.clone(),
        vex_contract.id(),
        "RUBY-Nexus-17/08/2024",
        Team::Team1,
    )
    .await?;

    assert!(finish_match_res.is_failure(), "Non admin was able to finish match");

    // Non admin tries to cancel match
    let cancel_match_res = cancel_match(
        alice.clone(),
        vex_contract.id(),
        "RUBY-Nexus-17/08/2024",
    )
    .await?;

    assert!(cancel_match_res.is_failure(), "Non admin was able to cancel match");

    // Non admin tries to change admin
    let mut change_admin_res = change_admin(
        alice.clone(),
        vex_contract.id(),
        alice.id().clone(),
    )
    .await?;

    assert!(change_admin_res.is_failure(), "Non admin was able to change admin");

    // Admin changes admin
    change_admin_res = change_admin(
        admin.clone(),
        vex_contract.id(),
        alice.id().clone(),
    )
    .await?;

    assert!(change_admin_res.is_success(), "Admin failed to change admin");

    // Old admin tries to create a match
    let create_match = admin
        .call(vex_contract.id(), "create_match")
        .args_json(serde_json::json!({"game": "CSGO", "team_1": "RUBY", "team_2": "Nexus", "in_odds_1": 1.2, "in_odds_2": 1.6, "date": "17/08/2024"}))
        .transact()
        .await?;

    assert!(create_match.is_failure(), "Old admin was able to create a match");

    // New admin creates a match
    let create_match = alice
        .call(vex_contract.id(), "create_match")
        .args_json(serde_json::json!({"game": "CSGO", "team_1": "RUBY", "team_2": "Nexus", "in_odds_1": 1.2, "in_odds_2": 1.6, "date": "17/08/2024"}))
        .transact()
        .await?;

    assert!(create_match.is_success(), "New admin failed to create a match");

    Ok(())
}