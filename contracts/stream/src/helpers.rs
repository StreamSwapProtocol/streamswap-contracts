use crate::ContractError;
use cosmwasm_std::{BankMsg, Coin, CosmosMsg, Decimal256, Timestamp, Uint128, Uint256};
use std::str::FromStr;
use streamswap_types::controller::Params as ControllerParams;

/// Stream validation related constants
const MIN_NAME_LENGTH: usize = 2;
const MAX_NAME_LENGTH: usize = 64;
const MIN_URL_LENGTH: usize = 12;
const MAX_URL_LENGTH: usize = 128;

/// Special characters that are allowed in stream names and urls
const SAFE_TEXT_CHARS: &str = "<>$!&?#()*+'-./\"";
const SAFE_URL_CHARS: &str = "-_:/?#@!$&()*+,;=.~[]'%";

// calculate the reward with decimal
pub fn get_decimals(value: Decimal256) -> Result<Decimal256, ContractError> {
    let stringed: &str = &value.to_string();
    let parts: &[&str] = &stringed.split('.').collect::<Vec<&str>>();
    match parts.len() {
        1 => Ok(Decimal256::zero()),
        2 => {
            let decimals = Decimal256::from_str(&("0.".to_owned() + parts[1]))?;
            Ok(decimals)
        }
        _ => Err(ContractError::InvalidDecimals {}),
    }
}

pub fn check_name_and_url(name: &str, url: &Option<String>) -> Result<(), ContractError> {
    if name.len() < MIN_NAME_LENGTH {
        return Err(ContractError::StreamNameTooShort {});
    }
    if name.len() > MAX_NAME_LENGTH {
        return Err(ContractError::StreamNameTooLong {});
    }
    if !name.chars().all(|c| {
        c.is_ascii_alphanumeric() || c.is_ascii_whitespace() || SAFE_TEXT_CHARS.contains(c)
    }) {
        return Err(ContractError::InvalidStreamName {});
    }

    if let Some(url) = url {
        if url.len() < MIN_URL_LENGTH {
            return Err(ContractError::StreamUrlTooShort {});
        }
        if url.len() > MAX_URL_LENGTH {
            return Err(ContractError::StreamUrlTooLong {});
        }
        if !url
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || SAFE_URL_CHARS.contains(c))
        {
            return Err(ContractError::InvalidStreamUrl {});
        }
    }
    Ok(())
}

// Function to validate stream times
pub fn validate_stream_times(
    now: Timestamp,
    bootstrapping_start_time: Timestamp,
    start_time: Timestamp,
    end_time: Timestamp,
    params: &ControllerParams,
) -> Result<(), ContractError> {
    if now > bootstrapping_start_time {
        return Err(ContractError::StreamInvalidBootstrappingStartTime {});
    }

    if bootstrapping_start_time > start_time {
        return Err(ContractError::StreamInvalidBootstrappingStartTime {});
    }

    if start_time > end_time {
        return Err(ContractError::StreamInvalidEndTime {});
    }
    let stream_duration = end_time
        .seconds()
        .checked_sub(start_time.seconds())
        .ok_or(ContractError::StreamInvalidEndTime {})?;

    if stream_duration < params.min_stream_duration {
        return Err(ContractError::StreamDurationTooShort {});
    }
    let bootstrapping_duration = start_time
        .seconds()
        .checked_sub(bootstrapping_start_time.seconds())
        .ok_or(ContractError::StreamInvalidStartTime {})?;
    if bootstrapping_duration < params.min_bootstrapping_duration {
        return Err(ContractError::StreamBootstrappingDurationTooShort {});
    }

    let waiting_duration = bootstrapping_start_time
        .seconds()
        .checked_sub(now.seconds())
        .ok_or(ContractError::StreamInvalidBootstrappingStartTime {})?;
    if waiting_duration < params.min_waiting_duration {
        return Err(ContractError::StreamWaitingDurationTooShort {});
    }
    Ok(())
}

pub fn build_u128_bank_send_msg(
    denom: String,
    to_addr: String,
    amount: Uint256,
) -> Result<CosmosMsg, ContractError> {
    let u128_amount = Uint128::try_from(amount)?;
    let revenue_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: to_addr,
        amount: vec![Coin {
            denom: denom,
            amount: u128_amount,
        }],
    });
    Ok(revenue_msg)
}
