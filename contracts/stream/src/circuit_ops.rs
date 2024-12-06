use crate::pool::pool_refund;
use crate::state::{CONTROLLER_PARAMS, POST_STREAM, STREAM_INFO, STREAM_STATE};
use crate::stream::{sync_stream, sync_stream_status};
use crate::ContractError;
use cosmwasm_std::{BankMsg, CosmosMsg, DepsMut, Env, MessageInfo, Response};
use cw_utils::NativeBalance;
use streamswap_types::controller::Params;
use streamswap_types::stream::Status;

pub fn execute_cancel_stream(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let controller_params: Params = CONTROLLER_PARAMS.load(deps.storage)?;

    if controller_params.protocol_admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    let mut stream = STREAM_STATE.load(deps.storage)?;
    sync_stream_status(&mut stream, env.block.time);

    if stream.is_finalized() || stream.is_cancelled() {
        return Err(ContractError::OperationNotAllowed {
            current_status: stream.status_info.status.to_string(),
        });
    }
    stream.status_info.status = Status::Cancelled;

    sync_stream(&mut stream, env.block.time);
    STREAM_STATE.save(deps.storage, &stream)?;

    // Refund all out tokens to stream creator(treasury)
    let mut refund_coins = NativeBalance::default() + stream.out_asset.clone();

    // refund pool creation if any
    let post_stream_ops = POST_STREAM.may_load(deps.storage)?;
    if let Some(post_stream_ops) = post_stream_ops {
        let pool_refund_coins = pool_refund(
            &deps,
            post_stream_ops.pool_config,
            stream.out_asset.denom.clone(),
        )?;
        for coin in pool_refund_coins {
            refund_coins += coin;
        }
    }

    refund_coins.normalize();
    let stream_info = STREAM_INFO.load(deps.storage)?;
    let funds_msgs: Vec<CosmosMsg> = refund_coins
        .into_vec()
        .iter()
        .map(|coin| {
            CosmosMsg::Bank(BankMsg::Send {
                to_address: stream_info.treasury.to_string(),
                amount: vec![coin.clone()],
            })
        })
        .collect();

    Ok(Response::new()
        .add_attribute("action", "cancel_stream")
        .add_attribute("status", "cancelled")
        .add_messages(funds_msgs))
}
pub fn execute_stream_admin_cancel(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut stream = STREAM_STATE.load(deps.storage)?;
    let stream_info = STREAM_INFO.load(deps.storage)?;
    // Only stream admin can cancel the stream with this method
    if info.sender != stream_info.stream_admin {
        return Err(ContractError::Unauthorized {});
    }

    sync_stream_status(&mut stream, env.block.time);

    // In order for stream admin to cancel the stream, the stream should be waiting
    if !stream.is_waiting() {
        return Err(ContractError::OperationNotAllowed {
            current_status: stream.status_info.status.to_string(),
        });
    }
    stream.status_info.status = Status::Cancelled;
    sync_stream(&mut stream, env.block.time);
    STREAM_STATE.save(deps.storage, &stream)?;

    // Refund all out tokens to stream creator(treasury)
    let mut refund_coins = NativeBalance::default() + stream.out_asset.clone();

    // refund pool creation if any
    let post_stream_ops = POST_STREAM.may_load(deps.storage)?;
    if let Some(post_stream_ops) = post_stream_ops {
        let pool_refund_coins = pool_refund(
            &deps,
            post_stream_ops.pool_config,
            stream.out_asset.denom.clone(),
        )?;
        for coin in pool_refund_coins {
            refund_coins += coin;
        }
    }

    refund_coins.normalize();

    let funds_msgs: Vec<CosmosMsg> = refund_coins
        .into_vec()
        .iter()
        .map(|coin| {
            CosmosMsg::Bank(BankMsg::Send {
                to_address: stream_info.treasury.to_string(),
                amount: vec![coin.clone()],
            })
        })
        .collect();

    Ok(Response::new()
        .add_attribute("action", "cancel_stream")
        .add_messages(funds_msgs)
        .add_attribute("status", "cancelled"))
}
