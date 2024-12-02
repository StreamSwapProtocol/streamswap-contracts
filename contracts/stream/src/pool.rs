use std::str::FromStr;

use crate::ContractError;
use cosmwasm_std::{attr, Attribute, Coin, CosmosMsg, Decimal256, DepsMut, Env, Uint128, Uint256};
use osmosis_std::types::osmosis::concentratedliquidity::v1beta1::MsgCreatePosition;
use osmosis_std::types::osmosis::poolmanager::v1beta1::PoolmanagerQuerier;
use osmosis_std::types::osmosis::concentratedliquidity::poolmodel::concentrated::v1beta1::MsgCreateConcentratedPool;
use streamswap_types::controller::{CreatePool, PoolConfig};
use streamswap_types::stream::StreamState;
use streamswap_utils::to_uint256;

pub fn pool_operations(deps: &DepsMut, env: &Env, create_pool: CreatePool, mut stream_state: &mut StreamState, messages: &mut Vec<CosmosMsg>, attributes: &mut Vec<Attribute>, mut creator_revenue: Uint256, pool_config: PoolConfig) -> Result<(), ContractError> {
    match (pool_config, create_pool) {
        (
            Some(PoolConfig::ConcentratedLiquidity { out_amount_clp }),
            Some(CreatePool::ConcentratedLiquidity {
                     lower_tick,
                     upper_tick,
                     tick_spacing,
                     spread_factor,
                 }),
        ) => {
            let pool_id = next_pool_id(&deps)?;

            // amount of in tokens allocated for clp
            let in_clp = calculate_in_amount_clp(
                to_uint256(stream_state.out_asset.amount),
                out_amount_clp,
                creator_revenue,
            );

            // extract in_clp from last revenue
            creator_revenue = creator_revenue.checked_sub(in_clp)?;

            // Create initial position message
            let create_initial_position_msg = build_create_initial_position_msg(
                pool_id,
                env.contract.address.to_string(),
                stream_state.in_denom.clone(),
                in_clp,
                stream_state.out_asset.denom.clone(),
                out_amount_clp,
                lower_tick,
                upper_tick,
            );

            // convert msg create pool to osmosis create clp pool msg
            let osmosis_create_clp_pool_msg = MsgCreateConcentratedPool {
                sender: env.contract.address.to_string(),
                denom0: stream_state.out_asset.denom.clone(),
                denom1: stream_state.in_denom.clone(),
                tick_spacing,
                spread_factor: spread_factor.clone(),
            };

            messages.push(osmosis_create_clp_pool_msg.into());
            messages.push(create_initial_position_msg.into());

            attributes.push(attr("pool_id", pool_id.clone().to_string()));
            attributes.push(attr("pool_type", "clp".to_string()));
            attributes.push(attr("pool_out_amount", out_amount_clp));
            attributes.push(attr("pool_in_amount", in_clp));
            attributes.push(attr("pool_lower_tick", lower_tick.to_string()));
            attributes.push(attr("pool_upper_tick", upper_tick.to_string()));
            attributes.push(attr("pool_spread_factor", spread_factor.to_string()));
            attributes.push(attr("pool_tick_spacing", tick_spacing.to_string()));
            Ok(())
        }
        (None, None) => Ok(()),
        // If either pool_config or create_pool is not set, return error
        _ => Err(ContractError::InvalidPoolConfig {}),
    }
}

/// This function is used to calculate the in amount of the pool
pub fn calculate_in_amount_clp(
    out_amount: Uint256,
    pool_out_amount: Uint256,
    creators_revenue: Uint256,
) -> Uint256 {
    let ratio = Decimal256::from_ratio(pool_out_amount, out_amount);
    let dec_creators_revenue = Decimal256::from_ratio(creators_revenue, Uint256::from(1u64));
    let dec_clp_amount = ratio * dec_creators_revenue;
    dec_clp_amount * Uint256::from(1u64)
}

/// This function is used to build the MsgCreatePosition for the initial pool position
pub fn build_create_initial_position_msg(
    pool_id: u64,
    sender: String,
    stream_in_denom: String,
    in_clp: Uint256,
    stream_out_asset_denom: String,
    pool_out_amount_clp: Uint256,
    lower_tick: i64,
    upper_tick: i64,
) -> MsgCreatePosition {
    MsgCreatePosition {
        pool_id,
        sender,
        lower_tick,
        upper_tick,
        tokens_provided: vec![
            osmosis_std::types::cosmos::base::v1beta1::Coin {
                denom: stream_out_asset_denom.to_string(),
                amount: pool_out_amount_clp.to_string(),
            },
            osmosis_std::types::cosmos::base::v1beta1::Coin {
                denom: stream_in_denom.to_string(),
                amount: in_clp.to_string(),
            },
        ],
        token_min_amount0: "0".to_string(),
        token_min_amount1: "0".to_string(),
    }
}

pub fn next_pool_id(deps: &DepsMut) -> Result<u64, ContractError> {
    // query the number of pools to get the pool id
    let current_num_of_pools = PoolmanagerQuerier::new(&deps.querier)
        .num_pools()?
        .num_pools;
    let pool_id = current_num_of_pools + 1;
    Ok(pool_id)
}

pub fn get_pool_creation_fee(deps: &DepsMut) -> Result<Vec<Coin>, ContractError> {
    let pool_creation_fee_vec = PoolmanagerQuerier::new(&deps.querier)
        .params()?
        .params
        .unwrap()
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
#[cfg(test)]
mod pool_test {
    use super::*;
    use osmosis_std::types::cosmos::base::v1beta1::Coin;

    #[test]
    fn test_calculate_in_amount_clp() {
        let out_amount = Uint256::from(100u64);
        let pool_out_amount = Uint256::from(1000u64);
        let spent_in = Uint256::from(10u64);

        let result = calculate_in_amount_clp(out_amount, pool_out_amount, spent_in);
        let expected = Uint256::from(100u64);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_create_initial_pool_position_msg() {
        let pool_id = 1;
        let treasury = "treasury_address";
        let stream_in_denom = "in_denom";
        let in_clp = Uint256::from(1000u64);
        let stream_out_asset_denom = "out_denom";
        let pool_out_amount_clp = Uint256::from(2000u64);

        let result = build_create_initial_position_msg(
            pool_id,
            treasury.to_string(),
            stream_in_denom.to_string(),
            in_clp,
            stream_out_asset_denom.to_string(),
            pool_out_amount_clp,
            100,
            1000,
        );

        let expected = MsgCreatePosition {
            pool_id,
            sender: treasury.to_string(),
            lower_tick: 100,
            upper_tick: 1000,
            tokens_provided: vec![
                Coin {
                    denom: stream_out_asset_denom.to_string(),
                    amount: pool_out_amount_clp.to_string(),
                },
                Coin {
                    denom: stream_in_denom.to_string(),
                    amount: in_clp.to_string(),
                },
            ],
            token_min_amount0: "0".to_string(),
            token_min_amount1: "0".to_string(),
        };

        assert_eq!(result, expected);
    }
}

pub fn pool_refund(
    deps: &DepsMut,
    pool_config: Option<PoolConfig>,
    out_denom: String,
) -> Result<Vec<Coin>, ContractError> {
    if let Some(pool_config) = pool_config {
        match pool_config {
            PoolConfig::ConcentratedLiquidity { out_amount_clp } => {
                let refund = vec![Coin {
                    denom: out_denom,
                    amount: Uint128::try_from(out_amount_clp)?,
                }];
                let mut pool_creation_fee_vec = get_pool_creation_fee(deps)?;
                pool_creation_fee_vec.extend(refund);

                Ok(pool_creation_fee_vec)
            }
        }
    } else {
        Ok(vec![])
    }
}
