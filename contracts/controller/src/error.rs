use cosmwasm_std::{
    ConversionOverflowError, DivideByZeroError, Instantiate2AddressError, OverflowError, StdError,
};
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
    Instantiate2AddressError(#[from] Instantiate2AddressError),

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

    #[error("sync position")]
    SyncPosition {},
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

    #[error("Invalid decimals")]
    InvalidDecimals {},

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

    #[error("Invalid controller params")]
    InvalidControllerParams {},

    #[error("Invalid pool out amount")]
    InvalidPoolOutAmount {},

    #[error("Invalid pool denom")]
    InvalidPoolDenom {},

    #[error("Pool creation fee not found")]
    PoolCreationFeeNotFound {},

    #[error("Invalid terms and services")]
    InvalidToSVersion {},
}

impl From<ContractError> for StdError {
    fn from(err: ContractError) -> StdError {
        StdError::generic_err(err.to_string())
    }
}
