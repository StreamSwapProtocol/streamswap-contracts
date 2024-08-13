use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use streamswap_types::controller::Params;

pub const PARAMS: Item<Params> = Item::new("params");
pub const FREEZESTATE: Item<bool> = Item::new("freezestate");
pub const LAST_STREAM_ID: Item<u64> = Item::new("last_stream_id");
pub const STREAMS: Map<u64, Addr> = Map::new("streams");
