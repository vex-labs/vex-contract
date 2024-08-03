use near_sdk::json_types::U128;
use near_sdk::{env, near, require, serde_json, Gas, NearToken};

pub use crate::ext::*;
use crate::*;

#[near]
impl Contract {
    pub fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> U128 {
        require!(
            env::predecessor_account_id() == USDC_CONTRACT_ID.parse::<AccountId>().unwrap(),
            "Bets can only be made in USDC"
        );

        require!(amount >= ONE_USDC, "You must bet at least one USDC");

        #[near(serializers = [json])]
        struct BetInfo {
            match_id: MatchId,
            team: Team,
        }

        // Parse msg into match_id and team
        let BetInfo { match_id, team }: BetInfo = serde_json::from_str(&msg)
            .unwrap_or_else(|err: serde_json::Error| panic!("Invalid json {}", err));

        // Get relevant match
        let relevant_match = self
            .matches
            .get_mut(&match_id)
            .unwrap_or_else(|| panic!("No match exists with match id: {}", &match_id));

        require!(
            matches!(relevant_match.match_state, MatchState::Future),
            "Match state must be Future to bet on it"
        );

        match team {
            Team::Team1 => {
                relevant_match.team_1_total_bets =
                    U128(relevant_match.team_1_total_bets.0 + amount.0)
            }
            Team::Team2 => {
                relevant_match.team_2_total_bets =
                    U128(relevant_match.team_2_total_bets.0 + amount.0)
            }
        }

        // Insert bet ID into bets by user - creates a new map if there isn't one
        if self.bets_by_user.get(&sender_id).is_none() {
            let new_map: IterableMap<BetId, MatchId> = IterableMap::new(sender_id.as_bytes());
            self.bets_by_user.insert(sender_id.clone(), new_map);
        };

        let bets_by_user = self.bets_by_user.get_mut(&sender_id).unwrap();

        self.last_bet_id.0 += 1;
        bets_by_user.insert(self.last_bet_id, match_id);

        // Determines potential winnings
        let potential_winnings = determine_potential_winnings(
            &team,
            &relevant_match.team_1_total_bets,
            &relevant_match.team_2_total_bets,
            &amount,
        );

        // Creates a new bet
        let new_bet = Bet {
            bettor: sender_id,
            team,
            bet_amount: amount,
            potential_winnings,
            pay_state: None,
        };

        // Inserts the new bet
        relevant_match.bets.insert(self.last_bet_id, new_bet);

        U128(0)
    }

    pub fn claim_winnings(&mut self, match_id: &MatchId, bet_id: &BetId) -> U128 {
        // Get relevant match
        let relevant_match = self
            .matches
            .get_mut(match_id)
            .unwrap_or_else(|| panic!("No match exists with match id: {}", match_id));

        require!(
            matches!(relevant_match.match_state, MatchState::Finished),
            "Match state must be Finished to claim winnings"
        );

        // Get relevant bet
        let relevant_bet = relevant_match
            .bets
            .get_mut(bet_id)
            .unwrap_or_else(|| panic!("No bet exists with bet id: {:?}", bet_id));

        require!(
            env::predecessor_account_id() == relevant_bet.bettor,
            "You did not make that bet so you cannot claim it"
        );

        require!(
            matches!(relevant_bet.pay_state, None),
            "You have already been paid out"
        );

        // Checks they selected the winning team
        if let Some(winner) = &relevant_match.winner {
            require!(
                &relevant_bet.team == winner,
                "You did not select the winning team"
            );
        } else {
            panic!("There is an error")
        };

        // Transfer USDC of amount potential winnings to the bettor
        ft_contract::ext(USDC_CONTRACT_ID.parse().unwrap())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(Gas::from_tgas(30))
            .ft_transfer(relevant_bet.bettor.clone(), relevant_bet.potential_winnings);

        relevant_bet.pay_state = Some(PayState::Paid);

        relevant_bet.potential_winnings
    }

    pub fn claim_refund(&mut self, match_id: &MatchId, bet_id: &BetId) -> U128 {
        // Get relevant match
        let relevant_match = self
            .matches
            .get_mut(match_id)
            .unwrap_or_else(|| panic!("No match exists with match id: {}", match_id));

        require!(
            matches!(relevant_match.match_state, MatchState::Error),
            "Match state must be in Error to claim refund"
        );

        // Find relevant bet
        let relevant_bet = relevant_match
            .bets
            .get_mut(&bet_id)
            .unwrap_or_else(|| panic!("No bet exists with bet id: {:?}", bet_id));

        require!(
            env::predecessor_account_id() == relevant_bet.bettor,
            "You did not make that bet so you cannot claim it"
        );

        require!(
            matches!(relevant_bet.pay_state, None),
            "You have already been paid out"
        );

        // Transfer USDC of amount potential winnings to the bettor
        ft_contract::ext(USDC_CONTRACT_ID.parse().unwrap())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(Gas::from_tgas(30))
            .ft_transfer(relevant_bet.bettor.clone(), relevant_bet.bet_amount);

        relevant_bet.pay_state = Some(PayState::RefundPaid);

        relevant_bet.bet_amount
    }
}

pub fn determine_potential_winnings(
    team: &Team,
    team_1_total_bets: &U128,
    team_2_total_bets: &U128,
    bet_amount: &U128,
) -> U128 {
    let (betted_team_bets, other_team_bets) = match team {
        Team::Team1 => (team_1_total_bets, team_2_total_bets),
        Team::Team2 => (team_2_total_bets, team_1_total_bets),
    };

    let betted_team_bets = betted_team_bets.0 as f64;
    let other_team_bets = other_team_bets.0 as f64;
    let bet_amount = bet_amount.0 as f64;

    let ln_target = (betted_team_bets + bet_amount) / betted_team_bets;
    let val = (1.0 / 1.05) * (bet_amount + other_team_bets * ln_target.ln());

    U128(val as u128)
}
