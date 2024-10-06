use near_sdk::{env, near, require, Gas, NearToken};

pub use crate::ext::*;
use crate::*;

#[near]
impl Contract {
    pub fn change_admin(&mut self, new_admin: AccountId) {
        require!(
            env::predecessor_account_id() == self.admin,
            "Only the admin can call this method"
        );

        self.admin = new_admin;
    }

    pub fn create_match(
        &mut self,
        game: String,
        team_1: String,
        team_2: String,
        in_odds_1: f64,
        in_odds_2: f64,
        date: String,
    ) {
        require!(
            env::predecessor_account_id() == self.admin,
            "Only the admin can call this method"
        );

        let match_id: MatchId = format!("{}-{}-{}", team_1, team_2, date);

        let in_prob_1: f64 = 1.0 / in_odds_1;
        let in_prob_2: f64 = 1.0 / in_odds_2;
        let divider = in_prob_1 + in_prob_2;
        let actual_prob_1 = in_prob_1 / divider;
        let actual_prob_2 = in_prob_2 / divider;
        let team_1_total_bets = U128(ONE_USDC.0 * (actual_prob_1 * WEIGHT_FACTOR).round() as u128);
        let team_2_total_bets = U128(ONE_USDC.0 * (actual_prob_2 * WEIGHT_FACTOR).round() as u128);

        let match_state = MatchState::Future;
        let winner: Option<Team> = None;
        let team_1_potential_winnings = U128(0);
        let team_2_potential_winnings = U128(0);

        let new_match = Match {
            game,
            team_1,
            team_2,
            team_1_total_bets,
            team_2_total_bets,
            team_1_initial_pool: team_1_total_bets,
            team_2_initial_pool: team_2_total_bets,
            team_1_potential_winnings,
            team_2_potential_winnings,
            match_state,
            winner,
        };

        self.matches.insert(match_id, new_match);
    }

    pub fn end_betting(&mut self, match_id: &MatchId) {
        require!(
            env::predecessor_account_id() == self.admin,
            "Only the admin can call this method"
        );

        let relevant_match = self
            .matches
            .get_mut(match_id)
            .unwrap_or_else(|| panic!("No match exists with match id: {}", match_id));

        require!(
            matches!(relevant_match.match_state, MatchState::Future),
            "Match state must be Future to end betting"
        );

        relevant_match.match_state = MatchState::Current;
    }

    pub fn finish_match(&mut self, match_id: &MatchId, winner: Team) {
        require!(
            env::predecessor_account_id() == self.admin,
            "Only the admin can call this method"
        );

        let relevant_match = self
            .matches
            .get_mut(match_id)
            .unwrap_or_else(|| panic!("No match exists with match id: {}", match_id));

        require!(
            matches!(relevant_match.match_state, MatchState::Current),
            "Match state must be Current to finish the match"
        );

        relevant_match.match_state = MatchState::Finished;
        relevant_match.winner = Some(winner);
    }

    pub fn cancel_match(&mut self, match_id: &MatchId) {
        require!(
            env::predecessor_account_id() == self.admin,
            "Only the admin can call this method"
        );

        let relevant_match = self
            .matches
            .get_mut(match_id)
            .unwrap_or_else(|| panic!("No match exists with match id: {}", match_id));

        require!(
            matches!(
                relevant_match.match_state,
                MatchState::Future | MatchState::Current
            ),
            "Match state must be Future or Current to cancel the match"
        );

        relevant_match.match_state = MatchState::Error;
    }

    pub fn take_from_fees_fund(&mut self, amount: U128, receiver: AccountId) -> U128 {
        require!(
            env::predecessor_account_id() == self.admin,
            "Only the admin can call this method"
        );

        require!(
            self.fees_fund >= amount,
            "Not enough funds in the fees fund"
        );

        ft_contract::ext(self.usdc_contract.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(Gas::from_tgas(30))
            .ft_transfer(receiver, amount);

        self.fees_fund = U128(self.fees_fund.0 - amount.0);

        self.fees_fund
    }

    pub fn take_from_insurance_fund(&mut self, amount: U128, receiver: AccountId) -> U128 {
        require!(
            env::predecessor_account_id() == self.admin,
            "Only the admin can call this method"
        );

        require!(
            self.insurance_fund >= amount,
            "Not enough funds in the insurance fund"
        );

        ft_contract::ext(self.usdc_contract.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(Gas::from_tgas(30))
            .ft_transfer(receiver, amount);

        self.insurance_fund = U128(self.insurance_fund.0 - amount.0);

        self.insurance_fund
    }
}
