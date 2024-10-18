use near_sdk::json_types::U128;
use near_sdk::{env, near, require, Gas, NearToken, PromiseError};

pub use crate::ext::*;
use crate::*;

#[near]
impl Contract {
    // Function to bet on a match with USDC
    pub(crate) fn bet(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        match_id: MatchId,
        team: Team,
    ) {
        require!(
            env::predecessor_account_id() == self.usdc_token_contract,
            "Bets can only be made in USDC"
        );

        require!(amount.0 >= ONE_USDC, "You must bet at least one USDC");

        // Get relevant match
        let relevant_match = self
            .matches
            .get_mut(&match_id)
            .unwrap_or_else(|| panic!("No match exists with match id: {}", &match_id));

        require!(
            matches!(relevant_match.match_state, MatchState::Future),
            "Match state must be Future to bet on it"
        );

        // Determines potential winnings
        let potential_winnings = determine_potential_winnings(
            &team,
            &relevant_match.team_1_total_bets,
            &relevant_match.team_2_total_bets,
            &amount,
        );

        // Increment total bets for the team
        match team {
            Team::Team1 => {
                relevant_match.team_1_total_bets =
                    U128(relevant_match.team_1_total_bets.0 + amount.0);
                relevant_match.team_1_potential_winnings =
                    U128(relevant_match.team_1_potential_winnings.0 + potential_winnings.0)
            }
            Team::Team2 => {
                relevant_match.team_2_total_bets =
                    U128(relevant_match.team_2_total_bets.0 + amount.0);
                relevant_match.team_2_potential_winnings =
                    U128(relevant_match.team_2_potential_winnings.0 + potential_winnings.0)
            }
        }

        // Creates a new bet
        let new_bet = Bet {
            match_id,
            team,
            bet_amount: amount,
            potential_winnings,
            pay_state: None,
        };

        // Increments bet ID
        self.last_bet_id.0 += 1;
        let bet_id_string = self.last_bet_id.0.to_string();

        // Inserts the new bet, creates a new map if the user has not bet previously
        if self.bets_by_user.get(&sender_id).is_none() {
            let new_map: IterableMap<BetId, Bet> = IterableMap::new(bet_id_string.as_bytes());
            self.bets_by_user.insert(sender_id.clone(), new_map);
        };

        let bets_by_user = self.bets_by_user.get_mut(&sender_id).unwrap();

        bets_by_user.insert(self.last_bet_id, new_bet);
    }

    // Function to claim winnings or refund
    pub fn claim(&mut self, bet_id: BetId) {
        require!(
            env::prepaid_gas() >= Gas::from_tgas(150),
            "You need to attach 300 TGas"
        );

        let bettor = env::predecessor_account_id();

        // Get relevant user
        let relevant_user = self
            .bets_by_user
            .get_mut(&bettor)
            .unwrap_or_else(|| panic!("You have not made a bet"));

        // Get relevant bet
        let relevant_bet = relevant_user
            .get_mut(&bet_id)
            .unwrap_or_else(|| panic!("No bet exists with bet id: {:?}", bet_id));

        require!(
            matches!(relevant_bet.pay_state, None),
            "You have already been paid out"
        );

        let match_id = &relevant_bet.match_id;

        // Get match state of the match in the bet
        let relevant_match = self.matches.get(match_id).unwrap_or_else(|| {
            panic!(
                "No match exists with match id: {} there must have been an error",
                match_id
            )
        });

        // Different flow depending on whether win or refund
        match relevant_match.match_state {
            MatchState::Finished => {
                // Checks they selected the winning team
                if let Some(winner) = &relevant_match.winner {
                    require!(
                        &relevant_bet.team == winner,
                        "You did not select the winning team"
                    );
                } else {
                    panic!("There is an error")
                };

                // Transfer USDC of amount potential_winnings to the bettor
                ft_contract::ext(self.usdc_token_contract.clone())
                    .with_attached_deposit(NearToken::from_yoctonear(1))
                    .with_static_gas(Gas::from_tgas(30))
                    .ft_transfer(bettor.clone(), relevant_bet.potential_winnings)
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(Gas::from_tgas(50))
                            .claim_callback(bettor, bet_id),
                    );

                relevant_bet.pay_state = Some(PayState::Paid);
            }
            MatchState::Error => {
                // Transfer USDC of amount bet_amount to the bettor
                ft_contract::ext(self.usdc_token_contract.clone())
                    .with_attached_deposit(NearToken::from_yoctonear(1))
                    .with_static_gas(Gas::from_tgas(30))
                    .ft_transfer(bettor.clone(), relevant_bet.bet_amount)
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(Gas::from_tgas(50))
                            .claim_callback(bettor, bet_id),
                    );
                relevant_bet.pay_state = Some(PayState::RefundPaid);
            }
            _ => panic!("Match state must be Finished or Error to claim funds"),
        }
    }

    #[private]
    pub fn claim_callback(
        &mut self,
        #[callback_result] call_result: Result<(), PromiseError>,
        bettor: AccountId,
        bet_id: BetId,
    ) -> String {
        if call_result.is_err() {
            // Get relevant user
            let relevant_user = self
                .bets_by_user
                .get_mut(&bettor)
                .unwrap_or_else(|| panic!("You have not made a bet"));

            // Get relevant bet
            let relevant_bet = relevant_user
                .get_mut(&bet_id)
                .unwrap_or_else(|| panic!("No bet exists with bet id: {:?}", bet_id));

            relevant_bet.pay_state = None;

            return "Failed transfer".to_string();
        }
        return "Successful transfer".to_string();
    }
}

// Function to determine potential winnings
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
