use cosmwasm_std::{ConversionOverflowError, DivideByZeroError, OverflowError, StdError, Uint128};
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

    #[error("Cannot migrate from different contract type: {previous_contract}")]
    CannotMigrate { previous_contract: String },

    #[error("No rewards accrued")]
    NoDistribution {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Exit fee must be between 0 and 1")]
    InvalidExitFeePercent {},

    #[error("Required denom not found in funds")]
    NoFundsSent {},

    #[error("In_denom does not match config")]
    InDenomIsNotAccepted {},

    #[error("Supplied funds do not match out_supply")]
    StreamOutSupplyFundsRequired {},

    #[error("Decrease amount exceeds user balance: {0}")]
    DecreaseAmountExceeds(Uint128),

    #[error("Wait for the unbonding")]
    WaitUnbonding {},

    #[error("No bond")]
    NoBond {},

    #[error("Stream not ended")]
    StreamNotEnded {},

    #[error("Update dist index")]
    UpdateDistIndex {},

    #[error("Update position")]
    UpdatePosition {},

    #[error("Stream duration is too short")]
    StreamDurationTooShort {},

    #[error("Stream starts too soon")]
    StreamStartsTooSoon {},

    #[error("Invalid start time")]
    StreamInvalidStartTime {},

    #[error("Invalid end time")]
    StreamInvalidEndTime {},

    #[error("Creation fee amount do not match the supplied funds")]
    StreamCreationFeeRequired {},

    #[error("Stream Ended")]
    StreamEnded {},

    #[error("Stream not started")]
    StreamNotStarted {},

    #[error("Invalid decimals")]
    InvalidDecimals {},

    #[error("Stream paused")]
    StreamPaused {},

    #[error("Stream is already paused")]
    StreamAlreadyPaused {},

    #[error("Stream not paused")]
    StreamNotPaused {},

    #[error("Stream not cancelled")]
    StreamNotCancelled {},

    #[error("Stream is cancelled")]
    StreamIsCancelled {},

    #[error("Stream is either paused or cancelled")]
    StreamKillswitchActive {},
}
