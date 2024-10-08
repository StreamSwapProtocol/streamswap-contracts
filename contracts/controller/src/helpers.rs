use std::str::FromStr;

use crate::error::ContractError;
use cosmwasm_std::{Coin, DepsMut, Uint128};
use osmosis_std::types::osmosis::poolmanager::v1beta1::PoolmanagerQuerier;

pub fn get_pool_creation_fee(deps: &DepsMut) -> Result<Vec<Coin>, ContractError> {
    let pool_creation_fee_vec = PoolmanagerQuerier::new(&deps.querier)
        .params()?
        .params
        .ok_or(ContractError::PoolCreationFeeNotFound {})?
        .pool_creation_fee;
    let mut cosmwasm_std_coin_vec = Vec::new();

    for coin in pool_creation_fee_vec.iter() {
        let amount = Uint128::from_str(&coin.amount.clone());
        match amount {
            Ok(amount) => {
                cosmwasm_std_coin_vec.push(Coin {
                    denom: coin.denom.clone(),
                    amount,
                });
            }
            Err(e) => return Err(e.into()),
        }
    }

    Ok(cosmwasm_std_coin_vec)
}
