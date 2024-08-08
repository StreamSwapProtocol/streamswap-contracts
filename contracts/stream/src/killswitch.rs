use crate::state::{FACTORY_PARAMS, POSITIONS, STREAM};
use crate::stream::{sync_stream_status, update_stream};
use crate::ContractError;
use cosmwasm_std::{
    attr, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, Timestamp, Uint128,
};
use streamswap_types::factory::Params;
use streamswap_types::stream::ThresholdState;
use streamswap_types::stream::{Status, ThresholdError};

pub fn execute_exit_cancelled(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut stream = STREAM.load(deps.storage)?;

    let mut position = POSITIONS.load(deps.storage, &info.sender)?;
    if position.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    // TODO: add test case for this
    if position.exit_date != Timestamp::from_seconds(0) {
        return Err(ContractError::SubscriberAlreadyExited {});
    }

    sync_stream_status(&mut stream, env.block.time);

    // This execution requires the stream to be cancelled or
    // the stream to be ended and the threshold not reached.
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
        update_stream(&mut stream, env.block.time);
        threshold_state.error_if_reached(deps.storage, &stream)?;
    }

    // no need to update position here, we just need to return total balance
    let total_balance = position.in_balance + position.spent;
    // update position exit date
    position.exit_date = env.block.time;
    POSITIONS.save(deps.storage, &position.owner, &position)?;

    let attributes = vec![
        attr("action", "withdraw_cancelled"),
        attr("total_balance", total_balance),
    ];

    let uint128_total_balance = Uint128::try_from(total_balance)?;
    // send funds to withdraw address or to the sender
    let res = Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin {
                denom: stream.in_denom,
                amount: uint128_total_balance,
            }],
        }))
        .add_attributes(attributes);

    Ok(res)
}

pub fn execute_cancel_stream(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let factory_params: Params = FACTORY_PARAMS.load(deps.storage)?;

    if factory_params.protocol_admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    let mut stream = STREAM.load(deps.storage)?;
    sync_stream_status(&mut stream, env.block.time);

    // TODO if finalized can not be cancelled
    if stream.is_ended() {
        return Err(ContractError::StreamEnded {});
    }

    if stream.is_cancelled() {
        return Err(ContractError::StreamIsCancelled {});
    }
    stream.status_info.status = Status::Cancelled;
    STREAM.save(deps.storage, &stream)?;

    //Refund all out tokens to stream creator(treasury)
    let messages: Vec<CosmosMsg> = vec![CosmosMsg::Bank(BankMsg::Send {
        to_address: stream.treasury.to_string(),
        amount: vec![Coin {
            denom: stream.out_asset.denom,
            amount: stream.out_asset.amount,
        }],
    })];

    Ok(Response::new()
        .add_attribute("action", "cancel_stream")
        .add_messages(messages)
        .add_attribute("status", "cancelled"))
}

pub fn execute_cancel_stream_with_threshold(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut stream = STREAM.load(deps.storage)?;
    sync_stream_status(&mut stream, env.block.time);
    // Stream should not be paused or cancelled
    if stream.is_cancelled() {
        return Err(ContractError::StreamKillswitchActive {});
    }

    if !stream.is_ended() {
        return Err(ContractError::StreamNotEnded {});
    }
    // Update stream before checking threshold
    if info.sender != stream.treasury {
        return Err(ContractError::Unauthorized {});
    }

    // This should be impossible because creator can not finalize stream when threshold is not reached
    if stream.status_info.status == Status::Finalized {
        return Err(ContractError::StreamAlreadyFinalized {});
    }

    update_stream(&mut stream, env.block.time);

    let threshold_state = ThresholdState::new();

    if !threshold_state.check_if_threshold_set(deps.storage)? {
        return Err(ContractError::ThresholdError(
            ThresholdError::ThresholdNotSet {},
        ));
    }
    // Threshold should not be reached
    threshold_state.error_if_reached(deps.storage, &stream)?;

    stream.status_info.status = Status::Cancelled;

    STREAM.save(deps.storage, &stream)?;

    //Refund all out tokens to stream creator(treasury)
    let messages: Vec<CosmosMsg> = vec![CosmosMsg::Bank(BankMsg::Send {
        to_address: stream.treasury.to_string(),
        amount: vec![Coin {
            denom: stream.out_asset.denom,
            amount: stream.out_asset.amount,
        }],
    })];

    Ok(Response::new()
        .add_attribute("action", "cancel_stream")
        .add_messages(messages)
        .add_attribute("status", "cancelled"))
}
pub fn execute_stream_admin_cancel(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut stream = STREAM.load(deps.storage)?;
    sync_stream_status(&mut stream, env.block.time);

    // In order for stream admin to cancel the stream, the stream should be waiting
    if !stream.is_waiting() {
        return Err(ContractError::StreamNotWaiting {});
    }

    if info.sender != stream.stream_admin {
        return Err(ContractError::Unauthorized {});
    }
    stream.status_info.status = Status::Cancelled;
    STREAM.save(deps.storage, &stream)?;

    //Refund all out tokens to stream creator(treasury)
    let messages: Vec<CosmosMsg> = vec![CosmosMsg::Bank(BankMsg::Send {
        to_address: stream.treasury.to_string(),
        amount: vec![stream.out_asset],
    })];

    Ok(Response::new()
        .add_attribute("action", "cancel_stream")
        .add_messages(messages)
        .add_attribute("status", "cancelled"))
}
