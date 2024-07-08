use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal256, Timestamp, Uint128};

#[cw_serde]
pub struct Position {
    /// Creator of the position.
    pub owner: Addr,
    /// Current amount of tokens in buy pool
    pub in_balance: Uint128,
    pub shares: Uint128,
    // Index is used to calculate the distribution a position has
    pub index: Decimal256,
    // Block time when the position was last updated.
    pub last_updated: Timestamp,
    // Total amount of `token_out` purchased in tokens at latest calculation
    pub purchased: Uint128,
    // Pending purchased accumulates purchases after decimal truncation
    pub pending_purchase: Decimal256,
    // Total amount of `token_in` spent tokens at latest calculation
    pub spent: Uint128,
    // Operator can update position
    pub operator: Option<Addr>,
}

impl Position {
    pub fn new(
        owner: Addr,
        in_balance: Uint128,
        shares: Uint128,
        index: Option<Decimal256>,
        operator: Option<Addr>,
        last_updated: Timestamp,
    ) -> Self {
        Position {
            owner,
            in_balance,
            shares,
            index: index.unwrap_or_default(),
            last_updated,
            purchased: Uint128::zero(),
            pending_purchase: Decimal256::zero(),
            spent: Uint128::zero(),
            operator,
        }
    }
}
