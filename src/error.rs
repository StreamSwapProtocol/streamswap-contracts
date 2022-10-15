use cosmwasm_std::{OverflowError, StdError, Uint128};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("Start and end dates should be same type")]
    DateInput {},

    #[error("No rewards accrued")]
    NoDistribution {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Do not send native funds")]
    NoFundsSent {},

    #[error("Amount required")]
    AmountRequired {},

    #[error("Decrease amount exceeds user balance: {0}")]
    DecreaseAmountExceeds(Uint128),

    #[error("Wait for the unbonding")]
    WaitUnbonding {},

    #[error("No bond")]
    NoBond {},
}
