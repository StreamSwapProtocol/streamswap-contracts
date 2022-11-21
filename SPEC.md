# Stream Swap index based spec

In this spec, relative index design is used.

The distribution logic is based on reward distribution smart contract
from [anchor project](https://github.com/Anchor-Protocol/anchor-bAsset-contracts/tree/master/contracts/anchor_basset_reward)

Distribution is done over time.
Ex: 100000 tokens will be distributed.
Distribution starts at: 1000 unix time and ends at 2000 unix time.
at 1300 unix, %30 percent will be distributed.
at 1600 unix, %60.

It is possible to make the distribution using curves, not only linear.

## Subscription

On subscription, user loads tokens into the smart contract and increasing the shares they own in the pool.

```rust
// compute amount of shares that should be minted for a new subscription amount
pub fn compute_shares_amount(&self, amount_in: Uint128, round_up: bool) -> Uint128{
  if self.shares.is_zero() || amount_in.is_zero() {
    return amount_in
  }
  let mut shares= self.shares.mul(amount_in);
  if round_up {
    shares = (shares + self.in_supply - Uint128::one()) / self.in_supply;
  } else {
    shares = shares / self.in_supply;
  }
  shares
}
```

## Distribution

On each `update_distribution_index` call, difference between the previous call and current call is handed out to
positions.
Whole distribution state is saved under `distribution_index`
On `update_distribution_index` call triggers distribution.

```
current_stage = (now - start) / (end - start)
diff = current_stage - stream.current_stage
new_distribution_balance = diff * stream.out_supply
spent_in = diff * stream.in_supply
deduced_in_supply = stream.in_supply - spent_buy
stream.dist_index = stream.dist_index + new_distribution_balance / stream.shares
```

## Trigger position purchase

Before a position purchase global distribution index is updated, total buy supply updated with spent amount.

Before withdraw and subscribe position purchase is triggered.

position.latest_stage is the percentage of the current stage of the sale like %5 or %50

spent calculation is done by calculating the stage change after latest action. Let's say user subscribed some tokens at
%10 of the sale. And he withdraws it at %50 of the sale. This means that subscribed tokens will be deduced by %40.

If a user subscribed at %0 and did not withdraw or add any tokens, at %100 all of his tokens will be sold.

If the user subscribes at %0 and adds more tokens at %30, at the new subscription action %30 percent of the initial tokens
will deduce and stream will continue with `(initial_token_amount * 30/100) + new_token_amount`

Here is the position purchase algorithm that calculates the purchased and sold amount.

```
index_diff = stream.dist_index - position.index
purchased = index_diff * position.shares
spent_diff = stream.current_dist_stage - position.latest\_dist\_stage
spent = spent_diff * position.in_balance
position.index = stream.dist_index
```

## Curve features

Curved distribution and spending, might create novel and different strategies for stream swapping.

For example: it is possible to distribute more in the initial phase of the sale. This could incentivize buyers to jump
onboard early on.
