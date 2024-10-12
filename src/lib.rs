use std::collections::VecDeque;

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
    // The account ID of this contract's admin
    pub admin: AccountId,

    // The USDC contract account ID
    pub usdc_token_contract: AccountId,

    // The VEX token contract account ID
    pub vex_token_contract: AccountId,

    // The treasury account ID
    pub treasury: AccountId,

    // The account ID of the Ref Finance contract
    pub ref_contract: AccountId,

    // The Ref Finance pool ID between USDC and VEX
    pub ref_pool_id: u64,

    // Map of all matches
    pub matches: IterableMap<MatchId, Match>,

    // Map of all bets ordered by user
    pub bets_by_user: LookupMap<AccountId, IterableMap<BetId, Bet>>,

    // The bet ID of the previous bet
    pub last_bet_id: BetId,

    // A map of balances related to staking for each user
    pub users_stake: LookupMap<AccountId, UserStake>,

    // A FIFO queue of matches that still have staking rewards to be distributed
    pub staking_rewards_queue: VecDeque<MatchStakeInfo>,

    // The total amount of USDC in the staking rewards fund, a sum of all in staking_rewards_queue
    pub usdc_staking_rewards: U128,

    // The timestamp of when the last stake swap occurred
    pub last_stake_swap_timestamp: U64,

    // The total unstaked VEX balance
    pub total_unstaked_balance: U128,

    // The total staked balance in VEX
    pub total_staked_balance: U128,

    // The total number of VEX stake shares
    pub total_stake_shares: U128,

    // The total amount of USDC in the fees fund
    pub fees_fund: U128,

    // The total amount of USDC in the insurance fund
    pub insurance_fund: U128,

    // The total amount of USDC that needs to be paid out
    pub funds_to_payout: U128,
}

#[near(serializers = [borsh])]
pub struct Match {
    // What game is being played
    pub game: String,

    // Team 1's name
    pub team_1: String,

    // Team 2's name
    pub team_2: String,

    // The total bets made on team 1 in USDC
    pub team_1_total_bets: U128,

    // The total bets made on team 2 in USDC
    pub team_2_total_bets: U128,

    // The initial pool of USDC for team 1
    pub team_1_initial_pool: U128,

    // The initial pool of USDC for team 2
    pub team_2_initial_pool: U128,

    // USDC to be paid out if team 1 wins
    pub team_1_potential_winnings: U128,

    // USDC to be paid out if team 2 wins
    pub team_2_potential_winnings: U128,

    // Whether the match is in the future, current, finished, or had an error
    pub match_state: MatchState,

    // The winning team
    pub winner: Option<Team>,
}

#[near(serializers = [json, borsh])]
pub struct Bet {
    // The match that is being bet on
    pub match_id: MatchId,

    // The team that is being bet on
    pub team: Team,

    // The amount of USDC being bet
    pub bet_amount: U128,

    // The winnings in USDC if the bet is successful
    pub potential_winnings: U128,

    // Whether the bet has been paid out
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
    // The number of stake shares the user has
    pub stake_shares: U128,

    // The amount of VEX the user has that is unstaked
    pub unstaked_balance: U128,

    // The timestamp of when the user can unstake their VEX
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

#[near(serializers = [json, borsh])]
pub struct MatchStakeInfo {
    // The USDC profit from the match that is to be distributed
    pub staking_rewards: U128,

    // The timestamp of when rewards will no longer be distributed (a month after the match ends)
    pub stake_end_time: U64,
}

// Construct a 256-bit unsigned integer
construct_uint! {
    pub struct U256(4);
}

// A unique identifier for a match of the form "team_1-team_2-date"
pub type MatchId = String;

// A unique identifier for a bet
pub type BetId = U64;

// The weight factor used to determine the inital pool sizes
pub const WEIGHT_FACTOR: f64 = 1000.0;

// One USDC in its lowest denomination
pub const ONE_USDC: u128 = 1_000_000; // Note that this will have to change if USDC decimals are not 6

// Fifty VEX in its lowest denomination
pub const FIFTY_VEX: u128 = 50_000_000_000_000_000_000; // Note that this will have to change if VEX decimals are not 18

// Amount of VEX allocated for rounding errors
pub const STAKE_SHARE_PRICE_GUARANTEE_FUND: u128 = 1_000_000_000_000_000;

// The initial account balance for the contract
pub const INITIAL_ACCOUNT_BALANCE: u128 = 50_000_000_000_000_000_000; // The contract needs to be initialized with 50 VEX

// One month in nanoseconds
pub const ONE_MONTH: u64 = 2_628_000_000_000_000;

// One week in nanoseconds
pub const ONE_WEEK: u64 = 604_800_000_000_000;

#[near]
impl Contract {
    #[init]
    #[private]
    pub fn init(
        admin: AccountId,
        usdc_token_contract: AccountId,
        vex_token_contract: AccountId,
        treasury: AccountId,
        ref_contract: AccountId,
        ref_pool_id: u64,
    ) -> Self {
        let total_staked_balance = U128(INITIAL_ACCOUNT_BALANCE - STAKE_SHARE_PRICE_GUARANTEE_FUND);

        Self {
            admin,
            usdc_token_contract,
            vex_token_contract,
            treasury,
            ref_contract,
            ref_pool_id,
            matches: IterableMap::new(b"m"),
            bets_by_user: LookupMap::new(b"u"),
            last_bet_id: U64(0),
            users_stake: LookupMap::new(b"s"),
            staking_rewards_queue: VecDeque::new(),
            usdc_staking_rewards: U128(0),
            last_stake_swap_timestamp: U64(0),
            total_unstaked_balance: U128(0),
            total_staked_balance,
            total_stake_shares: total_staked_balance,
            fees_fund: U128(0),
            insurance_fund: U128(0),
            funds_to_payout: U128(0),
        }
    }
}
