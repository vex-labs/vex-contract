# Frontend Notes

This file contains the information needed to understand how to interact with the frontend. If you are working on the frontend, please refer to this file for information. For further information like what each method does and returns, please refer to the README.md file or ping Owen on TG.

Please let me know if the contract needs any changes to make the frontend/indexer more efficient.

## Info

The admin on the contract is admin.betvex.testnet if you need to use any of the admin methods. It has private key "ed25519:2jDDEz3SDzY7ysvRoqeRQtzoAhJjybMvqePBHZKV4SkXquRLaroJpyxwCnPkc8oVREv3NMXPDfEHDMFySdifbRMQ".

The account used to create other accounts is users.betvex.testnet. It has private key "ed25519:2Y4pcSrMSbyfaDkhhtHoyMV8N3nRqUUZF1Zir4itTPWPYEJo4AxxHmtQ6ewHXcBdV3ozxSMaXMMUzj4PkBZjb7Qx".

The $USDC contract is usdc.betvex.testnet.

The $VEX contract is token.betvex.testnet.

To get these tokens ask Owen.

Also take a look at the DeFi docs for information about the tokens like how many decimals they have.

## Calls needed

**Betting** 
Make a bet on a match and team

ft_transfer_call {"receiver_id": "contract.betvex.testnet", "amount": "1000000", "msg":  "{"Bet" : {"match_id": "RUBY-Nexus-17/08/2024", "team": “Team1”}}"}

Make this call on the USDC token contract 

**Claiming**
Claim the winnings or refund for a bet 

claim {"bet_id": "1"}

**Staking**
Stake VEX

ft_transfer_call {"receiver_id": "contract.betvex.testnet", "amount": "1000000", "msg": "Stake"}

Make this call on the vex token contract 

**Unstaking**
Unstake a certain amount of VEX 

unstake {"amount": "1000000"}

Unstake all VEX

unstake_all {}

**perform_stake_swap**
Swap USDC staking rewards for VEX for everyone

perform_stake_swap {}

## View calls 

**get_matches**
Get a list of matches

get_matches {"from_index": null, "limit": null}

**get_match**
Get a single match

get_match {"match_id": "RUBY-Nexus-17/08/2024"}

**get_potential_winnings**
Get the potential winnings if you were to make a bet right now.
You can use this to get the real odds for a bet by diving the result by the bet amount.

get_potential_winnings {"match_id": "RUBY-Nexus-17/08/2024", "team": "Team1", "bet_amount": "1000000"}

**get_users_bets**
Get a list of bet IDs and their associated match IDs for a user. Should be used when viewing bets for a user.

get_users_bets {"bettor": "pivortex.testnet", "from_index": null, "limit": null}

**get_bet**
Get the bet info of a single bet. Should be called when viewing bets for a user.

get_bet {"bettor": "pivortex.testnet", "bet_id": "1"}

**get_user_staked_bal**
Get the amount of VEX staked by a user. Should be called to get the amount of VEX staked by a user. Should also be used to determine whether a a user can unstake the inputted amount of VEX (cannot unstake more than they have staked and cannot unstake an amount such that it would leave them will less than 50 VEX staked unless the unstake all).

get_user_staked_bal {"account_id": "pivortex.testnet"}

**get_user_stake_info**
Get the stake info for a user. Will need to be used to check whether they are able to unstake (they cannot unstake if the timestamp has not passed).

get_user_stake_info {"account_id": "pivortex.testnet"}

**can_stake_swap_happen**
Get whether a stake swap can happen. Should be called on the staking page to determine whether the button to swap rewards should be enabled.

can_stake_swap_happen {}

## Ref finance swap 

For swapping in ref finance [this page](https://github.com/vex-labs/vex-frontend/blob/main/src/utils/swapTokens.js) will be useful.

## Faucet

This is an example call to get USDC from the faucet:
https://testnet.nearblocks.io/txns/GyRhE8KBRGCTTN4UwzNCpGc7H4CQoYPiL2hXouLNZfNP

## Testing

There are two main system flows that the frontend should work seamlessly for.

1) A user stakes VEX, a user bets on a match, the match ends and the result is not in favour of the team they bet on (the match will have a net gain), the user can see they lost the bet, the user clicks to distribute the rewards, the user cam see they have gained VEX. The user unstakes a certain amount of VEX (not all).

2) A user stakes VEX, a user bets on a match, the match ends and the result is in favour of the team they bet on (the match will have a net loss), the user claims their winnings, the user should see that they have lost a certain amount of their VEX, the user unstakes all their VEX.

You should also test that the frontend does not allow a user to perform actions that are not possible, it is best to check this in the frontend and with view calls they having a failed transaction.


## Methods for testing
Must be called from the admin account.