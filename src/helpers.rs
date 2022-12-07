use crate::ContractError;
use cosmwasm_std::Decimal256;
use std::str::FromStr;

// calculate the reward with decimal
pub fn get_decimals(value: Decimal256) -> Result<Decimal256, ContractError> {
    let stringed: &str = &*value.to_string();
    let parts: &[&str] = &*stringed.split('.').collect::<Vec<&str>>();
    match parts.len() {
        1 => Ok(Decimal256::zero()),
        2 => {
            let decimals = Decimal256::from_str(&*("0.".to_owned() + parts[1]))?;
            Ok(decimals)
        }
        _ => Err(ContractError::InvalidDecimals {}),
    }
}
