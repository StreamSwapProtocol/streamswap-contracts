// use crate::stream::{StreamState, ThresholdError};
// use cosmwasm_std::{StdError, Storage, Uint256};
// use cw_storage_plus::Item;

// pub type Threshold = Uint256;

// pub const THRESHOLDS_STATE_KEY: &str = "thresholds";

// pub struct ThresholdState<'a>(Item<'a, Threshold>);

// impl<'a> ThresholdState<'a> {
//     pub fn new() -> Self {
//         ThresholdState(Item::new(THRESHOLDS_STATE_KEY))
//     }

//     pub fn set_threshold_if_any(
//         &self,
//         threshold: Option<Uint256>,
//         storage: &mut dyn Storage,
//     ) -> Result<(), ThresholdError> {
//         match threshold {
//             Some(threshold) => {
//                 if threshold.is_zero() {
//                     return Err(ThresholdError::ThresholdZero {});
//                 }
//                 self.0.save(storage, &threshold)?;
//                 Ok(())
//             }
//             None => Ok(()),
//         }
//     }
//     pub fn error_if_not_reached(
//         &self,
//         storage: &dyn Storage,
//         state: &StreamState,
//     ) -> Result<(), ThresholdError> {
//         // If threshold is not set, It returns ok
//         // If threshold is set, It returns error if threshold is not reached
//         let threshold = self.0.may_load(storage)?;
//         if let Some(threshold) = threshold {
//             if state.spent_in < threshold {
//                 Err(ThresholdError::ThresholdNotReached {})
//             } else {
//                 Ok(())
//             }
//         } else {
//             Ok(())
//         }
//     }

//     pub fn error_if_reached(
//         &self,
//         storage: &dyn Storage,
//         state: &StreamState,
//     ) -> Result<(), ThresholdError> {
//         let threshold = self.0.may_load(storage)?;
//         if let Some(threshold) = threshold {
//             if state.spent_in >= threshold {
//                 Err(ThresholdError::ThresholdReached {})
//             } else {
//                 Ok(())
//             }
//         } else {
//             Ok(())
//         }
//     }
//     pub fn check_if_threshold_set(&self, storage: &dyn Storage) -> Result<bool, ThresholdError> {
//         let threshold = self.0.may_load(storage)?;
//         Ok(threshold.is_some())
//     }
//     pub fn get_threshold(&self, storage: &dyn Storage) -> Result<Option<Threshold>, StdError> {
//         let threshold = self.0.may_load(storage)?;
//         Ok(threshold)
//     }
// }

// impl<'a> Default for ThresholdState<'a> {
//     fn default() -> Self {
//         ThresholdState::new()
//     }
// }
