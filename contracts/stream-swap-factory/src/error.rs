use cosmwasm_std::{ConversionOverflowError, DivideByZeroError, OverflowError, StdError};
use cw_utils::PaymentError;
use std::convert::Infallible;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    Infallible(#[from] Infallible),

    #[error("{0}")]
    DivideByZeroError(#[from] DivideByZeroError),

    #[error("{0}")]
    ConversionOverflowError(#[from] ConversionOverflowError),

    #[error("Invalid exit fee percent")]
    InvalidExitFeePercent {},
}
