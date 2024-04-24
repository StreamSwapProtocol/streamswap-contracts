use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal, Decimal256, Storage, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Params {
    pub admin: Addr,
    pub stream_creation_fee: Coin,
    pub exit_fee_percent: Decimal,
    pub stream_swap_code_id: u64,
}

pub const PARAMS: Item<Params> = Item::new("params");
