use crate::contract::update_stream;
use crate::state::{FACTORY_PARAMS, POSITIONS, STREAM};
use crate::ContractError;
use cosmwasm_std::{attr, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response};
use cw_utils::maybe_addr;
use streamswap_types::factory::Params;
use streamswap_types::stream::ThresholdState;
use streamswap_types::stream::{Status, ThresholdError};

pub fn execute_exit_cancelled(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operator_target: Option<String>,
) -> Result<Response, ContractError> {
    let mut stream = STREAM.load(deps.storage)?;

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
        if stream.end_time > env.block.time {
            return Err(ContractError::StreamNotCancelled {});
        }
        // Update stream before checking threshold
        update_stream(env.block.time, &mut stream)?;
        threshold_state.error_if_reached(deps.storage, &stream)?;
    }

    let operator_target =
        maybe_addr(deps.api, operator_target)?.unwrap_or_else(|| info.sender.clone());
    let position = POSITIONS.load(deps.storage, &operator_target)?;
    if position.owner != info.sender
        && position
            .operator
            .as_ref()
            .map_or(true, |o| o != info.sender)
    {
        return Err(ContractError::Unauthorized {});
    }

    // no need to update position here, we just need to return total balance
    let total_balance = position.in_balance + position.spent;
    POSITIONS.remove(deps.storage, &position.owner);

    let attributes = vec![
        attr("action", "withdraw_cancelled"),
        attr("operator_target", operator_target.clone()),
        attr("total_balance", total_balance),
    ];

    // send funds to withdraw address or to the sender
    let res = Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: operator_target.to_string(),
            amount: vec![Coin {
                denom: stream.in_denom,
                amount: total_balance,
            }],
        }))
        .add_attributes(attributes);

    Ok(res)
}

pub fn execute_cancel_stream(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let factory_params: Params = FACTORY_PARAMS.load(deps.storage)?;

    if factory_params.protocol_admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    let mut stream = STREAM.load(deps.storage)?;

    if stream.is_cancelled() {
        return Err(ContractError::StreamIsCancelled {});
    }
    stream.status = Status::Cancelled;
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

    if env.block.time < stream.end_time {
        return Err(ContractError::StreamNotEnded {});
    }
    if info.sender != stream.treasury {
        return Err(ContractError::Unauthorized {});
    }

    // Stream should not be paused or cancelled
    if stream.is_killswitch_active() {
        return Err(ContractError::StreamKillswitchActive {});
    }

    // This should be impossible because creator can not finalize stream when threshold is not reached
    if stream.status == Status::Finalized {
        return Err(ContractError::StreamAlreadyFinalized {});
    }

    if stream.last_updated < env.block.time {
        update_stream(env.block.time, &mut stream)?;
    }

    let threshold_state = ThresholdState::new();

    if !threshold_state.check_if_threshold_set(deps.storage)? {
        return Err(ContractError::ThresholdError(
            ThresholdError::ThresholdNotSet {},
        ));
    }
    // Threshold should not be reached
    threshold_state.error_if_reached(deps.storage, &stream)?;

    stream.status = Status::Cancelled;

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
pub fn sudo_pause_stream(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let mut stream = STREAM.load(deps.storage)?;

    STREAM.save(deps.storage, &stream)?;

    if env.block.time >= stream.end_time {
        return Err(ContractError::StreamEnded {});
    }
    // check if stream is not started
    if env.block.time < stream.start_time {
        return Err(ContractError::StreamNotStarted {});
    }
    // Paused or cancelled can not be paused
    if stream.is_killswitch_active() {
        return Err(ContractError::StreamKillswitchActive {});
    }
    update_stream(env.block.time, &mut stream)?;
    STREAM.save(deps.storage, &stream)?;

    Ok(Response::default()
        .add_attribute("action", "sudo_pause_stream")
        .add_attribute("is_paused", "true")
        .add_attribute("pause_block", env.block.height.to_string()))
}

pub fn sudo_resume_stream(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let mut stream = STREAM.load(deps.storage)?;
    //Cancelled can't be resumed
    if stream.is_cancelled() {
        return Err(ContractError::StreamIsCancelled {});
    }
    let pause_date = stream.pause_date.unwrap();
    //postpone stream times with respect to pause duration
    stream.end_time = stream
        .end_time
        .plus_nanos(env.block.time.nanos() - pause_date.nanos());
    stream.last_updated = stream
        .last_updated
        .plus_nanos(env.block.time.nanos() - pause_date.nanos());

    stream.status = Status::Active;
    stream.pause_date = None;
    STREAM.save(deps.storage, &stream)?;

    Ok(Response::default()
        .add_attribute("action", "resume_stream")
        .add_attribute("new_end_date", stream.end_time.to_string())
        .add_attribute("status", "active"))
}

pub fn sudo_cancel_stream(deps: DepsMut, _env: Env) -> Result<Response, ContractError> {
    let mut stream = STREAM.load(deps.storage)?;
    let factory_params: Params = FACTORY_PARAMS.load(deps.storage)?;
    if stream.is_cancelled() {
        return Err(ContractError::StreamIsCancelled {});
    }
    stream.status = Status::Cancelled;
    STREAM.save(deps.storage, &stream)?;

    //Refund all out tokens to stream creator(treasury)
    let messages: Vec<CosmosMsg> = vec![
        CosmosMsg::Bank(BankMsg::Send {
            to_address: stream.treasury.to_string(),
            amount: vec![Coin {
                denom: stream.out_asset.denom,
                amount: stream.out_asset.amount,
            }],
        }),
        //Refund stream creation fee to stream creator
        CosmosMsg::Bank(BankMsg::Send {
            to_address: stream.treasury.to_string(),
            amount: vec![Coin {
                denom: factory_params.stream_creation_fee.denom,
                amount: factory_params.stream_creation_fee.amount,
            }],
        }),
    ];

    Ok(Response::new()
        .add_attribute("action", "cancel_stream")
        .add_messages(messages)
        .add_attribute("status", "cancelled"))
}
