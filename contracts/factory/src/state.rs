use cw_storage_plus::Item;
use streamswap_types::factory::Params;

pub const PARAMS: Item<Params> = Item::new("params");
pub const FREEZESTATE: Item<bool> = Item::new("freezestate");
pub const LAST_STREAM_ID: Item<u64> = Item::new("last_stream_id");
