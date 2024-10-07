use near_sdk::json_types::{U128, U64};
use near_sdk::store::{IterableMap, LookupMap};
use near_sdk::{near, AccountId, PanicOnDefault};
use uint::construct_uint;

pub mod admin;
pub mod betting;
pub mod ext;
pub mod ft_on_transfer;
pub mod staking;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    pub matches: IterableMap<MatchId, Match>,
    pub bets_by_user: LookupMap<AccountId, IterableMap<BetId, Bet>>,
    pub last_bet_id: BetId,
    pub admin: AccountId,
    pub usdc_contract: AccountId,
    pub vex_token_contract: AccountId,
    pub fees_fund: U128,
    pub insurance_fund: U128,
    pub users_stake: LookupMap<AccountId, UserStake>,
    pub total_staked_balance: U128,
    pub total_stake_shares: U128,
    pub treasury: AccountId,
    pub pool_id: u64,
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
    pub team_1_potential_winnings: U128,
    pub team_2_potential_winnings: U128,
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

#[near(serializers = [json, borsh])]
pub struct UserStake {
    pub stake_shares: U128,
    pub unstaked_balance: U128,
    pub unstake_timestamp: U64,
}

impl Default for UserStake {
    fn default() -> Self {
        Self {
            stake_shares: U128(0),
            unstaked_balance: U128(0),
            unstake_timestamp: U64(0),
        }
    }
}

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

pub type MatchId = String;
pub type BetId = U64;

pub const WEIGHT_FACTOR: f64 = 1000.0;
pub const ONE_USDC: U128 = U128(1_000_000); // Note that this will have to change if USDC decimals are not 6
pub const FIFTY_VEX: U128 = U128(50_000_000_000_000_000_000); // Note that this will have to change if VEX decimals are not 18
pub const STAKE_SHARE_PRICE_GUARANTEE_FUND: u128 = 1_000_000_000_000_000;
pub const INITIAL_ACCOUNT_BALANCE: u128 = 50_000_000_000_000_000_000; // The contract needs to be initialized with 50 VEX

#[near]
impl Contract {
    #[init]
    #[private]
    pub fn init(
        admin: AccountId,
        usdc_contract: AccountId,
        vex_token_contract: AccountId,
        treasury: AccountId,
        pool_id: u64,
    ) -> Self {
        let total_staked_balance = U128(INITIAL_ACCOUNT_BALANCE - STAKE_SHARE_PRICE_GUARANTEE_FUND);

        Self {
            matches: IterableMap::new(b"m"),
            bets_by_user: LookupMap::new(b"u"),
            last_bet_id: U64(0),
            admin,
            usdc_contract,
            vex_token_contract,
            fees_fund: U128(0),
            insurance_fund: U128(0),
            users_stake: LookupMap::new(b"s"),
            total_staked_balance,
            total_stake_shares: total_staked_balance,
            treasury,
            pool_id,
        }
    }
}
