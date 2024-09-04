# betting-system

This repo holds the VEX betting smart contract.

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

- A stable contract (the last release) is deployed at "sexyvexycontract.testnet".

# Admin 

- The admin is set to vexsharedaccount.testnet with private key ed25519:3qB3Gehv1iuEXqjHRLm36HmHC7ZCKaQJfxFQFirUVjaBSXiohv4as6NtSQJmi86SoNXDaC2QR8mKFcCd52JPzeCB 

# Flow

## Betting flow 
1) The bettor selects a match and calls `ft_transfer_call` on the USDC contract which calls `ft_on_transfer` on the betting contract.
2) If the bet was successful or the match was cancelled the bettor calls `claim`.

## Admin / Oracle flow

1) When a new match needs to be added the admin calls `create_match`. 
2) When the match starts the admin calls `end_betting`. 
3) When the match finishes and the results are known the admin calls `finish_match`. 

- If there is a problem with a match the admin calls `cancel_match` between stages 1) and 3).

# Methods

## Change Methods

### ft_on_transfer

Used to place a bet on a match. Callable via [ft_transfer_call](https://docs.near.org/build/primitives/ft#attaching-fts-to-a-call).

**ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) U128** 

1) Checks that the token is USDC.
2) Checks they have bet one or more USDC.
3) Parses `msg`.
4) Fetches the match with the specified match ID and checks `match_state` is `Future`.
5) Calculates `potential_winnings` using `determine_potential_winnings`.
6) Adds bet amount to correct team's total bets.
7) Increments `last_bet_id`.
8) Inserts the a new `Bet` into into `bets_by_user`. 
9) Returns U128(0).   

- **sender_id: AccountId** The account ID of the bettor.
- **amount: U128** The bet amount in USDC. One whole USDC is 10^24.
- **msg: String** Stores the other information needed to place a bet in JSON in the format of `BetInfo`.

Returns the leftover USDC from the call, which will always be zero.

### claim

Used by a bettor to claim bet winnings or refund.

**claim_winnings(&mut self, bet_id: &BetID) -> U128**

1) Fetches the relevant bet from `bets_by_user`.
2) Checks that `pay_state` is `None`.
3) Checks that `match_state` is `Finished` or `Error`.
- If the match is `Finished`
    1) Checks that they selected the winning team.
    2) Transfers USDC equal to `potential_winnings` to the `bettor`.
    3) Changes `pay_state` to `Paid`.
    4) Returns `potential_winnings`.
- If the match is `Error`
    1) Transfers USDC equal to `bet_amount` to the `bettor`.
    2) Changes `pay_state` to `RefundPaid`.
    3) Returns `bet_amount`.

- **bet_id: &BetID** The bet ID of the bet the bettor is claiming their winnings for.

Returns the amount in USDC that the bettor receives.

## Only Callable by Admin 

### create_match

Used to create a new match.

**create_match(&mut self, game: String, team_1: String, team_2: String, in_odds_1: f64, in_odds_2: f64, date: String)**

1) Checks that the `admin` is calling the method.
2) Creates the match ID.
3) Determines the inital pool sizes by multiplying the inital odds of each team winning by the `WEIGHT_FACTOR`.
4) Creates a new match and adds it to `matches`.

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
3) Checks that the match has the `match_state` `Future`
4) Changes `match_state` to `Current`

- **match_id: &MatchID** The match ID of the match that betting is being ended for.

### finish_match

Used when a match finishes.

**finish_match(&mut self, match_id: &MatchID, winner: Team)**

1) Checks that the `admin` is calling the method.
2) Fetches the relevant match from `matches`.
3) Checks that the match has the `match_state` `Current`
4) Changes `match_state` to `Finished`.
5) Sets `winner`.

- **match_id: &MatchID** The match ID of the match that is finished.
- **winner: Team** The team that won the game

### cancel_match

Used when there is an error with a match or the match is cancelled.

**cancel_match(&mut self, match_id: MatchID)**

1) Checks that the `admin` is calling the method.
2) Fetches the relevant match from `matches`.
3) Checks whether the `match_state` is `Future` or `Current`.
4) Changes the `match_state` to `Error`.

- **match_id: MatchID** The match ID of the match that is being cancelled.

### change_admin

Used to change the admin of the betting contract.

**change_admin(&mut self, new_admin: AccountId)**

1) Checks that the `admin` is calling the method.
2) Changes `admin` to `new_admin`.

- **new_admin: AccountId** The account ID of the new admin.

## Only Callable by the Contract Account 

### init

Initializes the contract.

**init(&mut self, admin: AccountId)**

1) Sets the maps to empty.
2) Sets `last_bet_id` to 0.
3) Sets the `admin`.

## View Methods

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

### get_admin 

Fetches the admin of the contract.

**get_admin(&self) -> &AccountId**

1) Returns `admin`.

Returns the admin of the contract.

## Internal functions 

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

# Storage

## Structures

### Contract

The entry structure for the contract.

- **matches: IterableMap&lt;MatchId, Match&gt;** A map of matches yet to take place. 
- **bets_by_user: LookupMap&lt;AccountId, IterableMap&lt;BetId, Bet&gt;&gt;** A map that gives the bet IDs and the match ID of the match the bet was placed on.
- **last_bet_id: BetId** An integer that stores the bet ID of the last bet. Used for inputting what the next bet ID will be. 
- **admin: AccountID** Sets the account ID of the account that can call admin methods. The oracle will be the admin.


### Match

Stores the necessary information for a match.

- **game: String** What game the match is, e.g. Valorent, Overwatch, etc.
- **team_1: String** Name of team 1.
- **team_2: String** Name of team 2.
- **team_1_total_bets: U128** Total bets on team 1 in USDC, this includes initial weightings.
- **team_2_total_bets: U128** Total bets on team 2 in USDC, this includes initial weightings.
- **team_1_inital_pool: U128** Initial weightings adding to team 1's pool from initial odds.
- **team_2_inital_pool: U128** Initial weightings adding to team 2's pool from initial odds.
- **match_state: MatchState** An enumeration dictating what state the match is in.
- **winner: Option<Team>** An enumeration storing the winner of the match.

### Bet

Stores the necessary information for a bet.

- **match_id: MatchId** Match ID of the match they bet on.
- **team: Team** An enumeration storing the team they have bet on.
- **bet_amount: U128** The amount in USDC they bet on the match.
- **potential_winnings: U128** The amount the bettor will receive if they choose the correct team.
- **pay_state: Option&lt;PayState&gt;** An enumeration storing whether they have been paid out yet. If `None`, then either a winner is yet to be decided or the team they selected did not win.

### DisplayMatch

Stores the necessary information when fetching a match.

- **match_id: MatchID** The match ID of the match.
- **game: String** What game the match is, e.g. Valorent, Overwatch, etc.
- **team_1: String** Name of the first team.
- **team_2: String** Name of the second team.
- **team_1_real_bets: U128** Total actual bets placed on team 1.
- **team_2_real_bets: U128** Total actual bets placed on team 2.
- **match_state: MatchState** An enumeration dictating what state the match is in.
- **winner: Team** An enumeration storing the winner of the match.

### BetInfo

Stores the information needed to place a bet provided by `msg` from `ft_on_transfer`.

- **match_id: MatchID** Stores the match ID of the match the bettor is placing a bet on. 
- **team: Team** Stores which team the bettor is placing a bet on.

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

### test_large_bid

TODO

Tests the contract behaves as expected for very large bets.

