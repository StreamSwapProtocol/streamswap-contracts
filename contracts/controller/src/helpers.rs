use crate::error::ContractError;
use cosmwasm_std::Coin;
use streamswap_types::controller::CreatePool;
use streamswap_utils::to_uint256;

pub fn validate_create_pool(
    create_pool: Option<CreatePool>,
    out_asset: &Coin,
    in_denom: &str,
) -> Result<(), ContractError> {
    if let Some(create_pool) = create_pool {
        // pool cant be bigger than out_asset amount
        if create_pool.out_amount_clp > to_uint256(out_asset.amount) {
            return Err(ContractError::InvalidPoolOutAmount {});
        }
        if create_pool.msg_create_pool.denom0 != out_asset.denom {
            return Err(ContractError::InvalidPoolDenom {});
        }
        if create_pool.msg_create_pool.denom1 != in_denom {
            return Err(ContractError::InvalidPoolDenom {});
        }
    }
    Ok(())
}
