use near_sdk::json_types::U128;
use near_sdk::near;

use crate::betting::bettor::determine_potential_winnings;
use crate::*;

#[near(serializers = [json])]
pub struct ContractInfo {
    admin: AccountId,
    usdc_token_contract: AccountId,
    vex_token_contract: AccountId,
    treasury: AccountId,
    ref_contract: AccountId,
    ref_pool_id: U64,
}

#[near(serializers = [json])]
pub struct DisplayMatch {
    pub match_id: MatchId,
    pub game: String,
    pub team_1: String,
    pub team_2: String,
    pub team_1_odds: f64,
    pub team_2_odds: f64,
    pub team_1_real_bets: U128,
    pub team_2_real_bets: U128,
    pub match_state: MatchState,
    pub winner: Option<Team>,
}

#[near]
impl Contract {
    // Get general contract info
    pub fn get_contract_info(&self) -> ContractInfo {
        ContractInfo {
            admin: self.admin.clone(),
            usdc_token_contract: self.usdc_token_contract.clone(),
            vex_token_contract: self.vex_token_contract.clone(),
            treasury: self.treasury.clone(),
            ref_contract: self.ref_contract.clone(),
            ref_pool_id: U64(self.ref_pool_id),
        }
    }

    // Returns a list of matches wihtin a range
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

    // Returns a specific match by its ID
    pub fn get_match(&self, match_id: &MatchId) -> DisplayMatch {
        // Get relevant match
        let relevant_match = self
            .matches
            .get(match_id)
            .unwrap_or_else(|| panic!("No match exists with match id: {}", match_id));

        // Return formated match
        format_match(match_id, relevant_match)
    }

    // Returns the potential winnings you would get if you bet a certain
    // amount on a certain match on a certain team
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

    // Returns a specific bet by its user and ID
    pub fn get_bet(&self, bettor: &AccountId, bet_id: &BetId) -> &Bet {
        // Get relevant user
        let relevant_user = self
            .bets_by_user
            .get(bettor)
            .unwrap_or_else(|| panic!("No user exists with Account ID: {:?}", bettor));

        // Return relevant bet
        relevant_user
            .get(bet_id)
            .unwrap_or_else(|| panic!("No bet exists with bet id: {:?}", bet_id))
    }

    // Returns a list of bets made by a user within a range
    pub fn get_users_bets(
        &self,
        bettor: &AccountId,
        from_index: &Option<u32>,
        limit: &Option<u32>,
    ) -> Vec<(BetId, &Bet)> {
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
            .map(|(&key, value)| (key, value))
            .collect()
    }

    // Get funds to pay out to winners
    pub fn get_funds_to_payout(&self) -> U128 {
        self.funds_to_payout
    }
}

// Helper function to format a match to be displayed
pub fn format_match(match_id: &MatchId, match_struct: &Match) -> DisplayMatch {
    let (team_1_odds, team_2_odds) = determine_approx_odds(
        &match_struct.team_1_total_bets,
        &match_struct.team_2_total_bets,
    );

    DisplayMatch {
        match_id: match_id.clone(),
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

// Helper function to determine approximate odds, odds for an infitesimal bet
pub fn determine_approx_odds(team_1_total_bets: &U128, team_2_total_bets: &U128) -> (f64, f64) {
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
    let team_1_odds = 1.0 / implied_prob_1;
    let team_2_odds = 1.0 / implied_prob_2;

    (team_1_odds, team_2_odds)
}
