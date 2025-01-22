use near_sdk::json_types::U128;

use crate::{betting::bettor::determine_potential_winnings, Team};

#[test]
fn test_determine_potential_winnings() {
    determine_potential_winnings_base(
        500_000_000,
        1000_000_000, 
        100_000_000, 
        268_877_673);
}


fn determine_potential_winnings_base(team_1_total_bets: u128, team_2_total_bets: u128, bet_amount: u128, expected_winnings: u128) {
    let team_1 = Team::Team1;
    
    let team_1_total_bets = U128(team_1_total_bets);
    let team_2_total_bets = U128(team_2_total_bets);

    let bet_amount = U128(bet_amount);

    let expected_potential_winnings = U128(expected_winnings);

    let actual_potential_winnings = determine_potential_winnings(&team_1, &team_1_total_bets, &team_2_total_bets, &bet_amount);

    assert_eq!(
        expected_potential_winnings, actual_potential_winnings,
        "Winnings calculation error. Actual: {} Expected: {}", 
        actual_potential_winnings.0, 
        expected_potential_winnings.0
    );
}