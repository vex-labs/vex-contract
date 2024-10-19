use vex_contracts::Team;
mod setup;
use crate::setup::*;

#[tokio::test]

async fn test_admin_methods() -> Result<(), Box<dyn std::error::Error>> {
    let TestSetup {
        alice,
        admin,
        vex_token_contract,
        ..
    } = setup::TestSetup::new(false).await?;

    // Non admin tries to create a match
    let mut result = alice
        .call(vex_token_contract.id(), "create_match")
        .args_json(serde_json::json!({"game": "CSGO", "team_1": "RUBY", "team_2": "Nexus", "in_odds_1": 1.2, "in_odds_2": 1.6, "date": "17/08/2024"}))
        .transact()
        .await?;

    assert!(result.is_failure(), "Non admin was able to create a match");

    // Admin creates a match
    result = admin
        .call(vex_token_contract.id(), "create_match")
        .args_json(serde_json::json!({"game": "CSGO", "team_1": "RUBY", "team_2": "Nexus", "in_odds_1": 1.2, "in_odds_2": 1.6, "date": "17/08/2024"}))
        .transact()
        .await?;

    assert!(result.is_success(), "Admin failed to create a match");

    // Non admin tries to end betting
    result = end_betting(alice.clone(), vex_token_contract.id(), "RUBY-Nexus-17/08/2024").await?;

    assert!(result.is_failure(), "Non admin was able to end betting");

    // Non admin tries to cancel match
    result = cancel_match(alice.clone(), vex_token_contract.id(), "RUBY-Nexus-17/08/2024").await?;

    assert!(result.is_failure(), "Non admin was able to cancel match");

    // Admin ends betting
    result = end_betting(admin.clone(), vex_token_contract.id(), "RUBY-Nexus-17/08/2024").await?;

    assert!(result.is_success(), "Admin failed to end betting");

    // Non admin tries to finish match
    result = finish_match(
        alice.clone(),
        vex_token_contract.id(),
        "RUBY-Nexus-17/08/2024",
        Team::Team1,
    )
    .await?;

    assert!(result.is_failure(), "Non admin was able to finish match");

    // Non admin tries to cancel match
    result = cancel_match(alice.clone(), vex_token_contract.id(), "RUBY-Nexus-17/08/2024").await?;

    assert!(result.is_failure(), "Non admin was able to cancel match");

    // Non admin tries to change admin
    result = change_admin(alice.clone(), vex_token_contract.id(), alice.id().clone()).await?;

    assert!(result.is_failure(), "Non admin was able to change admin");

    // Admin changes admin
    result = change_admin(admin.clone(), vex_token_contract.id(), alice.id().clone()).await?;

    assert!(result.is_success(), "Admin failed to change admin");

    // Old admin tries to create a match
    result = admin
        .call(vex_token_contract.id(), "create_match")
        .args_json(serde_json::json!({"game": "CSGO", "team_1": "RUBY", "team_2": "Nexus", "in_odds_1": 1.2, "in_odds_2": 1.6, "date": "17/08/2024"}))
        .transact()
        .await?;

    assert!(result.is_failure(), "Old admin was able to create a match");

    // New admin creates a match
    result = alice
        .call(vex_token_contract.id(), "create_match")
        .args_json(serde_json::json!({"game": "CSGO", "team_1": "RUBY", "team_2": "Nexus", "in_odds_1": 1.2, "in_odds_2": 1.6, "date": "17/08/2024"}))
        .transact()
        .await?;

    assert!(result.is_success(), "New admin failed to create a match");

    Ok(())
}
