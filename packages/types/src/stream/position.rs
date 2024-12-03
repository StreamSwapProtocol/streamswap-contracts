use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal256, Timestamp, Uint256};

#[cw_serde]
pub struct Position {
    /// Creator of the position.
    pub owner: Addr,
    /// Current amount of tokens in buy pool
    pub in_balance: Uint256,
    pub shares: Uint256,
    // Index is used to calculate the distribution a position has
    pub index: Decimal256,
    // Block time when the position was last updated.
    pub last_updated: Timestamp,
    // Total amount of `token_out` purchased in tokens at latest calculation
    pub purchased: Uint256,
    // Pending purchased accumulates purchases after decimal truncation
    pub pending_purchase: Decimal256,
    // Total amount of `token_in` spent tokens at latest calculation
    pub spent: Uint256,
    // Exit date of the position
    pub exit_date: Timestamp,
}

impl Position {
    pub fn new(
        owner: Addr,
        in_balance: Uint256,
        shares: Uint256,
        index: Option<Decimal256>,
        last_updated: Timestamp,
    ) -> Self {
        Position {
            owner,
            in_balance,
            shares,
            index: index.unwrap_or_default(),
            last_updated,
            purchased: Uint256::zero(),
            pending_purchase: Decimal256::zero(),
            spent: Uint256::zero(),
            exit_date: Timestamp::from_nanos(0),
        }
    }
}
