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

### Distribution

Anyone can join a sale by sending a [SubscribeMsg](https://github.com/osmosis-labs/osmosis/blob/main/proto/osmosis/streamswap/v1/tx.proto#L13) transaction.
When doing so, the transaction author has to send the `amount` he wants to spend in transaction funds.
That `amount` will be credited from tx author and pledged to the sale.

At any time an investor can increase his participation for the sale by sending again `MsgSubscribe`
(his pledge will increase accordingly) or cancel it by sending
[`WithdrawMsg`](https://github.com/osmosis-labs/osmosis/blob/main/proto/osmosis/streamswap/v1/tx.proto#32).
When cancelling, the module will send back unspent pledged tokens to the investor
and keep the purchased tokens until the sale end_time.

`SubscribeMsg` can be submitted at any time after the sale is created and before it's end time.

From that moment, the investor will join the **token sale distribution stream**:

Distribution stream is done based on the current stage of the stream.
On each `update_dist_index` call, the contract will calculate the current stage of the stream and determine the amount to be distributed to the investors.
Difference of the current stage and the last known stage will be used to calculate the amount to be distributed. Based on percentage.

```
current_stage = (now - start) / (end - start)
diff = current_stage - stream.current_stage
new_distribution_balance = stream.out_supply * diff
```

// TODO: requires review, not sure about this part.
In token spending calculation is based on the stage difference.

```
spent_in = stream.in_supply * diff
deducted_buy = stream.in_supply - spent_in
```

The `new_distribution_balance` will be distributed to the deducted tokens.
Distribution index becomes this:
```
stream.dist_index = stream.dist_index + (new_distribution_balance / deducted_buy)
```

### Purchase / Spending

Spending happens when `trigger_purchase` is called.
When a position is purchased, the contract will calculate the amount of `out_denom` tokens to be distributed to the investor.
Withdrawing and subscribing more tokens will trigger position purchase first.
Purchase will update distribution index first.

```
spent_diff = stream.current_stage - position.current_stage
spent = spent_diff * position.in_balance
position.in_balance -= spent

index_diff = stream.dist_index - position.dist_index
purchased = index_diff * position.in_balance
```

After the calculation, position balance, spent and purchased will be updated.

// TODO: we can use the previous design's formula for distribution and spending.
// (now - stream.current_stage) / (stream.end_time - stream.current_stage)

### Exit Stream

When participating in a sale, investors receives a stream of sale tokens.
These tokens are locked until sale end to avoid second market creating during
the sale. Once sale is finished (block time is after `stream.end_time`), every
investor can send [`ExitMsg`](https://github.com/osmosis-labs/osmosis/blob/main/proto/osmosis/streamswap/v1/tx.proto#L37)
to close his position, withdraw purchased tokens to his account and claim unspent in tokens.
Sets `position.exited` to `true`.

### Finalize Stream

// TODO: anyone or only treasury can finalize?
To withdraw earned token to the `stream.treasury` account anyone can send a
transaction with [`FinalizeMsg`](https://github.com/osmosis-labs/osmosis/blob/main/proto/osmosis/streamswap/v1/tx.proto#L42) after the `sale.end_time`.
Transaction will send `stream.spent_in` tokens to `stream.treasury` account.
This transaction will send `stream.spent_int` tokens from the contract
to the `sale.treasury`.

### Price

Average price of the sale is `stream.current_out / stream.spent_int`.
Last streamed price: `new_distribution_balance / spent_in_distribution` at the time of update distribution index.

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
- `current_stage`: current stage of the stream. %0-100, where 0 is the start
  of the stream and 100 is the end of the stream.
- `dist_index`: variable to hold the latest distribution index. Used to calculate how much proportionally
  a position holder is entitled to receive from the stream.

### Position

`Position` object represents a particular position in a stream. It is created
when a user subsribed to a stream.
- `owner`: owner of the position.
- `stream_id`: id of the stream.
- `in_balance`: balance of `token_in` currently in the position.
- `index`: index of the position. Used to calculate incoming distribution belonging to the position
- `current_stage`: latest stage when the position was updated. Used to calculate spent amount
- `purchased`: total amount of `token_out` purchased in tokens at latest calculation
- `spent`: total amount of `token_in` spent tokens at latest calculation
- `exited`: exited becomes true when position is finalized and tokens are sent to the owner.

## Consequences

- The new sale mechanism provides a truly democratic way for token distribution and sale.
- It can be easily integrated with AMM pools: proceedings from the sale can
  automatically be pledged to AMM.

## Future directions

- providing incentive for sales with `OSMO` or `ATOM` used as a base currency.
- Basic DAO for distributing collected fees to maintainers of the project and decide fee distribution percentage.
