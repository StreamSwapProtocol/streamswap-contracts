use cosmwasm_std::{
    CheckedFromRatioError, ConversionOverflowError, Decimal, DivideByZeroError, Fraction,
    OverflowError, StdError, Storage, Uint128,
};
use cw_storage_plus::Map;
use std::convert::Infallible;
use thiserror::Error;

use crate::state::Stream;

pub type Threshold = Uint128;

#[derive(Error, Debug, PartialEq)]
pub enum ThresholdError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("{0}")]
    Infallible(#[from] Infallible),

    #[error("{0}")]
    DivideByZeroError(#[from] DivideByZeroError),

    #[error("{0}")]
    ConversionOverflowError(#[from] ConversionOverflowError),

    #[error("{0}")]
    CheckedFromRatioError(#[from] CheckedFromRatioError),

    #[error("Threshold not reached")]
    ThresholdNotReached {},

    #[error("Threshold reached")]
    ThresholdReached {},

    #[error("Threshold not set")]
    ThresholdNotSet {},
}
pub const THRESHOLDS_STATE_KEY: &str = "thresholds";

pub struct ThresholdState<'a>(Map<'a, u64, Threshold>);

impl<'a> ThresholdState<'a> {
    pub fn new() -> Self {
        ThresholdState(Map::new(THRESHOLDS_STATE_KEY))
    }
    pub fn set_treshold_if_any(
        &self,
        stream: Stream,
        stream_id: u64,
        storage: &mut dyn Storage,
        swap_fee_percent: Decimal,
    ) -> Result<(), ThresholdError> {
        match stream.min_price {
            Some(min_price) => {
                let out_supply = stream.out_supply.u128();
                // We should also include our swap fee percent to final threshold
                // Say creator wants to sell 1000 out_tokens, and swap fee is 0.3%
                // If creator is aiming to get 30_000 in_tokens, total threshold should be 30_000 + 0.3% of 30_000
                let target_price =
                    min_price.checked_div(Decimal::one().checked_sub(swap_fee_percent)?)?;
                let decimal_threshold = target_price
                    .checked_mul(Decimal::from_ratio(
                        Uint128::from(out_supply),
                        Uint128::one(),
                    ))?
                    .floor();
                let threshold =
                    Uint128::from(decimal_threshold.numerator() / decimal_threshold.denominator());
                self.0.save(storage, stream_id, &threshold)?;
                Ok(())
            }
            None => Ok(()),
        }
    }
    pub fn error_if_not_reached(
        &self,
        stream_id: u64,
        storage: &dyn Storage,
        spent_in: Uint128,
    ) -> Result<(), ThresholdError> {
        let state = self.0.may_load(storage, stream_id)?;
        if let Some(state) = state {
            if spent_in < state {
                Err(ThresholdError::ThresholdNotReached {})
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    pub fn error_if_reached(
        &self,
        stream_id: u64,
        storage: &dyn Storage,
        spent_in: Uint128,
    ) -> Result<(), ThresholdError> {
        let state = self.0.may_load(storage, stream_id)?;
        if let Some(state) = state {
            if spent_in >= state {
                Err(ThresholdError::ThresholdReached {})
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
    pub fn error_if_treshold_not_set(
        &self,
        stream_id: u64,
        storage: &dyn Storage,
    ) -> Result<(), ThresholdError> {
        let state = self.0.may_load(storage, stream_id)?;
        if state.is_none() {
            Err(ThresholdError::ThresholdNotReached {})
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::state::Stream;
    use cosmwasm_std::testing::MockStorage;
    use cosmwasm_std::{Addr, Decimal256, Uint128};

    #[test]
    fn test_thresholds_state() {
        let mut storage = MockStorage::new();
        let thresholds = ThresholdState::new();
        let stream = Stream {
            min_price: Some(Decimal::from_str("0.1").unwrap()),
            out_supply: Uint128::new(1000),
            in_supply: Uint128::new(1000),
            start_block: 0,
            end_block: 100,
            current_streamed_price: Decimal::percent(100),
            dist_index: Decimal256::one(),
            in_denom: "uusd".to_string(),
            last_updated_block: 0,
            name: "test".to_string(),
            url: Some("test".to_string()),
            out_denom: "uluna".to_string(),
            out_remaining: Uint128::new(1000),
            pause_block: None,
            shares: Uint128::new(0),
            spent_in: Uint128::new(0),
            status: crate::state::Status::Active,
            stream_creation_denom: "uusd".to_string(),
            stream_creation_fee: Uint128::new(0),
            stream_exit_fee_percent: Decimal::percent(0),
            treasury: Addr::unchecked("treasury"),
        };
        let stream_id = 1;

        thresholds
            .set_treshold_if_any(
                stream.clone(),
                stream_id,
                &mut storage,
                Decimal::from_str("0.042").unwrap(),
            )
            .unwrap();

        let result = thresholds.error_if_not_reached(stream_id, &storage, Uint128::new(0));
        assert_eq!(result.is_err(), true);

        let result = thresholds.error_if_not_reached(stream_id, &storage, Uint128::new(103));
        assert_eq!(result.is_err(), true);

        let result = thresholds.error_if_not_reached(stream_id, &storage, Uint128::new(104));
        assert_eq!(result.is_ok(), true);

        let mut new_stream = stream.clone();
        new_stream.min_price = Some(Decimal::from_str("14.37").unwrap());
        new_stream.out_supply = Uint128::new(100_000_000_000);

        let stream_id = 2;
        thresholds
            .set_treshold_if_any(
                new_stream.clone(),
                stream_id,
                &mut storage,
                Decimal::from_str("0.042").unwrap(),
            )
            .unwrap();

        let result = thresholds.error_if_not_reached(stream_id, &storage, Uint128::new(0));

        assert_eq!(result.is_err(), true);

        let result =
            thresholds.error_if_not_reached(stream_id, &storage, Uint128::new(100_000_000_000));

        assert_eq!(result.is_ok(), true);
    }
}
