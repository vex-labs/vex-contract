use crate::*;
use near_sdk::near;

#[near]
impl Contract {
    pub fn get_matches(&self, from_index: Option<U64>, limit: Option<U64>) -> Vec<DisplayMatch> {}

    pub fn get_match(&self, match_id: MatchId) -> DisplayMatch {}

    pub fn get_potential_winnings(&self, match_id: MatchId, team: Team, bet_amount: U128) -> U128 {}

    pub fn get_bet(&self, match_id: MatchId, bet_id: BetId) -> Bet {}

    pub fn get_users_bets(
        &self,
        bettor: AccountId,
        from_index: Option<U64>,
        limit: Option<U64>,
    ) -> Vec<BetId, MatchId> {
    }

    pub fn get_admin(&self) -> AccountId {}
}
