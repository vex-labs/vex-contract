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
    matches: IterableMap<MatchId, Match>,
    bets_by_user: LookupMap<AccountId, IterableMap<BetId, MatchId>>,
    last_bet_id: BetId,
    admin: AccountId,
}

#[near(serializers = [borsh])]
pub struct Match {
    game: String,
    team_1: String,
    team_2: String,
    team_1_total_bets: U128,
    team_2_total_bets: U128,
    team_1_initial_pool: U128,
    team_2_initial_pool: U128,
    match_state: MatchState,
    winner: Option<Team>,
    bets: IterableMap<BetId, Bet>,
}

#[near(serializers = [json, borsh])]
pub struct Bet {
    bettor: AccountId,
    team: Team,
    bet_amount: U128,
    potential_winnings: U128,
    pay_state: Option<PayState>,
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
pub const USDC_CONTRACT_ID: &'static str = "cusd.fakes.testnet";
const ONE_USDC: U128 = U128(1_000_000_000_000_000_000_000_000);

#[near]
impl Contract {
    #[init]
    #[private]
    pub fn init(admin: AccountId) -> Self {
        Self {
            matches: IterableMap::new(b"m"),
            bets_by_user: LookupMap::new(b"u"),
            last_bet_id: U64(0),
            admin,
        }
    }
}
