use crate::contract::{update_position, update_stream};
use crate::state::{Status, Stream, CONFIG, POSITIONS, STREAMS};
use crate::{contract, ContractError};
use cosmwasm_std::{
    attr, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdResult, Timestamp,
    Uint128,
};
use cw_utils::maybe_addr;

pub fn execute_withdraw_paused(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    stream_id: u64,
    cap: Option<Uint128>,
    position_owner: Option<String>,
) -> Result<Response, ContractError> {
    let mut stream = STREAMS.load(deps.storage, stream_id)?;
    // check if stream is paused
    if !stream.is_paused() {
        return Err(ContractError::StreamNotPaused {});
    }
    // can't withdraw after stream ended
    if env.block.time > stream.end_time {
        return Err(ContractError::StreamEnded {});
    }

    let position_owner = maybe_addr(deps.api, position_owner)?.unwrap_or(info.sender.clone());
    let mut position = POSITIONS.load(deps.storage, (stream_id, &position_owner))?;
    if position.owner != info.sender
        && position
            .operator
            .as_ref()
            .map_or(true, |o| o != &info.sender)
    {
        return Err(ContractError::Unauthorized {});
    }

    // on withdraw paused stream we don't update_stream
    update_position(
        stream.dist_index,
        stream.shares,
        stream.last_updated,
        stream.in_supply,
        &mut position,
    )?;

    let withdraw_amount = cap.unwrap_or(position.in_balance);
    // if amount to withdraw more then deduced buy balance throw error
    if withdraw_amount > position.in_balance {
        return Err(ContractError::DecreaseAmountExceeds(withdraw_amount));
    }

    // decrease in supply and shares
    let shares_amount = if withdraw_amount == position.in_balance {
        position.shares
    } else {
        stream.compute_shares_amount(withdraw_amount, true)
    };

    stream.in_supply = stream.in_supply.checked_sub(withdraw_amount)?;
    stream.shares = stream.shares.checked_sub(shares_amount)?;
    position.in_balance = position.in_balance.checked_sub(withdraw_amount)?;
    position.shares = position.shares.checked_sub(shares_amount)?;

    STREAMS.save(deps.storage, stream_id, &stream)?;
    POSITIONS.save(deps.storage, (stream_id, &position.owner), &position)?;

    let attributes = vec![
        attr("action", "withdraw_paused"),
        attr("stream_id", stream_id.to_string()),
        attr("position_owner", position_owner.clone()),
        attr("withdraw_amount", withdraw_amount),
    ];

    // send funds to withdraw address or to the sender
    let res = Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: position_owner.to_string(),
            amount: vec![Coin {
                denom: stream.in_denom,
                amount: withdraw_amount,
            }],
        }))
        .add_attributes(attributes);

    Ok(res)
}

pub fn execute_exit_cancelled(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    stream_id: u64,
    position_owner: Option<String>,
) -> Result<Response, ContractError> {
    let mut stream = STREAMS.load(deps.storage, stream_id)?;
    // check if stream is cancelled
    if !stream.is_cancelled() {
        return Err(ContractError::StreamNotCancelled {});
    }

    let position_owner = maybe_addr(deps.api, position_owner)?.unwrap_or(info.sender.clone());
    let mut position = POSITIONS.load(deps.storage, (stream_id, &position_owner))?;
    if position.owner != info.sender
        && position
            .operator
            .as_ref()
            .map_or(true, |o| o != &info.sender)
    {
        return Err(ContractError::Unauthorized {});
    }

    // no need to update position here, we just need to return total balance
    let total_balance = position.in_balance + position.spent;
    POSITIONS.remove(deps.storage, (stream_id, &position.owner));

    let attributes = vec![
        attr("action", "withdraw_cancelled"),
        attr("stream_id", stream_id.to_string()),
        attr("position_owner", position_owner.clone()),
        attr("total_balance", total_balance),
    ];

    // send funds to withdraw address or to the sender
    let res = Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: position_owner.to_string(),
            amount: vec![Coin {
                denom: stream.in_denom,
                amount: total_balance,
            }],
        }))
        .add_attributes(attributes);

    Ok(res)
}

pub fn execute_pause_stream(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    stream_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.protocol_admin {
        return Err(ContractError::Unauthorized {});
    }

    // update stream before pause
    let mut stream = STREAMS.load(deps.storage, stream_id)?;
    update_stream(env.block.time, &mut stream)?;
    pause_stream(env.block.time, &mut stream)?;
    STREAMS.save(deps.storage, stream_id, &stream)?;

    Ok(Response::default()
        .add_attribute("stream_id", stream_id.to_string())
        .add_attribute("is_paused", "true")
        .add_attribute("pause_date", env.block.time.to_string()))
}

pub fn pause_stream(now: Timestamp, stream: &mut Stream) -> StdResult<()> {
    stream.status = Status::Paused;
    stream.pause_date = Some(now);
    Ok(())
}

pub fn sudo_pause_stream(
    deps: DepsMut,
    env: Env,
    stream_id: u64,
) -> Result<Response, ContractError> {
    let mut stream = STREAMS.load(deps.storage, stream_id)?;
    update_stream(env.block.time, &mut stream)?;
    pause_stream(env.block.time, &mut stream)?;
    STREAMS.save(deps.storage, stream_id, &stream)?;

    Ok(Response::default()
        .add_attribute("stream_id", stream_id.to_string())
        .add_attribute("is_paused", "true")
        .add_attribute("pause_date", env.block.time.to_string()))
}

pub fn sudo_resume_stream(
    deps: DepsMut,
    env: Env,
    stream_id: u64,
) -> Result<Response, ContractError> {
    let mut stream = STREAMS.load(deps.storage, stream_id)?;
    // ok to use unwrap here
    let pause_date = stream.pause_date.unwrap();
    // new stream end time is the time passed between pause date and now
    stream.end_time = stream
        .end_time
        .plus_nanos(env.block.time.nanos() - pause_date.nanos());
    stream.status = Status::Active;
    stream.pause_date = None;
    STREAMS.save(deps.storage, stream_id, &stream)?;

    Ok(Response::default()
        .add_attribute("stream_id", stream_id.to_string())
        .add_attribute("new_end_date", stream.end_time.to_string())
        .add_attribute("status", "active"))
}

pub fn sudo_cancel_stream(
    deps: DepsMut,
    _env: Env,
    stream_id: u64,
) -> Result<Response, ContractError> {
    let mut stream = STREAMS.load(deps.storage, stream_id)?;
    if !stream.is_paused() {
        return Err(ContractError::StreamNotPaused {});
    }
    stream.status = Status::Cancelled;
    STREAMS.save(deps.storage, stream_id, &stream)?;

    Ok(Response::default()
        .add_attribute("stream_id", stream_id.to_string())
        .add_attribute("status", "cancelled"))
}
