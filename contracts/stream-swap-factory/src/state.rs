use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal, Decimal256, Storage, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Params {
    pub protocol_admin: Addr,
    pub fee_collector: Addr,
    pub stream_creation_fee: Coin,
    pub exit_fee_percent: Decimal,
    pub stream_swap_code_id: u64,
    pub accepted_in_denoms: Vec<String>,
    pub min_stream_blocks: u64,
    pub min_blocks_until_start_block: u64,
}

pub const PARAMS: Item<Params> = Item::new("params");
pub const FREEZESTATE: Item<bool> = Item::new("freezestate");
pub const LAST_STREAM_ID: Item<u64> = Item::new("last_stream_id");
