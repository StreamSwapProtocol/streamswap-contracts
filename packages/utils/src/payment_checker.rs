use cosmwasm_std::Coin;
use cosmwasm_std::StdError;
use cw_utils::NativeBalance;
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
    let mut expected_balance = NativeBalance::default();
    for coin in expected_funds {
        expected_balance += coin.clone();
    }
    expected_balance.normalize();

    let mut sent_balance = NativeBalance::default();
    for coin in sent_funds {
        sent_balance += coin.clone();
    }
    sent_balance.normalize();
    println!("expected_balance: {:?}", expected_balance);
    println!("sent_balance: {:?}", sent_balance);

    if expected_balance != sent_balance {
        return Err(CustomPaymentError::InsufficientFunds {
            expected: expected_funds.to_vec(),
            actual: sent_funds.to_vec(),
        });
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

        let sent_funds = vec![coin(1100, "uosmo")];
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
