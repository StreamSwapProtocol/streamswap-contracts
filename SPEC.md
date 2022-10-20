# Stream Swap index based spec

In this spec, relative index design is used.

The distribution logic is based on reward distribution smart contract from [anchor project](https://github.com/Anchor-Protocol/anchor-bAsset-contracts/tree/master/contracts/anchor_basset_reward)

Distribution is done over time not based on rounds. Distribution precision is higher.

Ex: 100000 tokens will be distributed.

Distribution starts at: 1000 unix time and ends at 2000 unix time.

at 1300 unix, %30 percent will be distributed.

at 1600 unix, %60.

It is possible to make the distribution using curves, not only in linear.

## Distribution

On each `update_distribution_index` call, difference between the previous call and current call is handed out to position stakers.

Whole distribution state is saved under `distribution_index`

On `update_distribution_index` call `distribution_index` is updated with given calculation.
$$
current\_distribution\_stage = \frac{now - start}{end - start}
$$


$$
diff = current\_dist\_stage - state.latest\_dist\_stage
$$

$$
new\_distribution\_balance = diff \times state.token\_out\_supply
$$

$$
spent\_buy = diff \times state.total\_buy\_supply
$$

$$
deduced\_buy\_supply = state.total\_buy\_supply - spent\_buy
$$

$$
state.global\_dist\_index = state.global\_dist\_index + \frac{new\_dist\_balance}{deduced\_buy\_supply}
$$

After this calculation 

current distribution stage is saved to state as global_dist_index

current calculated stage is saved to state as latest_dist_stage

## Trigger position purchase 

Before a position purchase global distribution index is updated, total buy supply updated with spent amount.

Before withdraw and subscribe position purchase is triggered.

position.latest_stage is the percentage of the current stage of the sale like %5 or %50

spent calculation is done by calculating the stage change after latest action. Let's say user subscribed some tokens at %10 of the sale. And he withdrawed it at %50 of the sale. This means that subscribed tokens will be deduced by %40.

If a user subscribed at %0 and did not withdraw or add any tokens, at %100 all of his tokens will be sold.

If the user subscribes at %0 and adds more tokens at %30, at the new subsription action %30 percent of the tokens he loaded will deduced and stream will continue with `(initial_token_amount * 30/100) + new_token_amount`

Here is the position purchase algorithm that calculates the purchased and sold amount.
$$
index\_diff = state.global\_distribution\_index - position.index
$$

$$
purchased =  index\_diff \times position.buy\_balance
$$

$$
spent\_diff = state.latest\_dist\_stage - position.latest\_dist\_stage
$$

$$
spent = spent\_diff \times position.buy\_balance
$$

$$
position.index = state.global\_dist\_index
$$



## Curve features

Curved distribution and spending, might create novel and different strategies for stream swapping.

For example: it is possible to distribute more in the initial phase of the sale. This could incentivize buyers to jump onboard early on.

