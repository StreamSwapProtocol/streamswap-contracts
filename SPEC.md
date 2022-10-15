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
diff = current\_stage - state.last\_known\_stage
$$

$$
new\_distribution\_balance = diff \times token\_out\_supply
$$

$$
spent\_buy = diff \times total\_buy\_supply
$$

$$
deduced\_buy\_supply = total\_buy\_supply - spent\_buy
$$

$$
distribution\_index = distribution\_index + \frac{new\_distribution\_balance}{deduced\_buy\_supply}
$$



## Curve features

Curved distribution and spending, might create novel and different strategies for stream swapping.

For example: it is possible to distribute more in the initial phase of the sale. This could incentivize buyers to jump onboard early on.

