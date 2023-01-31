# Streamswap

## Abstract

Streamswap is a new way and innovative way of having a token sale.
The mechanism allows anyone to create a new Sale event and sell any
amount of tokens in a more democratic way than the traditional solutions.

## Context

Since the first ICO boom, token sale mechanisms were one of the driving
force for web3 onboarding.
The promise of cheap tokens which can quickly accrue value is very attractive
to casual any sort of investors. Easy way of fundraising (the funding team)
opened doors for cohorts of new teams to focus on building on web3.

Traditional mechanisms of token sale included:

- Automated ICO, where the team decides the issuance price, and the sale
  happens through a swap controlled by a smart contract.
- Regulated, centralized ICO - token sale controlled by a dedicated company,
  which will perform all operations using centralized services meeting
  regulatory requirements (KYC...). Example: Coinlist sales.
- Balancer style ICO: a novel solution to utilize the Dutch Auction mechanism to
  find a fair strike price.

The first two mechanisms are not well suited for early-stage startups, where
the token sale price is usually defined by a founding team and can't be
impacted by the ecosystem's wisdom. False marketing actions are usually set up
to support their initial price.

The latter mechanism is not democratic - big entities can control the
price movements by placing big orders leaving smaller investors with nothing.

Stream swap is a new mechanism which allows anyone to create a new sale event where the sale happens continuously over a period of time.
You can imagine it as two flasks of liquid, mixing together over time, reaching equilibrium.

The price is determined on the fly as sell/buy balance fluctuate as buy supply increases with incoming subscriptions.
Sell side is distributed among the subscribers, by the proportion of their subscription amount to the total subscription amount.
Buy side is spent with respect to remaining end date of the sale event.
Example: A position subscribed at %80 percent of the sale will be fully spent at the end of the sale.

## Design

### Stream Creation

Anyone can create a `Stream` by sending [`CreateStream`](https://github.com/osmosis-labs/osmosis/blob/robert%2Fstreamswap-spec/proto/osmosis/streamswap/v1/tx.proto#L21) transaction.
Treasury owner must send creation fee tokens and `in_denom` tokens to contract at `CreateStream`.
Creation fees will be collected at an address later to be withdrawn after a sale finalizes.
The fee amount is determined by governance voting through sudo contract execution.

## Subscription

Anyone can join a sale by sending a [SubscribeMsg](https://github.com/osmosis-labs/osmosis/blob/main/proto/osmosis/streamswap/v1/tx.proto#L13) transaction.
When doing so, the transaction author has to send the `amount` he wants to spend in transaction funds.
That `amount` will be credited from the tx author and pledged to the sale.
`SubscribeMsg` can be submitted at any time after the sale is created and before its end time.
New shares will be minted to the owner of the position.
Share calculation works in this manner:

```
if shares == 0 || amount_in == 0 {
    return amount_in
}
new_shares = stream.shares * amount_in;
if round_up {
    new_shares = (new_shares + in_supply - 1) / in_supply
} else {
    new_shares = new_shares / in_supply
}
return new_shares
```

From that moment, the investor will join the **token sale distribution stream**:

At any time an investor can increase his participation in the sale by sending again `SubscribeMsg`
(his pledge will increase accordingly) or cancel it by sending `WithdrawMsg`. When canceling, the module will send back
unspent pledged tokens to the investor and keep the purchased tokens until the sale end_time.


### Distribution

Stream distribution is done based on the total amount of shares and time.
On each `update_stream` call, the contract will calculate the amount to be distributed to the investors.

```
diff = (stream.last_updated - now) / (stream.end - stream.last_updated)
new_distribution_balance = stream.out_remaining_token * diff
spent_in = stream.in_token_supply * diff
stream.in_supply -= spent_in
stream.out_remaining -= new_distribution_balance
stream.current_streamed_price = spent_in / new_distribution_balance
```

The `new_distribution_balance` will be distributed to shares.
Distribution index becomes this:

```
stream.dist_index += new_distribution_balance / shares
```

### Purchase / Spending

Spend calculation happens when `update_position` is called. Distribution and spending are working as lazy accounting. Meaning
the calculations are done continuously, with no action required. `update_stream` and `update_position` updates the current state of the stream and position.

When `update_position` is called, the contract will calculate the amount of tokens spent and accumulated so far by the investor.
Update position updates distribution index first.

```
index_diff = stream.dist_index - position.index;
purchased = position.shares * index_diff;
in_remaining = stream.in_supply * position.shares / stream.shares;
spent = position.in_balance - in_remaining;

position.spent += spent;
position.in_balance = in_remaining;
position.purchased += purchased;
position.index = stream.dist_index;
```

After the calculation, the position balance, spent amount, and purchased amount will be updated.

### Exit Stream

When participating in a sale, investors receive a stream of sale tokens.
These tokens are locked until the sale ends to avoid second market creation during
the sale. Once the sale is finished (block time is after `stream.end_time`), every
investor can send [`ExitMsg`](https://github.com/osmosis-labs/osmosis/blob/main/proto/osmosis/streamswap/v1/tx.proto#L37)
to close his position, withdraw purchased tokens to his account, and claim unspent tokens.
Before exiting both stream and position are updated for calculating the amount of position spent/bought.
On exit, the position data is removed to save space.

An exit fee applied on bought. User gets deducted `exit_fee` amount of tokens from his purchased amount.

### Finalize Stream

To withdraw earned tokens to the `stream.treasury` account anyone can send a
transaction with [`FinalizeMsg`](https://github.com/osmosis-labs/osmosis/blob/main/proto/osmosis/streamswap/v1/tx.proto#L42) after the `sale.end_time`.
Transaction will send `stream.spent_in` tokens to `stream.treasury` account.
This transaction will send `stream.spent_int` tokens from the contract
to the `sale.treasury`.
On finalize stream, exit fee on whole sale is applied to bought tokens and sent to fee collector address.

### Price

Average price of the sale: `stream.spent_in / (stream.out_supply - stream.out_remaining)`.
Last streamed price: `spent_in / new_distribution_balance` at the latest time of `update_stream`.

### Creation Fee

A creation fee is collected to prevent spams. Fee collection will be run during finalize stream.
The fee will be collected at `stream.creation_fee_address` which is the address of the multisig/distribution among parties
involved in developing and maintaining the project.

## DAO

We intend to use the DAO to govern the contract. The DAO will be able to change the fee amount, fee collector address, and exit fee amount.
Collected fees will be distributed to the DAO treasury to compansate people's effort and ensure project is funded for future development.
Deployment of the project is done through governance. This makes the owner of the contract to be the governance.

// TODO: Add message details
### Stream

`Stream` object represents a particular token sale/stream event and describes the main
required parameters conducting the streaming process:
- `name`: name of the stream
- `url`: an external resource describing a sale. Can be an IPFS link or a
  commonwealth post.
- `treasury`: the address where the sale earnings will go. When the sale is over,
  anyone can trigger a [`MsgFinalizeSale`](https://github.com/osmosis-labs/osmosis/blob/main/proto/osmosis/streamswap/v1/tx.proto#L42)
  to clean up the sale state and move the earnings to the treasury.
- `id`: the unique identifier of the sale.
- `out_denom`: the denom to sale (distributed to the investors).
  Also known as a base currency.
- `in_denom`: payment denom - used to buy `out_token`.
  Also known as the quote currency.
- `out_supply`: total initial supply of `token_out` to sale.
- `in_supply`: total supply of in tokens at latest distribution.
- `spent_in`:total number of `token_in` spent at latest state
- `start_time`: Unix timestamp when the stream starts.
- `end_time`: Unix timestamp when the stream ends.
- `current_streamed_price`: current price of the stream.
- `dist_index`: variable to hold the latest distribution index. Used to calculate how much proportionally
  a position holder is entitled to receive from the stream.
- `last_updated`: last updated time of stream
- `out_remaining`: total number of remaining out tokens at the time of update
- `shares`: total number of shares in the stream

### Position

`Position` object represents a particular position in a stream. It is created
when a user subscribes to a stream.
- `owner`: owner of the position.
- `last_updated`: last updated time of position
- `shares`: number of shares of the position.
- `in_balance`: balance of `token_in` currently in the position.
- `index`: index of the position. Used to calculate incoming distribution belonging to the position
- `purchased`: the total amount of `token_out` purchased in tokens at the latest calculation
- `spent`: the total amount of `token_in` spent tokens at the latest calculation
- `operator`: the operator of the position. Can be used to delegate position management to another account.
- `pending_purchase`: Accumulated decimals of position.purchased on update_position.

## Consequences

- The new sale mechanism provides a truly democratic way for token distribution and sale.
- It can be easily integrated with AMM pools: proceedings from the sale can
  automatically be pledged to AMM.

## Future directions

- Providing incentives for sales with `OSMO` or `ATOM` used as a base currency.
- Basic DAO for distributing collected fees to maintainers of the project and deciding on the fee distribution percentage.
