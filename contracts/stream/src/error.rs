use cosmwasm_std::{
    ConversionOverflowError, DivideByZeroError, Instantiate2AddressError, OverflowError, StdError,
    Uint256,
};
use cw_utils::PaymentError;
use std::convert::Infallible;
use streamswap_utils::payment_checker::CustomPaymentError;
use thiserror::Error;

use streamswap_types::stream::ThresholdError;

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
    ThresholdError(#[from] ThresholdError),

    #[error("{0}")]
    CustomPayment(#[from] CustomPaymentError),

    #[error("{0}")]
    ConversionOverflowError(#[from] ConversionOverflowError),
    #[error("No rewards accrued")]
    NoDistribution {},

    #[error("{0}")]
    Instantiate2AddressError(#[from] Instantiate2AddressError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Exit fee must be between 0 and 1")]
    InvalidExitFeePercent {},

    #[error("Subscriber already exited")]
    SubscriberAlreadyExited {},

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

    #[error("Withdraw amount exceeds user balance: {0}")]
    WithdrawAmountExceedsBalance(Uint256),

    #[error("Withdraw amount cannot be zero")]
    InvalidWithdrawAmount {},

    #[error("Invalid funds")]
    InvalidFunds {},

    #[error("Decrease amount exceeds user balance: {0}")]
    DecreaseAmountExceeds(Uint256),

    #[error("Wait for the unbonding")]
    WaitUnbonding {},

    #[error("No bond")]
    NoBond {},

    #[error("Stream not ended")]
    StreamNotEnded {},

    #[error("Stream is started update stream")]
    StreamIsStarted {},

    #[error("sync position")]
    UpdatePosition {},

    #[error("Stream duration is too short")]
    StreamDurationTooShort {},

    #[error("Stream duration is too long")]
    StreamDurationTooLong {},

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

    #[error("Stream not cancelled")]
    StreamNotCancelled {},

    #[error("Stream is cancelled")]
    StreamIsCancelled {},

    #[error("Stream killswitch is active")]
    StreamKillswitchActive {},

    #[error("Stream is already finalized")]
    StreamAlreadyFinalized {},

    #[error("Stream Name too short")]
    StreamNameTooShort {},

    #[error("Stream Name too long")]
    StreamNameTooLong {},

    #[error("Stream name is not in alphanumeric format")]
    InvalidStreamName {},

    #[error("Salt not provided for vesting creation")]
    InvalidSalt {},

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

    #[error("Invalid Bootstrapping start time")]
    StreamInvalidBootstrappingStartTime {},

    #[error("Stream Bootstrapping starts too soon")]
    StreamBootstrappingStartsTooSoon {},

    #[error("Stream not waiting")]
    StreamNotWaiting {},

    #[error("Stream bootrapping duration too short")]
    StreamBootstrappingDurationTooShort {},

    #[error("Stream waiting duration too short")]
    StreamWaitingDurationTooShort {},
}
