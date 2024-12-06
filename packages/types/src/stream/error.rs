// use cosmwasm_std::StdError;
// use thiserror::Error;

// #[derive(Error, Debug, PartialEq)]
// pub enum ThresholdError {
//     #[error(transparent)]
//     Std(#[from] StdError),

//     #[error("Threshold not reached")]
//     ThresholdNotReached {},

//     #[error("Threshold reached")]
//     ThresholdReached {},

//     #[error("Threshold not set")]
//     ThresholdNotSet {},

//     #[error("Min price can't be zero")]
//     ThresholdZero {},
// }
