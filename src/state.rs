use crate::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Storage, Uint128, Uint64};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub min_sale_duration: Uint64,
    pub min_duration_until_start_time: Uint64,
    pub sale_creation_denom: String,
    pub sale_creation_fee: Uint128,
    pub fee_collector: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct Sale {
    // Destination for the earned token_in
    pub treasury: Addr,
    // Proportional distribution variable to calculate the distribution of in token_out to buyers.
    pub dist_index: Decimal,
    // last calculated stage of sale, %0 -> %100
    pub current_stage: Decimal,
    // denom of the `token_out`
    pub out_denom: String,
    // total number of `token_out` to be sold during the continuous sale.
    pub out_supply: Uint128,
    // total number of `token_out` sold at latest state
    pub current_out: Uint128,
    // denom of the `token_in`
    pub in_denom: String,
    // total number of `token_in` on the buy side at latest state
    pub in_supply: Uint128,
    // total number of `token_in` spent at latest state
    pub current_in: Uint128,
    // TODO: convert to Timestamp
    // start time when the token emission starts. in nanos
    pub start_time: Uint64,
    // end time when the token emission ends. Can't be bigger than start +
    // 139years (to avoid round overflow)
    pub end_time: Uint64,
    // price at the time when distribution is triggered last
    pub latest_streamed_price: Uint128,
}

type SaleId = u64;
pub const SALES: Map<SaleId, Sale> = Map::new("sales");

const SALES_ID_COUNTER: Item<SaleId> = Item::new("sales_id_counter");

pub fn next_sales_id(store: &mut dyn Storage) -> Result<u64, ContractError> {
    let id: u64 = SALES_ID_COUNTER.may_load(store)?.unwrap_or_default() + 1;
    SALES_ID_COUNTER.save(store, &id)?;
    Ok(id)
}

#[cw_serde]
pub struct Position {
    // creator of the position
    pub owner: Addr,
    // current amount of tokens in buy pool
    pub in_balance: Uint128,
    // index is used to calculate the distribution a position has
    pub index: Decimal,
    pub latest_dist_stage: Decimal,
    // total amount of `token_out` purchased in tokens at latest calculation
    pub purchased: Uint128,
    // total amount of `token_in` spent tokens at latest calculation
    pub spent: Uint128,
    // finalized becomes true when position is finalized and tokens are sent to the recipient
    pub exited: bool,
}

// Position (sale_id, owner_addr) -> Position
pub const POSITIONS: Map<(SaleId, &Addr), Position> = Map::new("positions");
