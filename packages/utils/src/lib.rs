use cosmwasm_std::{Uint128, Uint256};

pub mod payment_checker;
pub mod payment;

pub fn to_uint256(value: Uint128) -> Uint256 {
    Uint256::from(value.u128())
}
