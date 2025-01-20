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
Get a list of bet IDs and their associated match IDs for a user.

get_users_bets {"bettor": "pivortex.testnet", "from_index": null, "limit": null}

**get_bet**
Get a single bet

get_bet {"bettor": "pivortex.testnet", "bet_id": "1"}

**get_user_staked_bal**
Get the amount of VEX staked by a user

get_user_staked_bal {"account_id": "pivortex.testnet"}

**get_user_stake_info**
Get the stake info for a user
Will need to be used to get when they can next unstake

get_user_stake_info {"account_id": "pivortex.testnet"}

I need to implement a view method to determine whether the stake swap can happen.

## Ref finance swap 

For swapping in ref finance [this page](https://github.com/vex-labs/vex-frontend/blob/main/src/utils/swapTokens.js) will be useful.

## Faucet

This is an example call to get USDC from the faucet:
https://testnet.nearblocks.io/txns/GyRhE8KBRGCTTN4UwzNCpGc7H4CQoYPiL2hXouLNZfNP

