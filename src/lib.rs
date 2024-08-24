use near_sdk::json_types::{U128, U64};
use near_sdk::store::{IterableMap, LookupMap};
use near_sdk::{near, AccountId, PanicOnDefault};

pub mod admin;
pub mod bettor;
pub mod ext;
pub mod view;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    pub matches: IterableMap<MatchId, Match>,
    pub bets_by_user: LookupMap<AccountId, IterableMap<BetId, Bet>>,
    pub last_bet_id: BetId,
    pub admin: AccountId,
    pub usdc_contract: AccountId,
}

#[near(serializers = [borsh])]
pub struct Match {
    pub game: String,
    pub team_1: String,
    pub team_2: String,
    pub team_1_total_bets: U128,
    pub team_2_total_bets: U128,
    pub team_1_initial_pool: U128,
    pub team_2_initial_pool: U128,
    pub match_state: MatchState,
    pub winner: Option<Team>,
}

#[near(serializers = [json, borsh])]
pub struct Bet {
    pub match_id: MatchId,
    pub team: Team,
    pub bet_amount: U128,
    pub potential_winnings: U128,
    pub pay_state: Option<PayState>,
}

#[derive(PartialEq, Clone)]
#[near(serializers = [json, borsh])]
pub enum Team {
    Team1,
    Team2,
}

#[near(serializers = [json, borsh])]
pub enum PayState {
    Paid,
    RefundPaid,
}

#[derive(Clone)]
#[near(serializers = [json, borsh])]
pub enum MatchState {
    Future,
    Current,
    Finished,
    Error,
}

pub type MatchId = String;
pub type BetId = U64;

pub const WEIGHT_FACTOR: f64 = 1000.0;
const ONE_USDC: U128 = U128(1_000_000_000_000_000_000_000_000);

#[near]
impl Contract {
    #[init]
    #[private]
    pub fn init(admin: AccountId, usdc_contract: AccountId) -> Self {
        Self {
            matches: IterableMap::new(b"m"),
            bets_by_user: LookupMap::new(b"u"),
            last_bet_id: U64(0),
            admin,
            usdc_contract,
        }
    }
}
