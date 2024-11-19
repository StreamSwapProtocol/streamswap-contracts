use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use streamswap_types::controller::Params;
use streamswap_types::stream::{Position, Stream};

pub const CONTROLLER_PARAMS: Item<Params> = Item::new("params");

pub const STREAM: Item<Stream> = Item::new("stream");

// Subscriber Vesting (owner_addr) -> (contract_addr)
pub const SUBSCRIBER_VESTING: Map<Addr, Addr> = Map::new("sub_vest");
// Creator Vesting (owner_addr) -> (contract_addr)
pub const CREATOR_VESTING: Map<Addr, Addr> = Map::new("cr_vest");

// Position (stream_id, owner_addr) -> Position
pub const POSITIONS: Map<&Addr, Position> = Map::new("positions");
