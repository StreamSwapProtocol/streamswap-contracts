use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use streamswap_types::controller::Params;
use streamswap_types::stream::{Position, Stream};

pub const CONTROLLER_PARAMS: Item<Params> = Item::new("params");

pub const STREAM: Item<Stream> = Item::new("stream");

// Vesting (owner_addr) -> (contract_addr)
pub const VESTING: Map<Addr, Addr> = Map::new("vesting");

// Position (stream_id, owner_addr) -> Position
pub const POSITIONS: Map<&Addr, Position> = Map::new("positions");
