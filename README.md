# Streamswap

## Abstract

Streamswap is a new way and innovative way of selling token sale.
The mechanism allows anyone to create a new Sale event and sell any
amount of tokens in a more democratic way than the traditional solutions.

## Context

Since the first ICO boom, token sale mechanism was one of the driving
force for web3 onboarding.
Promise of a cheap tokens which can quickly accrue value is very attractive
for casual any sort of investors. Easy way of fundraising (the funding team)
opened doors for cohorts of new teams to focus on building on web3.

Traditional mechanisms of token sale included:

- Automated ICO, where team decides about the issuance price and the sale
  happens through a swap controlled by a smart contract.
- Regulated, centralized ICO - token sale controlled by a dedicated company,
  which will preform all operations using centralized services meeting
  regulatory requirements (KYC...). Example: Coinlist sales.
- Balancer style ICO: a novel solution to utilize Dutch Auction mechanism to
  find a fair strike price.

The first two mechanisms are not well suited for early stage startups, where
the token sale price is usually defined by a founding team and can't be
impacted by the ecosystem wisdom. False marketing actions are usually setup
to support their initial price.

The latter mechanism is not democratic - big entities can control the
price movements or place big orders leaving smaller investors with nothing.

## Design

### Stream Creation

Anyone can create a `Stream` by sending [`CreateStream`](https://github.com/osmosis-labs/osmosis/blob/robert%2Fstreamswap-spec/proto/osmosis/streamswap/v1/tx.proto#L21) transaction.
Treasury owner must send creation fee tokens and `in_denom` tokens to contract at `CreateStream`.
Creation fees will be collected at an adress later to be withdrawn after a sale finalizes.
Fee amount is determined by governance voting through sudo contract execution.

## Subscription

Anyone can join a sale by sending a [SubscribeMsg](https://github.com/osmosis-labs/osmosis/blob/main/proto/osmosis/streamswap/v1/tx.proto#L13) transaction.
When doing so, the transaction author has to send the `amount` he wants to spend in transaction funds.
That `amount` will be credited from tx author and pledged to the sale. Shares will be minted for the position owner.

At any time an investor can increase his participation for the sale by sending again `MsgSubscribe`
(his pledge will increase accordingly) or cancel it by sending
[`WithdrawMsg`](https://github.com/osmosis-labs/osmosis/blob/main/proto/osmosis/streamswap/v1/tx.proto#32).
When cancelling, the module will send back unspent pledged tokens to the investor
and keep the purchased tokens until the sale end_time.

`SubscribeMsg` can be submitted at any time after the sale is created and before it's end time.

From that moment, the investor will join the **token sale distribution stream**:

### Distribution

Stream distribution is done based on the total amount of shares and time.
On each `update_stream` call, the contract will calculate the amount to be distributed to the investors.

```
diff = (last_updated - now) / (end - last_updateed)
new_distribution_balance = stream.out_remaining * diff
spent_in = stream.in_supply * diff
stream.in_supply -= spent_in
stream.out_remaining -= new_distribution_balance
stream.current_streamed_price = spent_in / new_distribution_balance;
```

The `new_distribution_balance` will be distributed to shares.
Distribution index becomes this:

```
stream.dist_index += new_distribution_balance / stream.in_supply
```

### Purchase / Spending

Spend calculation happens when `update_position` is called. Distribution and spending are working as lazy accounting. Meaning
the calculations are done continuously, no action required. `update_distribution` and `update_position` just updates the current state of the stream and position.

When `update_position` is called, the contract will calculate the amount of tokens spent and  accumulated so far by the investor.
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

After the calculation, position balance, spent and purchased will be updated.

### Exit Stream

When participating in a sale, investors receives a stream of sale tokens.
These tokens are locked until sale end to avoid second market creating during
the sale. Once sale is finished (block time is after `stream.end_time`), every
investor can send [`ExitMsg`](https://github.com/osmosis-labs/osmosis/blob/main/proto/osmosis/streamswap/v1/tx.proto#L37)
to close his position, withdraw purchased tokens to his account and claim unspent in tokens.
On exit position data is removed to save space.

### Finalize Stream

// TODO: anyone or only treasury can finalize?
To withdraw earned token to the `stream.treasury` account anyone can send a
transaction with [`FinalizeMsg`](https://github.com/osmosis-labs/osmosis/blob/main/proto/osmosis/streamswap/v1/tx.proto#L42) after the `sale.end_time`.
Transaction will send `stream.spent_in` tokens to `stream.treasury` account.
This transaction will send `stream.spent_int` tokens from the contract
to the `sale.treasury`.

### Price

Average price of the sale: `stream.spent_in / stream.out_supply - stream.out_remaining`.
Last streamed price: spent_in / new_distribution_balance` at latest time of update distribution.

### Creation Fee

Creation fee is collected to prevent spams. Fee collection will be run during finalize stream.
Fee will be collected at `stream.creation_fee_address` which is the address of the multisig/distribution among parties
involved developing and maintaining the project.

## State

### Stream

`Stream` object represent a particular token sale/stream event and describes the main
required parameters conducting the stream process:
- `name`: name of the stream
- `url`: an external resource describing a sale. Can be IPFS link or a
  commonwealth post.
- `treasury`: address where the sale earnings will go. When the sale is over,
  anyone can trigger a [`MsgFinalizeSale`](https://github.com/osmosis-labs/osmosis/blob/main/proto/osmosis/streamswap/v1/tx.proto#L42)
  to clean up the sale state and move the earning to the treasury.
- `id`: unique identifier of the sale.
- `out_denom`: denom to sale (distributed to the investors).
  Also known as a base currency.
- `in_denom`: payment denom - used to buy `out_token`.
  Also known as quote currency.
- `out_supply`: total initial supply of `token_out` to sale.
- `in_supply`: total supply of in tokens at latest distribution.
- `spent_int`:total number of `token_in` spent at latest state
- `start_time`: unix timestamp when the stream starts.
- `end_time`: Unix timestamp when the stream ends.
- `current_streamed_price`: current price of the stream.
- `dist_index`: variable to hold the latest distribution index. Used to calculate how much proportionally
  a position holder is entitled to receive from the stream.
- `last_updated`: last updated time of stream
- `out_remaining`: total number of remaining out tokens at the time of update

### Position

`Position` object represents a particular position in a stream. It is created
when a user subsribed to a stream.
- `owner`: owner of the position.
- `stream_id`: id of the stream.
- `last_updated`: last updated time of position
- `shares`: number of shares of the position.
- `in_balance`: balance of `token_in` currently in the position.
- `index`: index of the position. Used to calculate incoming distribution belonging to the position
- `purchased`: total amount of `token_out` purchased in tokens at latest calculation
- `spent`: total amount of `token_in` spent tokens at latest calculation
- `operator`: operator of the position. Can be used to delegate position management to another account.

## Consequences

- The new sale mechanism provides a truly democratic way for token distribution and sale.
- It can be easily integrated with AMM pools: proceedings from the sale can
  automatically be pledged to AMM.

## Future directions

- providing incentive for sales with `OSMO` or `ATOM` used as a base currency.
- Basic DAO for distributing collected fees to maintainers of the project and decide fee distribution percentage.
