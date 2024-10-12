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
    // Stores the admin account ID
    pub admin: AccountId,

    // Stores the USDC contract account ID
    pub usdc_token_contract: AccountId,

    // Stores the VEX token contract account ID
    pub vex_token_contract: AccountId,

    // Stores the treasury account ID
    pub treasury: AccountId,

    // Stores the account ID of the Ref Finance contract
    pub ref_contract: AccountId,

    // Stores the pool ID of the Ref Finance contract
    pub ref_pool_id: u64,

    // Stores all matches 
    pub matches: IterableMap<MatchId, Match>,

    // Stores bets made by a user 
    pub bets_by_user: LookupMap<AccountId, IterableMap<BetId, Bet>>,

    // Stores the last bet ID
    pub last_bet_id: BetId,

    // Stores the balances related to staking for each user
    pub users_stake: LookupMap<AccountId, UserStake>,

    // Stores matches that still have staking rewards to be distributed
    pub staking_rewards_queue: VecDeque<MatchStakeInfo>,

    // Stores the timestamp of when the last stake swap occurred
    pub last_stake_swap_timestamp: U64,

    // Stores the total unstaked balance
    pub total_unstaked_balance: U128,

    // Stores the total staked balance in VEX
    pub total_staked_balance: U128,

    // Stores the total number of VEX stake shares
    pub total_stake_shares: U128,

    // Stores the amount of USDC in the staking rewards fund
    pub usdc_staking_rewards: U128,

    // Stores the amount of USDC in the fees fund
    pub fees_fund: U128,

    // Stores the amount of USDC in the insurance fund
    pub insurance_fund: U128,

    // Stores the amount of USDC that needs to be paid out
    pub funds_to_payout: U128,
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

#[near(serializers = [json, borsh])]
pub struct MatchStakeInfo {
    pub staking_rewards: U128,
    pub stake_end_time: U64,
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
pub const ONE_MONTH: u64 = 2_628_000_000_000_000;
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
            last_stake_swap_timestamp: U64(0),
            total_unstaked_balance: U128(0),
            total_staked_balance,
            total_stake_shares: total_staked_balance,
            usdc_staking_rewards: U128(0),
            fees_fund: U128(0),
            insurance_fund: U128(0),
            funds_to_payout: U128(0),
        }
    }
}
