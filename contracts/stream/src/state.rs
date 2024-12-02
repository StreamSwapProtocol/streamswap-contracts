use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use streamswap_types::controller::Params;
use streamswap_types::stream::{Position, PostStreamActions, StreamInfo, StreamState};

pub const CONTROLLER_PARAMS: Item<Params> = Item::new("params");

// Stream State Related data
pub const STREAM_STATE: Item<StreamState> = Item::new("ss");

// Stream information
pub const STREAM_INFO: Item<StreamInfo> = Item::new("si");

// Post Stream Action Related Information
pub const POST_STREAM: Item<PostStreamActions> = Item::new("ps");

pub const TOS: Item<String> = Item::new("tos");

// Subscriber Vesting (owner_addr) -> (contract_addr)
pub const SUBSCRIBER_VESTING: Map<Addr, Addr> = Map::new("sub_vest");

// Creator Vesting (owner_addr) -> (contract_addr)
pub const CREATOR_VESTING: Map<Addr, Addr> = Map::new("cr_vest");

// Position (stream_id, owner_addr) -> Position
pub const POSITIONS: Map<&Addr, Position> = Map::new("positions");

/// Terms and services ipfs link signature signed by user
/// Both for creator and subscriber
pub const TOS_SIGNED: Map<&Addr, String> = Map::new("tos_signed");
