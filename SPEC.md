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
current\_stage = \frac{now - start}{end - start}
$$

$$
diff = current\_stage - last\_known\_stage
$$

$$
new\_distribution\_balance = diff \times token\_out\_supply
$$

$$
distribution\_index = distribution\_index + \frac{new\_distribution\_balance}{total\_buy}
$$



Each position state holds a `position_index` value which represents the current state of release a position has.

On `trigger_purchase` calls, `position_index` is caught up to `distribution_index`
$$
diff = distribution\_index - position\_index
$$

$$
purchased = diff \times position\_balance
$$

## Spending

In the initial design spending happens linearly.

Ex: 10000 token is by the user and 0-10000 is start and end unix times.

Spending graph looks like this:

![Screen Shot 2022-09-29 at 18.04.05](/Users/orkunkulce/Library/Application Support/typora-user-images/Screen Shot 2022-09-29 at 18.04.05.png)

Ex: At the half time, half of the token will be spend.

Spent amount is not deduced from positions balance until `withdraw` is triggered.



It is very possible to implement a curve spending feature in future.

## Curve features

Curved distribution and spending, might create novel and different strategies for stream swapping.

For example: it is possible to distribute more in the initial phase of the sale. This could incentivize buyers to jump onboard early on.

Or 
