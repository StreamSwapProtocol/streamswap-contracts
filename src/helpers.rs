use crate::ContractError;
use cosmwasm_std::{Decimal256, StdError};
use std::str::FromStr;

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

pub fn check_name_and_url(name: &String, url: &Option<String>) -> Result<(), ContractError> {
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

pub fn from_semver(err: semver::Error) -> ContractError {
    ContractError::from(StdError::generic_err(format!("Semver: {}", err)))
}

use cosmwasm_std::{Coin, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum CustomPaymentError {
    #[error(transparent)]
    Std(#[from] StdError),
    #[error("Insufficient funds sent")]
    InsufficientFunds {
        expected: Vec<Coin>,
        actual: Vec<Coin>,
    },
}
pub fn check_payment(
    sent_funds: &[Coin],
    expected_funds: &[Coin],
) -> Result<(), CustomPaymentError> {
    // Remove 0 amounts
    let expected_funds = expected_funds
        .iter()
        .filter(|coin| coin.amount > Uint128::zero())
        .cloned() // Clone the elements
        .collect::<Vec<_>>();

    // Check length
    if sent_funds.len() > expected_funds.len() {
        return Err(CustomPaymentError::InsufficientFunds {
            expected: expected_funds.to_vec(),
            actual: sent_funds.to_vec(),
        });
    }

    let mut mut_sent_funds = sent_funds.to_vec(); // Create a mutable copy

    for expected in expected_funds.clone() {
        if let Some(sent_index) = mut_sent_funds
            .iter()
            .position(|sent| expected.denom == sent.denom)
        {
            let sent = &mut mut_sent_funds[sent_index];
            if expected.amount > sent.amount {
                return Err(CustomPaymentError::InsufficientFunds {
                    expected: expected_funds.to_vec(),
                    actual: sent_funds.to_vec(),
                });
            } else {
                sent.amount = sent.amount.checked_sub(expected.amount).unwrap();
            }
        } else {
            return Err(CustomPaymentError::InsufficientFunds {
                expected: expected_funds.to_vec(),
                actual: sent_funds.to_vec(),
            });
        }
    }

    Ok(())
}

// Test check_payment
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::coin;

    #[test]
    fn test_check_payment() {
        let sent_funds = vec![coin(100, "uosmo"), coin(100, "uusd")];
        let expected_funds = vec![coin(100, "uosmo"), coin(100, "uusd")];
        let res = check_payment(&sent_funds, &expected_funds);
        assert!(res.is_ok());

        let sent_funds = vec![coin(100, "uosmo"), coin(100, "uusd")];
        let expected_funds = vec![coin(100, "uosmo")];
        let res = check_payment(&sent_funds, &expected_funds);
        assert!(res.is_err());

        let sent_funds = vec![coin(100, "uosmo")];
        let expected_funds = vec![coin(100, "uosmo"), coin(100, "uusd")];
        let res = check_payment(&sent_funds, &expected_funds);
        assert!(res.is_err());

        let sent_funds = vec![coin(100, "uosmo"), coin(100, "uusd")];
        let expected_funds = vec![coin(100, "uosmo"), coin(200, "uusd")];
        let res = check_payment(&sent_funds, &expected_funds);
        assert!(res.is_err());

        let sent_funds = vec![coin(300, "uosmo")];
        let expected_funds = vec![coin(100, "uosmo"), coin(200, "uosmo")];
        let res = check_payment(&sent_funds, &expected_funds);
        assert!(res.is_ok());

        let sent_funds = vec![coin(300 - 1, "uosmo")];
        let expected_funds = vec![coin(100, "uosmo"), coin(200, "uosmo")];
        let res = check_payment(&sent_funds, &expected_funds);
        assert!(res.is_err());

        let sent_funds = vec![coin(300, "uosmo"), coin(100, "uusd")];
        let expected_funds = vec![coin(300, "uosmo"), coin(200, "uatom")];
        let res = check_payment(&sent_funds, &expected_funds);
        assert!(res.is_err());

        let sent_funds = vec![coin(1100, "uosmo")];
        let expected_funds = vec![
            coin(100, "uosmo"),
            coin(200, "uosmo"),
            coin(300, "uosmo"),
            coin(500, "uosmo"),
        ];
        let res = check_payment(&sent_funds, &expected_funds);
        assert!(res.is_ok());
        let sent_funds = vec![coin(1100 + 1, "uosmo")];
        let expected_funds = vec![
            coin(100, "uosmo"),
            coin(200, "uosmo"),
            coin(300, "uosmo"),
            coin(500, "uosmo"),
        ];
        let res = check_payment(&sent_funds, &expected_funds);
        assert!(res.is_ok());

        let sent_funds = vec![coin(1100 - 1, "uosmo")];
        let expected_funds = vec![
            coin(100, "uosmo"),
            coin(200, "uosmo"),
            coin(300, "uosmo"),
            coin(500, "uosmo"),
        ];
        let res = check_payment(&sent_funds, &expected_funds);
        assert!(res.is_err());

        let sent_funds = vec![coin(1100, "uosmo")];
        let expected_funds = vec![coin(0, "something"), coin(1100, "uosmo")];
        let res = check_payment(&sent_funds, &expected_funds);
        assert!(res.is_ok());
    }
}
