# vex-contract

This repo holds the main VEX contract that includes a betting and staking system.
Please note that these docs are currently out of date since the staking system was added 

# Contributing

> [!Caution]
> Do **NOT** push directly to or merge your own commits into main. This repo has no branch protection rules so no warning will displayed if you try to do this.

To contribute:
1) Open a new branch with an appropriate name and select main as the source.
2) Commit your code to your branch.
3) Create a PR to main from your branch.
4) If your contribution solves an issue please write "Closes #issue-number" in the PR.
5) Select Owen (PiVortex) as a reviewer.

If you need any help with this please don't hesitate to ask for help.

# Deployments

- A stable contract (the last release) is deployed at "contract.betvex.testnet".

# Admin 

- The admin is set to admin.betvex.testnet, ask Owen for private key

# Events 
Events are emitted when key methods are called / completed. The list of events and data emitted can be found in the [`events.rs`](./src/events.rs) file.

# Flow

## Betting flow 
1) The bettor selects a match and calls `ft_transfer_call` on the USDC contract which calls `ft_on_transfer` on the betting contract.
2) If the bet was successful or the match was cancelled the bettor calls `claim`.

## Admin / Oracle flow

1) When a new match needs to be added the admin calls `create_match`. 
2) When the match starts the admin calls `end_betting`. 
3) When the match finishes and the results are known the admin calls `finish_match`. 

- If there is a problem with a match the admin calls `cancel_match` between stages 1) and 3).

## User Staking Flow
1) A user calls `ft_transfer_call` on the VEX contract which calls `ft_on_transfer` on the contract, with the message `Stake` to stake VEX into the contract.
2) The user calls `unstake` or `unstake_all` to unstake their VEX.

## Contract Staking Flow
1) When the admin calls `finish_match` the game either resulted in a profit or a loss.
In the case of profit:
    1) The profit is added to the rewards queue.
    2) When the rewards have built up enough anyone calls `perform_stake_swap` to swap the rewards for VEX and distribute them to the users.
In the case of loss: 
    1) The insurance fund is used to cover the loss if possible, if not then VEX is sold to cover the loss.

From this point on the docs have not been updated to reflect the addition of the staking system. The main changes to add the staking system can be found [HERE](https://github.com/vex-labs/vex-contract/commit/e7ba1596212c9e09ea282bb7438350781046ebf6#diff-2a76032456d2f72cc6b25681ab39f258aca3ec14dc2035f1cd103cef599dd519L17)

# Methods

## Change Methods

### ft_on_transfer

Used to place a bet on a match. Callable via [ft_transfer_call](https://docs.near.org/build/primitives/ft#attaching-fts-to-a-call).

**ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> U128** 

1) Checks the message msg field.
If the message is `Stake` then the `stake` method is called.
If the message is `AddUSDC` then the `add_usdc` method is called.
If the message is `Bet` then the `bet` method is called.

- **sender_id: AccountId** The account ID of the bettor.
- **amount: U128** The bet amount in USDC. One whole USDC is 10^24.
- **msg: String** Stores the other information needed to place a bet in JSON in the format of `BetInfo`.

Returns the leftover USDC from the call, which will always be zero.

### claim

Used by a bettor to claim bet winnings or refund.

**claim(&mut self, bet_id: &BetID)**

1) Fetches the relevant bet from `bets_by_user`.
2) Checks that `pay_state` is `None`.
3) Checks that `match_state` is `Finished` or `Error`.
- If the match is `Finished`
    1) Checks that they selected the winning team.
    2) Transfers USDC equal to `potential_winnings` to the `bettor`.
    3) Changes `pay_state` to `Paid`.
- If the match is `Error`
    1) Transfers USDC equal to `bet_amount` to the `bettor`.
    2) Changes `pay_state` to `RefundPaid`.
4) Then it makes a call to `claim_callback` to verify the transfer was successful, if not it will revert the paystate to `None`.
5) If the transfer was successful it emits an event.

- **bet_id: &BetID** The bet ID of the bet the bettor is claiming their winnings for.

### perform_stake_swap

Swaps the USDC staking rewards for VEX.

**perform_stake_swap(&mut self) -> PromiseOrValue<()>**

1) Check that the user has attached 300 TGas.
2) Call `perform_stake_swap_internal`.

Returns a promise.

### unstake

Unstakes a given amount of VEX and sends it to the user.

**unstake(&mut self, amount: U128) -> Promise**

1) Checks that the user is unstaking more than 0 VEX.
2) Finds the relevant account from `users_stake`.
3) Checks that the user can unstake and that the unstake timestamp has passed.
4) Calculates the number of shares required to unstake the given amount.
5) Checks that the user has enough shares to unstake.
6) Calculates the amount of VEX the user will receive by unstaking the corresponding number of shares.
7) Caculates the amount of VEX that will be unstaked from the total to guarantee the "stake" share price never decreases.
8) Make sure the user keeps at least 50 VEX staked or unstakes all.
9) Modify the users stake shares and the total stake shares and the total staked balance.
10) If the user has no stake shares, remove them from the map.
11) Emit an event.
12) Transfer the amount of VEX to the user.

- **amount: U128** The amount of VEX the user is unstaking.

### unstake_all

Unstakes all VEX to their unstaked balance.

**unstake_all(&mut self) -> Promise**

1) Get the account ID of the user calling the method.
2) Find the relevant account from `users_stake`.
3) Calculate the amount of VEX the user will receive by unstaking all the "stake" shares.
4) Call `unstake` with the full amount from staked balance.

Returns a promise.

## Only Callable by Admin 

### create_match

Used to create a new match.

**create_match(&mut self, game: String, team_1: String, team_2: String, in_odds_1: f64, in_odds_2: f64, date: String)**

1) Checks that the `admin` is calling the method.
2) Creates the match ID.
3) Determines the initial pool sizes by multiplying the initial odds of each team winning by the `WEIGHT_FACTOR`.
4) Creates a new match and adds it to `matches`.
5) Emits an event.

- **game: String** What game the match is, e.g. Valorent, Overwatch, etc.
- **team_1: String** Name of the first team.
- **team_2: String** Name of the second team.
- **in_odds_1: f64** Average external odds for team 1 to win.
- **in_odds_2: f64** Average external odds for team 2 to win.
- **date: String** The date the match is taking place.

### end_betting

Used to close betting on the match.

**end_betting(&mut self, match_id: &MatchID)**

1) Checks that the `admin` is calling the method.
2) Fetches the relevant match from `matches`.
3) Checks that the match has the `match_state` `Future`.
4) Changes `match_state` to `Current`.
5) Emits an event.

- **match_id: &MatchID** The match ID of the match that betting is being ended for.

### finish_match

Used when a match finishes.

**finish_match(&mut self, match_id: &MatchID, winner: Team)**

1) Checks that the `admin` is calling the method.
2) Fetches the relevant match from `matches`.
3) Checks that the match has the `match_state` `Current`
4) Changes `match_state` to `Finished`.
5) Sets `winner`.
6) Calculates the total profit or loss.
7) Emits an event.
8) Calls `handle_profit` or `handle_loss` to handle the profit or loss.

- **match_id: &MatchID** The match ID of the match that is finished.
- **winner: Team** The team that won the game

### cancel_match

Used when there is an error with a match or the match is cancelled.

**cancel_match(&mut self, match_id: MatchID)**

1) Checks that the `admin` is calling the method.
2) Fetches the relevant match from `matches`.
3) Checks whether the `match_state` is `Future` or `Current`.
4) Changes the `match_state` to `Error`.
5) Emits an event.

- **match_id: MatchID** The match ID of the match that is being cancelled.

### take_from_fees_fund

Used to take an amount of funds from the fees fund and send it to the `receiver`.

**take_from_fees_fund(&mut self, amount: U128, receiver: AccountId) -> U128**

1) Checks that the `admin` is calling the method.
2) Checks that there are enough funds in the fees fund.
3) Transfers the funds to the `receiver`.
4) Emits an event.

- **amount: U128** The amount of funds to be transferred.
- **receiver: AccountId** The account ID of the receiver of the funds.

Returns the amount of funds left in the fees fund.

### take_from_insurance_fund

Used to take an amount of funds from the insurance fund and send it to the `receiver`.

**take_from_insurance_fund(&mut self, amount: U128, receiver: AccountId) -> U128**

1) Checks that the `admin` is calling the method.
2) Checks that there are enough funds in the insurance fund.
3) Transfers the funds to the `receiver`.
4) Emits an event.

- **amount: U128** The amount of funds to be transferred.
- **receiver: AccountId** The account ID of the receiver of the funds.

Returns the amount of funds left in the insurance fund.

### change_admin

Used to change the admin of the betting contract.

**change_admin(&mut self, new_admin: AccountId)**

1) Checks that the `admin` is calling the method.
2) Changes `admin` to `new_admin`.

- **new_admin: AccountId** The account ID of the new admin.

## Only Callable by the Contract Account 

### init

Initializes the contract.

**init(&mut self, admin: AccountId, usdc_token_contract: AccountId, vex_token_contract: AccountId, treasury: AccountId, ref_contract: AccountId, ref_pool_id: U64, rewards_period: U64, unstake_time_buffer: U64, min_swap_amount: U128)**

1) Sets initial values for the contract and initializes structures.

## View Methods

### get_contract_info

Fetches the contract info.

**get_contract_info(&self) -> ContractInfo**

1) Returns certain general contract info.

Returns the contract info.

### get_matches

Fetches a vector of matches within a limit.

**get_matches(&self, from_index: &Option&lt;u32&gt;, limit: Option&lt;u32&gt;) -> Vec&lt;DisplayMatch&gt;**

1) If `from_index` is `None` set to 0 and if `limit` is `None` then it is set to the length of `matches`.
2) Iterate through `matches`.
3) For each `Match` convert to `DisplayMatch` using `format_match`.
4) Add each `DisplayMatch` to a vector.
5) Return the vector.

- **from_index: &Option&lt;U64>** Specifies the index at which the method will start iterating.
- **limit: &Option&lt;U64>** Specifies how many matches will be collected.

Returns a vector of matches to display.

### get_match

Fetches a single match.

**get_match(&self, match_id:& MatchID) -> DisplayMatch**

1) Fetches the relevant match from `matches`.
2) Converts from `Match` to `DisplayMatch` using `format_match`.
3) Returns the match.

- **match_id: &MatchID** The match ID of the match to be fetched.

Returns a single instance of `DisplayMatch`.

### get_potential_winnings

Gets the amount in USDC the bettor would receive if they were to make a bet right now. 

**get_potential_winnings(&self, match_id: &MatchID, team: &Team, bet_amount: &U128) -> U128**

1) Fetches the relevant match from `matches`.
2) Checks which team they have picked.
3) Gets the potential winnings via `determine_potential_winnings`.
4) Return the potential winnings.

- **match_id: &MatchID** The match ID of the match the bettor would bet on.
- **team: &Team** The team that the bettor would bet on.
- **bet_amount: &U128** The amount in USDC the bettor would bet. One USDC is 10^24.

Returns the potential winnings.

### get_bet

Fetches a single bet.

**get_bet(&self, bettor: &AccountId, bet_id: &BetID) -> Bet** 

1) Fetches the relevant account from `bet_by_user`.
2) Fetches the relevant bet.
3) Returns the bet.

- **bettor: &AccountId** Account ID of the bettor for which bet will be returned.
- **bet_id: &BetID** The bet ID of the bet to be fetched.

Returns a single instance of `Bet`.

### get_users_bets

Fetches a vector of bet Ids and their associated match IDs within a limit for a single bettor.

**get_users_bets(&self, bettor: &AccountId, from_index: &Option&lt;u32&gt;, limit: Option&lt;u32&gt;) -> Vec&lt;(BetId, MatchId)&gt;**

1) If `from_index` is `None` set to 0 and if `limit` is `None` then it is set to the number of bets for the bettor.
2) Fetches the relevant account from `bet_by_user`.
2) Iterate through the map of bets.
3) For each bet add `BetId` and `Bet` to a vector.
4) Return the vector.

- **bettor: &AccountId** Account ID of the bettor for which the bets will be returned.
- **from_index: &Option&lt;U64&gt;** Specifies the index at which the method will start iterating.
- **limit: &Option&lt;U64&gt;** Specifies how many bet IDs will be fetched.

Returns a vector of BetIds and their Bet.

### get_user_staked_bal

Fetches the amount of VEX staked by a user.

**get_user_staked_bal(&self, account_id: AccountId) -> U128**

1) Fetches the relevant account from `users_stake`.
2) Returns the amount of VEX staked.

- **account_id: AccountId** Account ID of the user for which the staked balance will be returned.

Returns the amount of VEX staked by the user.

### get_user_stake_info

Fetches the stake info for a user.

**get_user_stake_info(&self, account_id: AccountId) -> &UserStake**

1) Fetches the relevant account from `users_stake`.
2) Returns the stake info.

- **account_id: AccountId** Account ID of the user for which the stake info will be returned.

Returns the stake info for the user.

### get_staking_rewards_queue

Fetches the staking rewards queue.

**get_staking_rewards_queue(&self) -> &VecDeque<MatchStakeInfo>**

1) Returns the staking rewards queue.

Returns the staking rewards queue.

### get_usdc_staking_rewards

Fetches the total USDC staking rewards.

**get_usdc_staking_rewards(&self) -> U128**

1) Returns the total USDC staking rewards.

Returns the total USDC staking rewards.

### get_last_stake_swap_timestamp

Fetches the last stake swap timestamp.

**get_last_stake_swap_timestamp(&self) -> U64**

1) Returns the last stake swap timestamp.

Returns the last stake swap timestamp.

### get_total_staked_balance

Fetches the total staked balance.

**get_total_staked_balance(&self) -> U128**

1) Returns the total staked balance.

Returns the total staked balance.

### get_total_stake_shares

Fetches the total stake shares.

**get_total_stake_shares(&self) -> U128**

1) Returns the total stake shares.

Returns the total stake shares.

### get_fees_fund

Fetches the fees fund balance.

**get_fees_fund(&self) -> U128**

1) Returns the fees fund balance.

Returns the fees fund balance.

### get_insurance_fund

Fetches the insurance fund balance.

**get_insurance_fund(&self) -> U128**

1) Returns the insurance fund balance.

Returns the insurance fund balance.

### get_funds_to_add

Fetches the amount of USDC that needs to be added to the contract.

**get_funds_to_add(&self) -> U128**

1) Returns the amount of USDC that needs to be added to the contract.

Returns the amount of USDC that needs to be added to the contract.

### can_stake_swap_happen

Checks if a stake swap can happen.

**can_stake_swap_happen(&self) -> bool**

1) Calculates the rewards that can be swapped as per **stake_swap_rewards_to_swap**
2) Checks if the rewards are greater than the minimum swap amount.
3) If the rewards are greater than the minimum swap amount then return true.
4) If the rewards are less than the minimum swap amount then return false.

Returns whether a stake swap can happen.

## Internal functions 

These are methods that are only callable by the contract.

### bet

Places a bet on a match. Called by `ft_on_transfer`.

**bet(&mut self, sender_id: AccountId, amount: U128, match_id: MatchId, team: Team)**

1) Checks that the token is USDC.
2) Checks they have bet one or more USDC.
3) Parses `msg`.
4) Fetches the match with the specified match ID and checks `match_state` is `Future`.
5) Calculates `potential_winnings` using `determine_potential_winnings`.
6) Adds bet amount to correct team's total bets.
7) Increments `last_bet_id`.
8) Inserts the a new `Bet` into into `bets_by_user`. 
9) Emits an event.
10) Returns U128(0).

- **sender_id: AccountId** The account ID of the user placing the bet.
- **amount: U128** The amount of USDC the user is betting.
- **match_id: MatchId** The match ID of the match the user is betting on.
- **team: Team** The team the user is betting on.

Returns U128(0).

### determine_approx_odds

Calculates the approximate odds for a match. These odds are if the bettor were to bet an infinitesimal amount.

**determine_approx_odds(team_1_total_bets: U128, team_2_total_bets: U128) -> (f64, f64)**

1) Calculates approximate odds.
2) Returns approximate odds.

- **team_1_total_bets: U128** Total bets on team 1 in USDC, this includes initial weightings.
- **team_1_total_bets: U128** Total bets on team 2 in USDC, this includes initial weightings.

Returns a tuple of odds for team 1 and team 2.

### determine_potential_winnings

Calculates the potential winnings for a bet.

**determine_potential_winnings(team: &Team, team_1_total_bets: &U128, team_2_total_bets: &U128, bet_amount: &U128,) -> U128**

1) Checks which team they have selected.
2) Calculates potential winnings for the given arguments.
3) Returns the potential winnings.

- **team: &Team** The team the bettor has selected.
- **team_1_total_bets: &U128** Total bets on team 1 in USDC, this includes initial weightings.
- **team_2_total_bets: &U128** Total bets on team 2 in USDC, this includes initial weightings.
- **bet_amount: &U128** The amount in USDC the bettor would bet. One USDC is 10^24.

Returns the potential winnings in USDC for a bet.

### format_match

Reformats a match's details from `Match` to `DisplayMatch`. 

**format_match(match_id, &MatchId, match_struct: &Match) -> DisplayMatch**

1) Get odds from `determine_approx_odds`.
2) Reformats from `Match` to `DisplayMatch`.
3) Returns an instance of `DisplayMatch`.

- **match_id: &MatchId** The match ID of the match to be formatted.
- **match_struct: &Match** The `Match` structure to be formatted.

Returns an instance of `DisplayMatch`.

### assert_one_yocto

Checks that a user has deposited 1 YoctoNEAR in the current call.

**assert_one_yocto()**

1) Check that the user has attached 1 YoctoNEAR.

### assert_admin

Checks that the user is the admin.

**assert_admin()**

1) Check that the user is the admin.

### handle_loss

Handles the case when a match finishes and there is a loss. Called by `finish_match`.

**handle_loss(&mut self, loss: u128) -> PromiseOrValue<()>**

1) If the loss can be covered by the insurance fund then do so.
2) If the loss cannot be covered by the insurance fund then call the `ref_contract` to get the amount of VEX needed to cover the loss.
3) Call `ref_loss_view_callback` passing the difference.
4) If the call fails then panic.
5) Add an extra 5% to the amount to account for price change between blocks.
6) Deposit VEX from the staking pool into ref finance and call `ref_loss_deposit_callback` passing the difference.
7) If the call fails then panic.
8) Set the insurance fund to 0 as it will all be used.
9) Remove the amount of VEX deposited from the total staked balance.
10) Call ref finance to swap the VEX for USDC and call `ref_loss_swap_callback` passing the difference.
11) If the call fails then panic.
12) Call ref finance to withdraw the USDC that was swapped into and call `ref_loss_withdraw_callback` passing the difference.
13) If the call fails then panic.
14) If the amount received is greater than the difference then add the excess to the insurance fund.
15) If the amount received is still less than the difference then set the amount of USDC that needs to be added to the contract.

- **loss: u128** The loss from a match in USDC.

Returns a promise.

### handle_profit

Handles the case when a match finishes and there is a profit. Called by `finish_match`.

**handle_profit(&mut self, profit: u128) -> PromiseOrValue<()>**

1) Calculate how profit is distributed.
2) Increase fees and insurance funds.
3) Send funds to treasury.
4) Call `perform_stake_swap_internal` with the amount of USDC for staking so rewards are distributed at the timestamp of the match being added to the list so extra rewards are not distributed.

- **profit: u128** The profit from a matchin USDC.

Returns a promise.

### perform_stake_swap_internal

Swaps the USDC staking rewards for VEX.

**perform_stake_swap_internal(&mut self, extra_usdc_for_staking: U128) -> PromiseOrValue<()>**

1) If the staking queue is empty then skip and update the last stake swap timestamp.
2) If extra_usdc_for_staking is greater than 0 then add it to the staking rewards queue and the total USDC staking rewards.
3) Iterate through the staking rewards queue and calculate the rewards for any matches that have expired and count the number of matches to pop.
4) Calculate the rewards for the matches that have not yet expired.
5) Update the last stake swap timestamp.
6) Call ref finance to deposit the USDC rewards and call `ref_profit_deposit_callback` passing the number of matches removed, the amount of USDC staking rewards left and the total rewards to swap.
7) If the call fails then reset the previous timestamp and return.
8) If the call succeeds then set the new total USDC staking rewards, remove the matches from the queue and add the new rewards to the queue.
9) Call ref finance to swap the USDC rewards for VEX and call `ref_profit_swap_callback` passing who called the method.
10) If the call fails then panic.
11) Call ref finance to withdraw the VEX that was swapped into and call `ref_profit_withdraw_callback` passing who called the method.
12) If the call fails then panic.
13) Reward the initial caller with 1% of the VEX that was swapped into.
14) Add the withdrawn VEX to the total staked balance.

- **extra_usdc_for_staking: U128** The extra USDC for staking if called by `handle_profit`.

### stake 

Stakes VEX tokens. Called by `ft_on_transfer`.

**stake(&mut self, sender_id: AccountId, amount: U128)**

1) Checks that VEX is being staked.
2) Get the rounded down number of stake shares.
3) Get the amount of VEX for the rounded down stake shares.
4) Check if the user's staked VEX + the amount they are staking is at least 50.
5) Get the user's stake account or create a new one if it doesn't exist.
6) Set the unstake timestamp to 1 week from now.
7) Update the user's staked shares balance.
8) Calculate the stake amount (rounding errors handling).
9) Update aggregate values.
10) Emit an event.

- **sender_id: AccountId** The account ID of the user staking the VEX.
- **amount: U128** The amount of VEX the user is staking.

### add_usdc

Adds USDC to the contract. Called by `ft_on_transfer`.

**add_usdc(&mut self, amount: U128)**

1) Checks that the caller is the USDC token contract.
2) If the funds to add is greater than 0 then add it to the funds to add.
3) Add the rest to the insurance fund.

- **amount: U128** The amount of USDC the user is adding to the contract.

### num_shares_from_staked_amount_rounded_down

Calculates the number of stake shares from a staked amount rounded down.

**num_shares_from_staked_amount_rounded_down(&self, amount: u128) -> u128**

1) Checks that the total staked balance is greater than 0.
2) Calculate the number of shares.

- **amount: u128** The amount of VEX the user is staking.

Returns the number of shares.

### num_shares_from_staked_amount_rounded_up

Calculates the number of stake shares from a staked amount rounded up.

**num_shares_from_staked_amount_rounded_up(&self, amount: u128) -> u128**

1) Checks that the total staked balance is greater than 0.
2) Calculate the number of shares.

- **amount: u128** The amount of VEX the user is staking.

Returns the number of shares.

### staked_amount_from_num_shares_rounded_down

Calculates the staked amount from the number of stake shares rounded down.

**staked_amount_from_num_shares_rounded_down(&self, num_shares: u128) -> u128**

1) Checks that the total staked balance is greater than 0.
2) Calculate the staked amount.

- **num_shares: u128** The number of shares the user has.

Returns the staked amount.

### staked_amount_from_num_shares_rounded_up

Calculates the staked amount from the number of stake shares rounded up.

**staked_amount_from_num_shares_rounded_up(&self, num_shares: u128) -> u128**

1) Checks that the total staked balance is greater than 0.
2) Calculate the staked amount.

- **num_shares: u128** The number of shares the user has.

Returns the staked amount.

# Storage

## Structures

### Contract

The entry structure for the contract.

- **admin: AccountID** The account ID of the account that can call admin methods. The oracle will be the admin.
- **usdc_token_contract: AccountID** The account ID of the USDC token contract.
- **vex_token_contract: AccountID** The account ID of the VEX token contract.
- **treasury: AccountID** The account ID of the treasury.
- **ref_contract: AccountID** The account ID of the Ref Finance contract.
- **ref_pool_id: u64** The pool ID of the Ref Finance pool between USDC and VEX.
- **matches: IterableMap&lt;MatchId, Match&gt;** A map of matches yet to take place. 
- **bets_by_user: LookupMap&lt;AccountId, IterableMap&lt;BetId, Bet&gt;&gt;** A map that gives the bet IDs and the match ID of the match the bet was placed on.
- **last_bet_id: BetId** An integer that stores the bet ID of the last bet. Used for inputting what the next bet ID will be. 
- **users_stake: LookupMap&lt;AccountId, UserStake&gt;** A map of users and their stake information.
- **staking_rewards_queue: VecDeque&lt;MatchStakeInfo&gt;** A FIFO queue of matches that still have staking rewards to be distributed.
- **usdc_staking_rewards: U128** The total amount of USDC in the staking rewards fund, a sum of all in staking_rewards_queue.
- **last_stake_swap_timestamp: U64** The timestamp of when the last stake swap occurred.
- **total_staked_balance: U128** The total amount of VEX staked in the contract.
- **total_stake_shares: U128** The total amount of stake shares in the contract.
- **fees_fund: U128** The amount of USDC in the fees fund.
- **insurance_fund: U128** The amount of USDC in the insurance fund.
- **funds_to_add: U128** The amount of USDC that needs to be added to the contract.
- **funds_to_payout: U128** The amount of USDC that needs to be paid out.
- **rewards_period: u64** The time that rewards for staking are distributed over in nanoseconds, default is one month - 2_628_000_000_000_000
- **unstake_time_buffer: u64** The buffer time in nanoseconds before a user unstake since last staking, default is one week - 604_800_000_000_000
- **min_swap_amount: u128** The minimum amount of rewards required to be able to swap, default is 100 USDC.


### Match

Stores the necessary information for a match.

- **game: String** What game the match is, e.g. Valorent, Overwatch, etc.
- **team_1: String** Name of team 1.
- **team_2: String** Name of team 2.
- **team_1_total_bets: U128** Total bets on team 1 in USDC, this includes initial weightings.
- **team_2_total_bets: U128** Total bets on team 2 in USDC, this includes initial weightings.
- **team_1_inital_pool: U128** Initial weightings adding to team 1's pool from initial odds.
- **team_2_inital_pool: U128** Initial weightings adding to team 2's pool from initial odds.
- **team_1_potential_winnings: U128** USDC to be paid out if team 1 wins.
- **team_2_potential_winnings: U128** USDC to be paid out if team 2 wins.
- **match_state: MatchState** An enumeration dictating what state the match is in.
- **winner: Option<Team>** An enumeration storing the winner of the match.

### Bet

Stores the necessary information for a bet.

- **match_id: MatchId** Match ID of the match they bet on.
- **team: Team** An enumeration storing the team they have bet on.
- **bet_amount: U128** The amount in USDC they bet on the match.
- **potential_winnings: U128** The amount the bettor will receive if they choose the correct team.
- **pay_state: Option&lt;PayState&gt;** An enumeration storing whether they have been paid out yet. If `None`, then either a winner is yet to be decided or the team they selected did not win.

### UserStake

Stores the necessary information for when a user stakes VEX.

- **stake_shares: U128** The number of stake shares the user has.
- **unstake_timestamp: U64** The timestamp of when the user can unstake their VEX.

### MatchStakeInfo

Stores the necessary information for when a match has staking rewards to be distributed.

- **staking_rewards: U128** The USDC profit from the match that is to be distributed.
- **stake_end_time: U64** The timestamp of when rewards will no longer be distributed (a month after the match ends).

## Enumerations

### Team

Stores which the team as an enum

- **Team1** Team 1
- **Team2** Team 2.

### PayState

Stores whether a bettor has been paid or not.

- **Paid** The bettor has been paid the amount equal to `potential_winnings`.
- **RefundPaid** The bettor has been paid the amount equal to `bet_amount`.

### MatchState

Stores what state a match is in. 

- **Future** The match has not started yet. 
- **Current** The match is taking place. 
- **Finished** The match is finished. 
- **Error** The match had an error or was cancelled. 

## Type Aliases

**MatchId: String** A combination of team names and the date that the match is set to take place in the form "team1-team2-dd/mm/yyyy".

**BetId: U64** A unique identifier for a single bet across the whole contract.

## Constants

**WEIGHT_FACTOR: f64 = 1000.0** Sets the weight of the initial odds. If this is higher then the odds will change less on user bets, more so initially. 

**ONE_USDC: u128 = 1_000_000** One USDC in its lowest denomination.

**FIFTY_VEX: u128 = 50_000_000_000_000_000_000** Fifty VEX in its lowest denomination.

**STAKE_SHARE_PRICE_GUARANTEE_FUND: u128 = 1_000_000_000_000_000_000** The amount of VEX allocated for rounding errors.

**INITIAL_ACCOUNT_BALANCE: u128 = 100_000_000_000_000_000_000** The initial account balance for the contract.

# Tests 

## Unit Tests

None 

## Sandbox Tests

### test_usual_flow

Tests that the usual flow of a match works as expected. 

### test_error_at_future

Tests the contract behaves as expected when the match is cancelled in the future stage.

### test_error_at_current

Tests the contract behaves as expected when the match is cancelled in the current stage.

### test_admin_methods

Tests that a non admin cannot call admin methods and that admin switches correctly.

### test_wrong_ft

Tests that the contract rejects tokens that are not USDC.

### test_staking_system_usual_flow

Tests the usual flow of the staking system works as expected.

### test_large_bid

TODO

Tests the contract behaves as expected for very large bets.

