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
        relevant_match.winner = Some(winner.clone());

        let total_bets = relevant_match.team_1_total_bets.0 + relevant_match.team_2_total_bets.0
            - relevant_match.team_1_initial_pool.0
            - relevant_match.team_2_initial_pool.0;

        let (difference, is_profit) = match winner {
            Team::Team1 => {
                self.funds_to_payout =
                    U128(self.funds_to_payout.0 + relevant_match.team_1_potential_winnings.0);
                if total_bets > relevant_match.team_1_potential_winnings.0 {
                    (
                        total_bets - relevant_match.team_1_potential_winnings.0,
                        true,
                    )
                } else {
                    (
                        relevant_match.team_1_potential_winnings.0 - total_bets,
                        false,
                    )
                }
            }
            Team::Team2 => {
                self.funds_to_payout =
                    U128(self.funds_to_payout.0 + relevant_match.team_2_potential_winnings.0);
                if total_bets > relevant_match.team_2_potential_winnings.0 {
                    (
                        total_bets - relevant_match.team_2_potential_winnings.0,
                        true,
                    )
                } else {
                    (
                        relevant_match.team_2_potential_winnings.0 - total_bets,
                        false,
                    )
                }
            }
        };

        match is_profit {
            true => self.handle_profit(difference),
            false => self.handle_loss(difference),
        };
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

    pub(crate) fn handle_profit(&mut self, profit: u128) {
        // Calculate how profit is distributed
        let usdc_staking_rewards =
            (U256::from(60) * U256::from(profit) / U256::from(100)).as_u128();
        let treasury_rewards = (U256::from(30) * U256::from(profit) / U256::from(100)).as_u128();
        let insurace_rewards = (U256::from(5) * U256::from(profit) / U256::from(100)).as_u128();
        let fees_rewards = profit - usdc_staking_rewards - treasury_rewards - insurace_rewards;

        // Increase fees and insurance funds
        self.fees_fund = U128(self.fees_fund.0 + fees_rewards);
        self.insurance_fund = U128(self.insurance_fund.0 + insurace_rewards);

        // Send funds to treasury
        ft_contract::ext(self.usdc_contract.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(Gas::from_tgas(30))
            .ft_transfer(self.treasury.clone(), U128(treasury_rewards));

        // Peform stake swap so rewards are distributed at the timestamp of the
        // match being added to the list so extra rewards are not distributed
        // self.perform_stake_swap();

        self.usdc_staking_rewards = U128(self.usdc_staking_rewards.0 + usdc_staking_rewards);

        // Add to staking rewards queue
        let match_stake_info = MatchStakeInfo {
            staking_rewards: U128(usdc_staking_rewards),
            stake_end_time: U64(env::block_timestamp() + ONE_MONTH),
        };
        self.staking_rewards_queue.push_back(match_stake_info);
    }

    pub(crate) fn handle_loss(&mut self, loss: u128) {}
}
