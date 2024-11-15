use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal256, Timestamp, Uint128};
use cw_storage_plus::Map;

use crate::state::StreamId;

#[cw_serde]
pub struct PositionV0_1_0 {
    /// creator of the position.
    pub owner: Addr,
    /// current amount of tokens in buy pool
    pub in_balance: Uint128,
    pub shares: Uint128,
    // index is used to calculate the distribution a position has
    pub index: Decimal256,
    pub last_updated: Timestamp,
    // total amount of `token_out` purchased in tokens at latest calculation
    pub purchased: Uint128,
    // pending purchased accumulates purchases after decimal truncation
    pub pending_purchase: Decimal256,
    // total amount of `token_in` spent tokens at latest calculation
    pub spent: Uint128,
    // operator can update position
    pub operator: Option<Addr>,
}
pub const OLD_POSITIONS: Map<(StreamId, Addr), PositionV0_1_0> = Map::new("positions");
