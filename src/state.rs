use cosmwasm_std::{Addr, Decimal, Uint128, Uint64};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    // Proportional distribution variable to calculate the distribution of in token_out to buyers.
    pub global_distribution_index: Decimal,
    // last calculated stage of sale, %0 -> %100
    pub latest_distribution_stage: Decimal,
    // denom of the `token_out`
    pub token_out_denom: String,
    // total number of `token_out` to be sold during the continuous sale.
    pub token_out_supply: Uint128,
    // total number of `token_out` sold at latest state
    pub total_out_sold: Uint128,
    // denom of the `token_in`
    pub token_in_denom: String,
    // total number of `token_in` on the buy side at latest state
    pub total_in_supply: Uint128,
    // total number of `token_in` spent at latest state
    pub total_in_spent: Uint128,
    // TODO: convert to scheduled
    // start time when the token emission starts. in nanos
    pub start_time: Uint64,
    // end time when the token emission ends. Can't be bigger than start +
    // 139years (to avoid round overflow)
    pub end_time: Uint64,
}
pub const STATE: Item<State> = Item::new("state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Position {
    // creator of the position
    pub owner: Addr,
    // current amount of tokens in buy pool
    pub buy_balance: Uint128,
    // index is used to calculate the distribution a position has
    pub index: Decimal,
    // total amount of purchased in tokens at latest calculation
    pub purchased: Uint128,
    // total amount of spent out tokens at latest calculation
    pub spent: Uint128
}

// Position (owner_addr) -> Position
pub const POSITIONS: Map<&Addr, Position> = Map::new("positions");

/*
/// list_accrued_rewards settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn list_positions(
    deps: Deps,
    start_after: Option<Addr>,
    limit: Option<u32>,
) -> StdResult<Vec<PositionResponse>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = calc_range_start(deps.api, start_after.map(Addr::unchecked))?.map(Bound::exclusive);

    POSITIONS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|elem| {
            let (addr, v) = elem?;
            Ok(PositionResponse {
                address: addr.to_string(),
                balance: v.buy_balance,
                index: v.index,
            })
        })
        .collect()
}

fn calc_range_start(api: &dyn Api, start_after: Option<Addr>) -> StdResult<Option<Vec<u8>>> {
    match start_after {
        Some(human) => {
            let mut v: Vec<u8> = api.addr_canonicalize(human.as_ref())?.0.into();
            v.push(0);
            Ok(Some(v))
        }
        None => Ok(None),
    }
}


 */
