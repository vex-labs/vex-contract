# betting-system

This repo holds the VEX betting smart contract.

# Contriubting

> [!Caution]
> Do **NOT** push directly to or merge your own commits into main. This repo has no branch protection rules so no warning will displayed if you try to do this.

To contribute:
1) Open a new branch with an appropriate name and select main as the source.
2) Commit your code to your branch.
3) Create a PR to main from your branch.
4) If your contribution solves an issue please write "Closes #issue-number" in the PR.
5) Select Owen (PiVortex) as a reviewer.

If you need any help with this please don't hesitate to ask for help.


# Flow

## Betting flow 
1) The bettor selects a match and calls `ft_transfer_call` on the USDC contract which calls `ft_on_transfer` on the betting contract.
2) If the bet was successful the bettor calls `claim_winnings`, if the match was canceled the bettor calls `claim_refund`.

## Admin / Oracle flow

1) When a new match needs to be added the admin calls `create_match`. 
2) When the match starts the admin calls `end_betting`. 
3) When the match finishes and the results are known the admin calls `finish_match`. 

- If there is a problem with a match the admin calls `cancel_match` between stages 1) and 3).

# Methods

## Change Methods

### ft_on_transfer

Used to place a bet on a match.

**ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> PromiseOrValue&lt;U128&gt;** 

1) Checks that the token is USDC.
2) Decerializes `msg`.
3) Fetches the match with the specified match ID.
4) Inserts a new `Bet` into `bets` for the specified match with the correct bet details and reinserts the match.
5) Inserts the `BetId` and `MatchId` into `bets_by_user`. 
6) Returns U128(0).   

- **sender_id: AccountId** The account ID of the bettor.
- **amount: U128** The bet amount in USDC.
- **msg: String** Stores the other information needed to place a bet in JSON in the format of 
`BetInfo`.

Returns the leftover USDC from the call, which will always be zero.

### claim_winnings

Used by a bettor to claim bet winnings.

**claim_winnings(&mut self, match_id: MatchID, bet_id: BetID) -> U128**

1) Fetches the relevant bid from the relevant match.
2) Checks that `match_state` is `Finished`.
3) Checks that the predecessor is equal to `bettor`.
4) Checks that `pay_state` is `None`
5) Transfers USDC equal to `potential_winnings` to the `bettor`.
6) Changes `pay_state` to `Paid`.
7) Returns `potential_winnings`.

- **match_id: MatchID** The match ID of the match for which the bet was placed on.
- **bet_id: BetID** The bet ID of the bet the bettor is claiming their winnings for.

Returns the amount in USDC that the bettor receives.

### claim_refund

Used by a bettor to claim bet refunds when a match has an error or is canceled.

**claim_refund(&mut self, match_id: MatchID, bet_id: BetID) -> U128**

1) Fetches the relevant bid from the relevant match.
2) Checks that `match_state` is `Error`.
3) Checks that the predecessor is equal to `bettor`.
4) Checks that `pay_state` is `None`.
5) Transfers USDC equal to `bet_amount` to the `bettor`.
6) Changes `pay_state` to `RefundPaid`.
7) Returns `bet_amount`.

- **match_id: MatchID** The match ID of the match for which the bet was placed on.
- **bet_id: BetID** The bet ID of the bet the bettor is claiming their refund for.

Returns the amount in USDC that the bettor receives.


## Only Callable by Admin 

### create_match

Used to create a new match.

**create_match(&mut self, game: String, team_1: String, team_2: String, in_odds_1: f64, in_odds_2: f64, date: String)**

1) Checks that the `admin` is calling the method.
2) Creates the match ID.
3) Determines the initial odds of the game by adjusting the in odds to be a total implied probability of 105%.
4) Dertermines the inital pool sizes by multiplying the inital probability of each team winning by the `weight_factor`.
5) Creates a new match and adds it to `matches`.

- **game: String** What game the match is, e.g. Valorent, Overwatch, etc.
- **team_1: String** Name of the first team.
- **team_2: String** Name of the second team.
- **in_odds_1: f64** Average external odds for team 1 to win.
- **in_odds_2: f64** Average external odds for team 2 to win.
- **date: String** The date the match is taking place.


### end_betting

Used to close betting on the match.

**end_betting(&mut self, match_id: MatchID)**

1) Checks that the `admin` is calling the method.
2) Fetches the relevant match from `matches`.
3) Checks that the match has the `match_state` `Future`
4) Changes `match_state` to `Current`

- **match_id: MatchID** The match ID of the match that betting is being ended for.

### finish_match

Used when a match finishes.

**finish_match(&mut self, match_id: MatchID)**

1) Checks that the `admin` is calling the method.
2) Fetches the relevant match from `matches`.
3) Checks that the match has the `match_state` `Current`
4) Changes `match_state` to `Finished`.
5) Changes `winner`.

- **match_id: MatchID** The match ID of the match that is finished.


### cancel_match

Used when there is an error with a match or the match is canceled.

**cancel_match(&mut self, match_id: MatchID)**

1) Checks that the `admin` is calling the method.
2) Fetches the relevant match from `matches`.
3) Checks whether the `match_state` is `Future` or `Current`.
4) Changes the `match_state` to `Error`.

- **match_id: MatchID** The match ID of the match that is being canceled.

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

**get_matches(&self, from_index: Option&lt;U64&gt;, limit: Option&lt;U64>) -> Vec&lt;DisplayMatch>**

1) If `from_index` is `None` set to 0 and if `limit` is `None` then it is set to the length of `matches`.
2) Iterate through `matches`.
3) For each `Match` convert to `DisplayMatch` using `format_match`.
4) Add each `DisplayMatch` to a vector.
5) Return the vector.

- **from_index: Option&lt;U64>** Specifies the index at which the method will start iterating.
- **limit: Option&lt;U64>** Specifies how many matches will be collected.

Returns a vector of matches to display.

### get_match

Fetches a single match.

**get_match(&self, match_id: MatchID) -> DisplayMatch**

1) Fetches the relevant match from `matches`.
2) Converts from `Match` to `DisplayMatch` using `format_match`.
3) Returns the match.

- **match_id: MatchID** The match ID of the match to be fetched.

Returns a single instance of `DisplayMatch`.

### get_potential_winnings

Gets the amount in USDC the bettor would receive if they were to make a bet right now. 

**get_potential_winnings(&self, match_id: MatchID, team: Team, bet_amount: U128) -> U128**

1) Fetches the relevant match from `matches`.
2) Checks which team they have picked.
3) Gets the potential winnings via `determine_potential_winnings`.
4) Return the potential winnings.

- **match_id: MatchID** The match ID of the match the bettor would bet on.
- **team: Team** The team that the bettor would bet on.
- **bet_amount: U128** The amount in USDC the bettor would bet.

Returns the potential winnings.

### get_bet

Fetches a single bet.

**get_bet(&self, match_id: MatchID, bet_id: BetID) -> Bet** 

1) Fetches the relevant match from `matches`.
2) Fetches the relevant bet from `bets`.
3) Returns the bet.

- **match_id: MatchID** The match ID of the match where the bet was placed.
- **bet_id: BetID** The bet ID of the bet to be fetched.

Returns a single instance of `Bet`.

### get_users_bets

Fetches a vector of bet Ids and their associated match IDs within a limit for a single bettor.

**get_users_bets(&self, bettor: AccountID, from_index: Option&lt;U64&gt;, limit: Option&lt;U64&gt;) -> Vec&lt;BetId, MatchId&gt;**

1) If `from_index` is `None` set to 0 and if `limit` is `None` then it is set to the number of bets for the bettor.
2) Iterate through the map of bet IDs.
3) For each `BetId` add `BetId` and `MatchId` to a vector.
4) Return the vector.

- **from_index: Option&lt;U64&gt;** Specifies the index at which the method will start iterating.
- **limit: Option&lt;U64&gt;** Specifies how many bet IDs will be fetched.

Returns a vector of BetIds and their associated match IDs.

### get_admin 

Fetches the admin of the contract.

**get_admin(&self) -> AccountId**

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

**determine_potential_winnings(selected_team_total_bets: U128, other_team_total_bet: U128, bet_amount: U128) -> U128**

1) Calculates potential winnings for the given arguments.
2) Returns the potential winnings.

- **selected_team_total_bets: U128** Total bets on the selected team in USDC, this includes initial weightings.
- **other_team_total_bets: U128** Total bets on the non-selected team in USDC, this includes initial weightings.
- **bet_amount: U128** The bet amount in USDC for the bet.

Returns the potential winnings in USDC for a bet.

### format_match

Reformats a match's details from `Match` to `DisplayMatch`. 

**format_match(Match) -> DisplayMatch**

1) Reformats from `Match` to `DisplayMatch`.
2) Returns an instance of `DisplayMatch`.

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
- **bets_by_user: LookupMap&lt;AccountId, IterableMap&lt;BetId, MatchId&gt;&gt;** A map that gives the bet IDs and the match ID of the match the bet was placed on.
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
- **winner: Team** An enumeration storing the winner of the match.
- **bets: IterableMap&lt;BetID, Bet&gt;** A map of bets made on the match.


### Bet

Stores the necessary information for a bet.

- **bettor: AccountId** Account ID of the account making the bet.
- **team: Team** An enumeration storing the team they have bet on.
- **bet_amount: U128** The amount in USDC they bet on the match.
- **potential_winnings: U128** The amount the bettor will receive if they chose the correct team.
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
- **Error** The match had an error or was canceled. 

## Type Aliases

**MatchId: String** A combination of team names and the date that the match is set to take place in the form "team1-team2-dd/mm/yyyy".

**BetId: U64** A unique identifier for a single bet across the whole contract.

## Constants

**WEIGHT_FACTOR: f64** Sets the weight of the initial odds. If this is higher then the odds will change less on user bets, more so initially. Initially set to 1000. 

# Uncertainties and considerations

- If I am fetching a match will it fetch the list of bets and be high gas cost? Can I push a bet to matches without loading all the bets?
- Is it better to have all the instances of Bet not stored in a Match and instead just list the bet IDs and Bets live independently of matches? I think it's best to have it nested (as it is right now) as when I am accessing a bet it will be related to a match unless it is a view method.
- Check how much storage is being used to place a bet, there may need to be restrictions on this as people can just place a million bets and use all the storage in the contract. For now, set the minimum bet to 1 USDC.
- Currently, matches and their associated bets stay in the contract forever which uses a lot of storage. Consider deleting matches. Can we just index historical data instead?
- get_matches currently fetches all types of matches, change to input the MatchStatus to get matches in certain statuses.
- Do I need bets_by_user if I'm using an indexer?
- Take a look at using U128 instead of f64 for odds, look how price oracles do it.
