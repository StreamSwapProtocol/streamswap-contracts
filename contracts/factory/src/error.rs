use cosmwasm_std::{ConversionOverflowError, DivideByZeroError, OverflowError, StdError};
use cw_denom::DenomError;
use cw_utils::PaymentError;
use std::convert::Infallible;
use streamswap_utils::payment_checker::CustomPaymentError;
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
    CustomPayment(#[from] CustomPaymentError),

    #[error("{0}")]
    Infallible(#[from] Infallible),

    #[error("{0}")]
    DivideByZeroError(#[from] DivideByZeroError),

    #[error("{0}")]
    ConversionOverflowError(#[from] ConversionOverflowError),

    #[error("{0}")]
    DenomError(#[from] DenomError),

    #[error("Invalid exit fee percent")]
    InvalidExitFeePercent {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("No rewards accrued")]
    NoDistribution {},

    #[error("Required denom not found in funds")]
    NoFundsSent {},

    #[error("In_denom does not match config")]
    InDenomIsNotAccepted {},

    #[error("Out_denom can not be the same as in_denom")]
    SameDenomOnEachSide {},

    #[error("Out supply must be greater than zero")]
    ZeroOutSupply {},

    #[error("Supplied funds do not match out_supply")]
    StreamOutSupplyFundsRequired {},

    #[error("Withdraw amount cannot be zero")]
    InvalidWithdrawAmount {},

    #[error("Invalid funds")]
    InvalidFunds {},

    #[error("Wait for the unbonding")]
    WaitUnbonding {},

    #[error("No bond")]
    NoBond {},

    #[error("Stream not ended")]
    StreamNotEnded {},

    #[error("Stream is started update stream")]
    StreamIsStarted {},

    #[error("Update position")]
    UpdatePosition {},

    #[error("Stream duration is too short")]
    StreamDurationTooShort {},

    #[error("Stream duration is too long")]
    StreamDurationTooLong {},

    #[error("Stream starts too soon")]
    StreamStartsTooSoon {},

    #[error("Stream bootstrapping starts too soon")]
    StreamBootstrappingStartsTooSoon {},

    #[error("Invalid start time ")]
    StreamInvalidStartTime {},

    #[error("Invalid boothstrapping start time ")]
    StreamInvalidBootstrappingStartTime {},

    #[error("Invalid end time ")]
    StreamInvalidEndTime {},

    #[error("Creation fee amount do not match the supplied funds")]
    StreamCreationFeeRequired {},

    #[error("Stream Ended")]
    StreamEnded {},

    #[error("Stream not started")]
    StreamNotStarted {},

    #[error("Invalid decimals")]
    InvalidDecimals {},

    #[error("Stream not cancelled")]
    StreamNotCancelled {},

    #[error("Stream is cancelled")]
    StreamIsCancelled {},

    #[error("Stream killswitch is active")]
    StreamKillswitchActive {},

    #[error("Stream is already finalized")]
    StreamAlreadyFinalized {},

    #[error("Contract is frozen")]
    ContractIsFrozen {},

    #[error("Stream Name too short")]
    StreamNameTooShort {},

    #[error("Stream Name too long")]
    StreamNameTooLong {},

    #[error("Stream name is not in alphanumeric format")]
    InvalidStreamName {},

    #[error("Stream URL too short")]
    StreamUrlTooShort {},

    #[error("Stream URL too long")]
    StreamUrlTooLong {},

    #[error("Stream URL is not properly formatted or contains unsafe characters")]
    InvalidStreamUrl {},

    #[error("Invalid stream creation fee")]
    InvalidStreamCreationFee {},

    #[error("Invalid exit fee")]
    InvalidStreamExitFee {},

    #[error("Invalid factory params")]
    InvalidFactoryParams {},
}

impl From<ContractError> for StdError {
    fn from(err: ContractError) -> StdError {
        StdError::generic_err(err.to_string())
    }
}
