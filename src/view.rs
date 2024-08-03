use near_sdk::json_types::U128;
use near_sdk::near;

use crate::bettor::determine_potential_winnings;
use crate::*;

#[allow(dead_code)]
#[near(serializers = [json])]
pub struct DisplayMatch {
    match_id: MatchId,
    game: String,
    team_1: String,
    team_2: String,
    team_1_odds: f64,
    team_2_odds: f64,
    team_1_real_bets: U128,
    team_2_real_bets: U128,
    match_state: MatchState,
    winner: Option<Team>,
}

#[near]
impl Contract {
    pub fn get_admin(&self) -> &AccountId {
        &self.admin
    }

    pub fn get_matches(&self, from_index: &Option<u32>, limit: &Option<u32>) -> Vec<DisplayMatch> {
        let from = from_index.unwrap_or(0);
        let limit = limit.unwrap_or(self.matches.len());

        // Iterates over matches. formats them, and outputs them
        self.matches
            .iter()
            .skip(from as usize)
            .take(limit as usize)
            .map(|(match_id, m)| format_match(match_id, m))
            .collect()
    }

    pub fn get_match(&self, match_id: &MatchId) -> DisplayMatch {
        // Get relevant match
        let relevant_match = self
            .matches
            .get(match_id)
            .unwrap_or_else(|| panic!("No match exists with match id: {}", match_id));

        // Return formated match
        format_match(match_id, relevant_match)
    }

    pub fn get_potential_winnings(
        &self,
        match_id: &MatchId,
        team: &Team,
        bet_amount: &U128,
    ) -> U128 {
        // Get relevant match
        let relevant_match = self
            .matches
            .get(match_id)
            .unwrap_or_else(|| panic!("No match exists with match id: {}", match_id));

        // Return potential winnings
        determine_potential_winnings(
            team,
            &relevant_match.team_1_total_bets,
            &relevant_match.team_2_total_bets,
            bet_amount,
        )
    }

    pub fn get_bet(&self, match_id: &MatchId, bet_id: &BetId) -> &Bet {
        // Get relevant match
        let relevant_match = self
            .matches
            .get(match_id)
            .unwrap_or_else(|| panic!("No match exists with match id: {}", match_id));

        // Return relevant bet
        relevant_match
            .bets
            .get(bet_id)
            .unwrap_or_else(|| panic!("No bet exists with bet id: {:?}", bet_id))
    }

    pub fn get_users_bets(
        &self,
        bettor: &AccountId,
        from_index: &Option<u32>,
        limit: &Option<u32>,
    ) -> Vec<(BetId, MatchId)> {
        // Get relevant user's bets
        let relevant_user_bets = self
            .bets_by_user
            .get(bettor)
            .unwrap_or_else(|| panic!("{} is not a bettor", bettor));

        let from = from_index.unwrap_or(0);
        let limit = limit.unwrap_or(self.matches.len());

        // Return bet IDs and their match IDs
        relevant_user_bets
            .iter()
            .skip(from as usize)
            .take(limit as usize)
            .map(|(&key, value)| (key, value.clone()))
            .collect()
    }
}

fn format_match(match_id: &MatchId, match_struct: &Match) -> DisplayMatch {
    let (team_1_odds, team_2_odds) = determine_approx_odds(
        &match_struct.team_1_total_bets,
        &match_struct.team_2_total_bets,
    );

    DisplayMatch {
        match_id: match_id.clone(), // Assuming there's a match_id field in Match
        game: match_struct.game.clone(),
        team_1: match_struct.team_1.clone(),
        team_2: match_struct.team_2.clone(),
        team_1_odds,
        team_2_odds,
        team_1_real_bets: U128(
            match_struct.team_1_total_bets.0 - match_struct.team_1_initial_pool.0,
        ),
        team_2_real_bets: U128(
            match_struct.team_2_total_bets.0 - match_struct.team_2_initial_pool.0,
        ),
        match_state: match_struct.match_state.clone(),
        winner: match_struct.winner.clone(),
    }
}

fn determine_approx_odds(team_1_total_bets: &U128, team_2_total_bets: &U128) -> (f64, f64) {
    let team_1_bets: f64 = team_1_total_bets.0 as f64;
    let team_2_bets: f64 = team_2_total_bets.0 as f64;

    // Calculate total bets
    let total_bets = team_1_bets + team_2_bets;

    // Calculate the divider to make the implied probability sum to 1.05
    let divider = total_bets / 1.05; // Market margin is set to 5%

    // Calculate implied probabilities
    let implied_prob_1 = team_1_bets / divider;
    let implied_prob_2 = team_2_bets / divider;

    // Calculate odds
    let team_1_odds = (100.0 / implied_prob_1).round() / 100.0;
    let team_2_odds = (100.0 / implied_prob_2).round() / 100.0;

    (team_1_odds, team_2_odds)
}
