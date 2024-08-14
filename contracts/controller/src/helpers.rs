use std::str::FromStr;

use crate::error::ContractError;
use cosmwasm_std::{Coin, DepsMut, Uint128};
use osmosis_std::types::osmosis::poolmanager::v1beta1::PoolmanagerQuerier;
use streamswap_types::controller::CreatePool;
use streamswap_utils::to_uint256;

pub fn validate_create_pool(
    create_pool: CreatePool,
    out_asset: &Coin,
    in_denom: &str,
) -> Result<(), ContractError> {
    // pool cant be bigger than out_asset amount
    if create_pool.out_amount_clp > to_uint256(out_asset.amount) {
        return Err(ContractError::InvalidPoolOutAmount {});
    }
    // pool out amount cant be zero
    if create_pool.out_amount_clp.is_zero() {
        return Err(ContractError::InvalidPoolOutAmount {});
    }
    if create_pool.msg_create_pool.denom0 != out_asset.denom {
        return Err(ContractError::InvalidPoolDenom {});
    }
    if create_pool.msg_create_pool.denom1 != in_denom {
        return Err(ContractError::InvalidPoolDenom {});
    }
    Ok(())
}

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
