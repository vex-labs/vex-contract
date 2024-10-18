use near_sdk::json_types::U128;
use near_sdk::{near, serde_json};

pub use crate::ext::*;
use crate::*;

#[near(serializers = [json])]
struct BetInfo {
    match_id: MatchId,
    team: Team,
}

#[near(serializers = [json])]
enum FtTransferAction {
    Stake,
    AddUSDC,
    Bet(BetInfo),
}

#[near]
impl Contract {
    pub fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> U128 {
        // Send to relevant function based on msg
        match serde_json::from_str(&msg) {
            Ok(FtTransferAction::Stake) => {
                self.deposit(sender_id, amount);
            }
            Ok(FtTransferAction::AddUSDC) => {
                self.add_usdc(amount);
            }
            Ok(FtTransferAction::Bet(bet_info)) => {
                self.bet(sender_id, amount, bet_info.match_id, bet_info.team);
            }

            // add option to fill up the difference
            Err(err) => {
                panic!("Invalid call {}", err);
            }
        }

        U128(0)
    }
}
