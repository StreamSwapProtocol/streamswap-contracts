use crate::helpers::build_u128_bank_send_msg;
use crate::pool::pool_refund;
use crate::state::{CONTROLLER_PARAMS, POSITIONS, POST_STREAM, STREAM_INFO, STREAM_STATE};
use crate::stream::{sync_stream, sync_stream_status};
use crate::ContractError;
use cosmwasm_std::{attr, BankMsg, CosmosMsg, DepsMut, Env, MessageInfo, Response, Timestamp};
use streamswap_types::controller::Params;
use streamswap_types::stream::ThresholdState;
use streamswap_types::stream::{Status, ThresholdError};

pub fn execute_exit_cancelled(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut stream = STREAM_STATE.load(deps.storage)?;

    let mut position = POSITIONS.load(deps.storage, &info.sender)?;
    if position.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    if position.exit_date != Timestamp::from_seconds(0) {
        return Err(ContractError::SubscriberAlreadyExited {});
    }

    sync_stream_status(&mut stream, env.block.time);

    // This execution requires the stream to be cancelled or
    // the stream to be ended and the threshold not reached.
    // If any of other condition fails return not cancelled error.
    if !stream.is_cancelled() {
        let threshold_state = ThresholdState::new();
        // Threshold should be set
        let is_set = threshold_state.check_if_threshold_set(deps.storage)?;
        if !is_set {
            return Err(ContractError::StreamNotCancelled {});
        }

        // Stream should be ended
        if !stream.is_ended() {
            return Err(ContractError::StreamNotCancelled {});
        }
        // Update stream before checking threshold
        sync_stream(&mut stream, env.block.time);
        threshold_state.error_if_reached(deps.storage, &stream)?;
    }

    // no need to sync position here, we just need to return total balance
    let total_balance = position.in_balance + position.spent;
    // sync position exit date
    position.exit_date = env.block.time;
    position.last_updated = env.block.time;
    POSITIONS.save(deps.storage, &position.owner, &position)?;

    let send_msg = build_u128_bank_send_msg(
        stream.in_denom.clone(),
        info.sender.to_string(),
        total_balance,
    )?;
    let attributes = vec![
        attr("action", "exit_cancelled"),
        attr("to_address", info.sender.to_string()),
        attr("total_balance", total_balance),
        attr("exit_date", position.exit_date.to_string()),
        attr("last_updated", position.last_updated.to_string()),
    ];
    // send funds to the sender
    let res = Response::new()
        .add_message(send_msg)
        .add_attributes(attributes);

    Ok(res)
}

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
    let mut refund_coins = vec![stream.out_asset.clone()];

    // refund pool creation if any
    let post_stream_ops = POST_STREAM.may_load(deps.storage)?;
    if let Some(post_stream_ops) = post_stream_ops {
        let pool_refund_coins = pool_refund(
            &deps,
            post_stream_ops.pool_config,
            stream.out_asset.denom.clone(),
        )?;
        refund_coins.extend(pool_refund_coins);
    }

    let stream_info = STREAM_INFO.load(deps.storage)?;
    let funds_msgs: Vec<CosmosMsg> = refund_coins
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

pub fn execute_cancel_stream_with_threshold(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut stream = STREAM_STATE.load(deps.storage)?;
    let stream_info = STREAM_INFO.load(deps.storage)?;
    // Only stream creator can cancel the stream with threshold not reached
    if info.sender != stream_info.stream_admin {
        return Err(ContractError::Unauthorized {});
    }
    sync_stream_status(&mut stream, env.block.time);
    // Stream should not be cancelled of finalized, should be ended.
    // Creator should not able to finalize the stream with threshold not reached but only cancel it.
    if !stream.is_ended() {
        return Err(ContractError::OperationNotAllowed {
            current_status: stream.status_info.status.to_string(),
        });
    }

    sync_stream(&mut stream, env.block.time);

    let threshold_state = ThresholdState::new();

    if !threshold_state.check_if_threshold_set(deps.storage)? {
        return Err(ContractError::ThresholdError(
            ThresholdError::ThresholdNotSet {},
        ));
    }
    // Threshold should not be reached
    threshold_state.error_if_reached(deps.storage, &stream)?;

    stream.status_info.status = Status::Cancelled;

    STREAM_STATE.save(deps.storage, &stream)?;

    // Refund all out tokens to stream creator(treasury)
    let mut refund_coins = vec![stream.out_asset.clone()];

    // refund pool creation if any
    let post_stream_ops = POST_STREAM.may_load(deps.storage)?;
    if let Some(post_stream_ops) = post_stream_ops {
        let pool_refund_coins = pool_refund(
            &deps,
            post_stream_ops.pool_config,
            stream.out_asset.denom.clone(),
        )?;
        refund_coins.extend(pool_refund_coins);
    }

    let funds_msgs: Vec<CosmosMsg> = refund_coins
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
    let mut refund_coins = vec![stream.out_asset.clone()];

    // refund pool creation if any
    let post_stream_ops = POST_STREAM.may_load(deps.storage)?;
    if let Some(post_stream_ops) = post_stream_ops {
        let pool_refund_coins = pool_refund(
            &deps,
            post_stream_ops.pool_config,
            stream.out_asset.denom.clone(),
        )?;
        refund_coins.extend(pool_refund_coins);
    }

    let funds_msgs: Vec<CosmosMsg> = refund_coins
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
