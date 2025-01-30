use near_sdk::log;
use near_sdk::serde::Serialize;
use near_sdk::serde_json::json;

use crate::*;

const EVENT_STANDARD: &str = "betvex-contract";
const EVENT_STANDARD_VERSION: &str = "1.0.0";

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
#[must_use = "Don't forget to `.emit()` this event"]
pub enum Event<'a> {
    NewMatch {
        match_id: MatchId,
        game: String,
        date: U64,
        team_1: String,
        team_2: String,
        team_1_initial_pool: U128,
        team_2_initial_pool: U128,
    },
    EndBetting {
        match_id: MatchId,
    },
    CancelMatch {
        match_id: MatchId,
    },
    FinishMatch {
        match_id: MatchId,
        winner: Team,
    },
    Bet {
        account_id: &'a AccountId,
        bet_id: BetId,
        amount: U128,
        match_id: MatchId,
        team: Team,
        potential_winnings: U128, 
        new_team_1_pool_size: U128,
        new_team_2_pool_size: U128,
    },
    ClaimWinnings {
        account_id: &'a AccountId,
        bet_id: BetId,
        amount_received: U128,
    },
    ClaimRefund {
        account_id: &'a AccountId,
        bet_id: BetId,
        amount_received: U128,
    },
    StakeVex {
        account_id: &'a AccountId,
        amount: U128,
        new_total_staked: U128,
    },
    UnstakeVex {
        account_id: &'a AccountId,
        amount: U128,
        new_total_staked: U128,
    },
}

impl Event<'_> {
    pub fn emit(&self) {
        let data = json!(self);
        let event_json = json!({
            "standard": EVENT_STANDARD,
            "version": EVENT_STANDARD_VERSION,
            "event": data["event"],
            "data": [data["data"]]
        })
        .to_string();
        log!("EVENT_JSON:{}", event_json);
    }
}
