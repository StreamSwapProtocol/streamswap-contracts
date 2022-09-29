# Stream Swap index based spec

In this spec, relative index design is used.

The code is based on reward distribution smart contract from anchor project.

Distribution is done over time not based on rounds. Distribution precision is higher.

Ex: 100000 tokens will be distributed.

Distribution starts at: 1000 unix time and ends at 2000 unix time.

at 1300 unix, %30 percent will be distributed.

at 1600 unix, %60.

## Distribution

On each `update_distribution_index` call, difference between the previous call and current call is handed out to position stakers.

Whole distribution state is saved under `distribution_index`



During `update_distribution_index` call `distribution_index` is updated with given calculation.
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

Initially spending will happen linearly.

It is possible to implement curve spending in future.



