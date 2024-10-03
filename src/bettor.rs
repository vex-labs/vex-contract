use near_sdk::json_types::U128;
use near_sdk::{env, near, require, serde_json, Gas, NearToken};

pub use crate::ext::*;
use crate::*;

#[near]
impl Contract {
    pub fn claim(&mut self, bet_id: &BetId) -> U128 {
        let bettor = env::predecessor_account_id();

        // Get relevant user
        let relevant_user = self
            .bets_by_user
            .get_mut(&bettor)
            .unwrap_or_else(|| panic!("You have not made a bet"));

        // Get relevant bet
        let relevant_bet = relevant_user
            .get_mut(bet_id)
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

                // Transfer USDC of amount potential winnings to the bettor
                ft_contract::ext(self.usdc_contract.clone())
                    .with_attached_deposit(NearToken::from_yoctonear(1))
                    .with_static_gas(Gas::from_tgas(30))
                    .ft_transfer(bettor, relevant_bet.potential_winnings);

                relevant_bet.pay_state = Some(PayState::Paid);

                return relevant_bet.potential_winnings;
            }
            MatchState::Error => {
                // Transfer USDC of amount potential winnings to the bettor
                ft_contract::ext(self.usdc_contract.clone())
                    .with_attached_deposit(NearToken::from_yoctonear(1))
                    .with_static_gas(Gas::from_tgas(30))
                    .ft_transfer(bettor, relevant_bet.bet_amount);

                relevant_bet.pay_state = Some(PayState::RefundPaid);

                return relevant_bet.bet_amount;
            }
            _ => panic!("Match state must be Finished or Error to claim funds"),
        }
    }
}