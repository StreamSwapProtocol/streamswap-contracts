use crate::msg::{
    AveragePriceResponse, ExecuteMsg, InstantiateMsg, LatestStreamedPriceResponse, MigrateMsg,
    PositionResponse, PositionsResponse, QueryMsg, StreamResponse, StreamsResponse, SudoMsg,
};
use crate::state::{next_stream_id, Config, Position, Stream, CONFIG, POSITIONS, STREAMS};
use crate::ContractError;
use cosmwasm_std::{
    attr, entry_point, to_binary, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MessageInfo, Order, Response, StdResult, Timestamp, Uint128, Uint64,
};

use cw_storage_plus::Bound;
use cw_utils::{maybe_addr, must_pay};
use std::ops::Mul;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        min_stream_duration: msg.min_stream_duration,
        min_duration_until_start_time: msg.min_duration_until_start_time,
        stream_creation_denom: msg.stream_creation_denom,
        stream_creation_fee: msg.stream_creation_fee,
        fee_collector: deps.api.addr_validate(&msg.fee_collector)?,
    };
    CONFIG.save(deps.storage, &config)?;

    let attrs = vec![
        attr("min_stream_duration", msg.min_stream_duration),
        attr(
            "min_duration_until_start_time",
            msg.min_duration_until_start_time,
        ),
        attr("stream_creation_fee", msg.stream_creation_fee),
    ];
    Ok(Response::default().add_attributes(attrs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateStream {
            treasury,
            name,
            url,
            in_denom: token_in_denom,
            out_denom: token_out_denom,
            out_supply: token_out_supply,
            start_time,
            end_time,
        } => execute_create_stream(
            deps,
            env,
            info,
            treasury,
            name,
            url,
            token_in_denom,
            token_out_denom,
            token_out_supply,
            start_time,
            end_time,
        ),
        ExecuteMsg::TriggerPositionPurchase { stream_id } => {
            execute_trigger_purchase(deps, env, info, stream_id)
        }
        ExecuteMsg::UpdateDistributionIndex { stream_id } => {
            execute_update_dist_index(deps, env, stream_id)
        }
        ExecuteMsg::Subscribe { stream_id } => execute_subscribe(deps, env, info, stream_id),
        ExecuteMsg::Withdraw {
            stream_id,
            cap,
            recipient,
        } => execute_withdraw(deps, env, info, stream_id, recipient, cap),
        ExecuteMsg::FinalizeStream {
            stream_id,
            new_treasury,
        } => execute_finalize_stream(deps, env, info, stream_id, new_treasury),
        ExecuteMsg::ExitStream {
            stream_id,
            recipient,
        } => execute_exit_stream(deps, env, info, stream_id, recipient),
        ExecuteMsg::CollectFees {} => execute_collect_fees(deps, env, info),
    }
}

pub fn execute_create_stream(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    treasury: String,
    name: String,
    url: String,
    in_denom: String,
    out_denom: String,
    out_supply: Uint128,
    start_time: Timestamp,
    end_time: Timestamp,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if end_time.nanos() - start_time.nanos() < config.min_stream_duration.u64() {
        return Err(ContractError::StreamDurationTooShort {});
    }

    if start_time.nanos() - env.block.time.nanos() < config.min_duration_until_start_time.u64() {
        return Err(ContractError::StreamStartsTooSoon {});
    }

    let funds = must_pay(&info, out_denom.as_str())?;
    if funds != out_supply {
        return Err(ContractError::AmountRequired {});
    }

    // TODO: what if fee denom and out denom are same?
    let creation_fee = must_pay(&info, config.stream_creation_denom.as_str())?;
    if creation_fee != config.stream_creation_fee {
        return Err(ContractError::CreationFeeRequired {});
    }

    let stream = Stream::new(
        name.clone(),
        deps.api.addr_validate(&treasury)?,
        url.clone(),
        out_denom.clone(),
        out_supply,
        in_denom.clone(),
        start_time,
        end_time,
    );
    let id = next_stream_id(deps.storage)?;
    STREAMS.save(deps.storage, id, &stream)?;

    let attr = vec![
        attr("action", "create_stream"),
        attr("id", id.to_string()),
        attr("treasury", treasury),
        attr("name", name),
        attr("url", url),
        attr("token_in_denom", in_denom),
        attr("token_out_denom", out_denom),
        attr("token_out_supply", out_supply),
        attr("start_time", start_time.to_string()),
        attr("end_time", end_time.to_string()),
    ];
    Ok(Response::default().add_attributes(attr))
}

/// Increase global_distribution_index with new distribution release
pub fn execute_update_dist_index(
    deps: DepsMut,
    env: Env,
    stream_id: u64,
) -> Result<Response, ContractError> {
    let mut stream = STREAMS.load(deps.storage, stream_id)?;
    let (_, dist_amount) = update_dist_index(env.block.time, &mut stream)?;
    STREAMS.save(deps.storage, stream_id, &stream)?;

    let attrs = vec![
        attr("action", "update_distribution_index"),
        attr("stream_id", stream_id.to_string()),
        attr("distribution_amount", dist_amount),
        attr("stream_dist_index", stream.dist_index.to_string()),
    ];
    let res = Response::new().add_attributes(attrs);
    Ok(res)
}

pub fn update_dist_index(
    now: Timestamp,
    stream: &mut Stream,
) -> Result<(Decimal, Uint128), ContractError> {
    // if now is after end_time, set now to end_time
    let now = if now > stream.end_time {
        stream.end_time
    } else {
        now
    };

    // calculate the current distribution stage
    // dist stage is the (now - start) / (end - start), gives %0-100 percentage
    let numerator = now.nanos() - stream.start_time.nanos();
    let denominator = stream.end_time.nanos() - stream.start_time.nanos();
    let current_dist_stage = Decimal::from_ratio(numerator, denominator);

    // calculate new distribution
    let stage_diff = current_dist_stage.mul(stream.current_stage);

    let new_distribution_balance = stage_diff.mul(stream.out_supply);
    let spent_in = stage_diff.mul(stream.in_supply);
    let deducted_in_supply = stream.in_supply.checked_sub(spent_in)?;

    // can deduct from in_supply be zero?
    stream.dist_index += Decimal::from_ratio(new_distribution_balance, deducted_in_supply);
    stream.current_stage = current_dist_stage;
    stream.spent_in += spent_in;
    stream.in_supply = deducted_in_supply;

    // streamed price is new dist / spent in
    stream.current_streamed_price = new_distribution_balance / spent_in;

    Ok((stage_diff, new_distribution_balance))
}

pub fn execute_trigger_purchase(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    stream_id: u64,
) -> Result<Response, ContractError> {
    let addr = info.sender;

    let mut stream = STREAMS.load(deps.storage, stream_id)?;
    let (_, _) = update_dist_index(env.block.time, &mut stream)?;
    STREAMS.save(deps.storage, stream_id, &stream)?;

    let mut position = POSITIONS.load(deps.storage, (stream_id, &addr))?;
    let (purchased, spent) =
        trigger_purchase(stream.dist_index, stream.current_stage, &mut position)?;
    POSITIONS.save(deps.storage, (stream_id, &position.owner), &position)?;

    Ok(Response::new()
        .add_attribute("action", "trigger_position_purchase")
        .add_attribute("recipient", addr)
        .add_attribute("purchased", purchased)
        .add_attribute("spent", spent))
}

// calculate the user purchase based on the positions index and the global index.
// returns purchased out amount and spent in amount
pub fn trigger_purchase(
    stream_dist_index: Decimal,
    stream_current_stage: Decimal,
    position: &mut Position,
) -> Result<(Uint128, Uint128), ContractError> {
    let stage_diff = stream_current_stage - position.current_stage;
    let spent = stage_diff.mul(position.in_balance);

    let index_diff = stream_dist_index - position.index;

    // update buy balance with spent tokens before calculating purchase, to correct supply reduce
    // on update distribution index
    position.in_balance -= spent;
    let purchased = position.in_balance.mul(index_diff);

    position.index = stream_dist_index;
    position.in_balance -= spent;
    position.current_stage = stream_current_stage;
    position.purchased += purchased;
    position.spent += spent;

    Ok((purchased, spent))
}

pub fn execute_subscribe(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    stream_id: u64,
) -> Result<Response, ContractError> {
    let mut stream = STREAMS.load(deps.storage, stream_id)?;
    if env.block.time > stream.end_time {
        return Err(ContractError::StreamEnded {});
    }
    let in_amount = must_pay(&info, &stream.in_denom)?;

    // if position exists, first update the distribution index and trigger purchase
    // else create position
    let position = POSITIONS.may_load(deps.storage, (stream_id, &info.sender))?;
    match position {
        None => {
            let new_position = Position::new(info.sender.clone(), in_amount);
            // TODO: update dist before position creation?
            POSITIONS.save(deps.storage, (stream_id, &info.sender), &new_position)?;
        }
        Some(mut position) => {
            if position.owner != info.sender {
                return Err(ContractError::Unauthorized {});
            }

            // trigger update distribution index before purchase
            update_dist_index(env.block.time, &mut stream)?;
            trigger_purchase(stream.dist_index, stream.current_stage, &mut position)?;

            position.in_balance += in_amount;
            POSITIONS.save(deps.storage, (stream_id, &info.sender), &position)?;
        }
    }

    // increase in supply
    stream.in_supply += in_amount;
    STREAMS.save(deps.storage, stream_id, &stream)?;

    let res = Response::new()
        .add_attribute("action", "subscribe")
        .add_attribute("stream_id", stream_id.to_string())
        .add_attribute("owner", info.sender)
        .add_attribute("in_supply", stream.in_supply)
        .add_attribute("in_amount", in_amount);

    Ok(res)
}

pub fn execute_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    stream_id: u64,
    recipient: Option<String>,
    cap: Option<Uint128>,
) -> Result<Response, ContractError> {
    let mut stream = STREAMS.load(deps.storage, stream_id)?;

    let mut position = POSITIONS.load(deps.storage, (stream_id, &info.sender))?;
    if position.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    update_dist_index(env.block.time, &mut stream)?;
    STREAMS.save(deps.storage, stream_id, &stream)?;

    let (purchased, spent) =
        trigger_purchase(stream.dist_index, stream.current_stage, &mut position)?;
    POSITIONS.save(deps.storage, (stream_id, &position.owner), &position)?;
    let withdraw_amount = cap.unwrap_or(position.in_balance - spent);

    // if amount to withdraw more then deduced buy balance throw error
    if withdraw_amount > position.in_balance - spent {
        return Err(ContractError::DecreaseAmountExceeds(withdraw_amount));
    }

    stream.current_out += purchased;
    stream.spent_in += spent;
    stream.in_supply -= withdraw_amount;
    STREAMS.save(deps.storage, stream_id, &stream)?;

    let recipient = recipient
        .map(|r| deps.api.addr_validate(&r))
        .transpose()?
        .unwrap_or(info.sender);
    let attributes = vec![
        attr("action", "withdraw"),
        attr("recipient", recipient.as_str()),
        attr("withdraw_amount", withdraw_amount),
    ];

    // send funds to withdraw address or to the sender
    let res = Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: recipient.to_string(),
            amount: vec![Coin {
                denom: stream.in_denom,
                amount: withdraw_amount,
            }],
        }))
        .add_attributes(attributes);

    Ok(res)
}

pub fn execute_finalize_stream(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    stream_id: u64,
    new_treasury: Option<String>,
) -> Result<Response, ContractError> {
    let stream = STREAMS.load(deps.storage, stream_id)?;

    if stream.treasury != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    if env.block.time < stream.end_time {
        return Err(ContractError::StreamNotEnded {});
    }

    if stream.current_stage < Decimal::one() {
        return Err(ContractError::UpdateDistIndex {});
    }

    let treasury = new_treasury
        .map(|t| deps.api.addr_validate(&t))
        .transpose()?
        .unwrap_or(stream.treasury);

    let send_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: treasury.to_string(),
        amount: vec![Coin {
            denom: stream.in_denom,
            amount: stream.spent_in,
        }],
    });

    let config = CONFIG.load(deps.storage)?;
    let fee_send_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: config.fee_collector.to_string(),
        amount: vec![Coin {
            denom: config.stream_creation_denom,
            amount: config.stream_creation_fee,
        }],
    });

    let attributes = vec![
        attr("action", "finalize_stream"),
        attr("stream_id", stream_id.to_string()),
        attr("treasury", treasury.as_str()),
        attr("spent_in", stream.spent_in),
    ];

    Ok(Response::new()
        .add_message(send_msg)
        .add_message(fee_send_msg)
        .add_attributes(attributes))
}

pub fn execute_exit_stream(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    stream_id: u64,
    recipient: Option<String>,
) -> Result<Response, ContractError> {
    let stream = STREAMS.load(deps.storage, stream_id)?;
    if env.block.time < stream.end_time {
        return Err(ContractError::StreamNotEnded {});
    }
    if stream.current_stage < Decimal::one() {
        return Err(ContractError::UpdateDistIndex {});
    }

    let mut position = POSITIONS.load(deps.storage, (stream_id, &info.sender))?;
    if position.exited {
        return Err(ContractError::PositionAlreadyExited {});
    }
    if position.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    if position.current_stage < Decimal::one() {
        return Err(ContractError::TriggerPositionPurchase {});
    }

    let recipient = recipient
        .map(|r| deps.api.addr_validate(&r))
        .transpose()?
        .unwrap_or(position.owner.clone());

    let send_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: recipient.to_string(),
        amount: vec![Coin {
            denom: stream.out_denom,
            amount: position.purchased,
        }],
    });
    let leftover_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: recipient.to_string(),
        amount: vec![Coin {
            denom: stream.in_denom,
            amount: position.in_balance,
        }],
    });

    position.exited = true;
    position.in_balance = Uint128::zero();
    POSITIONS.save(deps.storage, (stream_id, &position.owner), &position)?;

    let attributes = vec![
        attr("action", "exit_stream"),
        attr("stream_id", stream_id.to_string()),
        attr("recipient", recipient.as_str()),
        attr("purchased", position.purchased),
        attr("leftover", position.in_balance),
    ];
    Ok(Response::new()
        .add_message(send_msg)
        .add_message(leftover_msg)
        .add_attributes(attributes))
}

pub fn execute_collect_fees(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.fee_collector != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let collected_fees = deps
        .querier
        .query_balance(env.contract.address, config.stream_creation_denom.as_str())?;
    let send_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: config.fee_collector.to_string(),
        amount: vec![collected_fees],
    });

    Ok(Response::new().add_message(send_msg))
}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::UpdateConfig {
            min_stream_duration,
            min_duration_until_start_time,
            stream_creation_denom,
            stream_creation_fee,
            fee_collector,
        } => sudo_update_config(
            deps,
            env,
            min_stream_duration,
            min_duration_until_start_time,
            stream_creation_denom,
            stream_creation_fee,
            fee_collector,
        ),
    }
}

pub fn sudo_update_config(
    deps: DepsMut,
    _env: Env,
    min_stream_duration: Option<Uint64>,
    min_duration_until_start_time: Option<Uint64>,
    stream_creation_denom: Option<String>,
    stream_creation_fee: Option<Uint128>,
    fee_collector: Option<String>,
) -> Result<Response, ContractError> {
    let mut cfg = CONFIG.load(deps.storage)?;

    cfg.min_stream_duration = min_stream_duration.unwrap_or(cfg.min_stream_duration);
    cfg.min_duration_until_start_time =
        min_duration_until_start_time.unwrap_or(cfg.min_duration_until_start_time);
    cfg.stream_creation_denom = stream_creation_denom.unwrap_or(cfg.stream_creation_denom);
    cfg.stream_creation_fee = stream_creation_fee.unwrap_or(cfg.stream_creation_fee);

    let collector = fee_collector
        .map(|r| deps.api.addr_validate(&r))
        .transpose()?
        .unwrap_or(cfg.fee_collector);
    cfg.fee_collector = collector;

    CONFIG.save(deps.storage, &cfg)?;

    let attributes = vec![
        attr("action", "update_config"),
        attr("min_stream_duration", cfg.min_stream_duration),
        attr(
            "min_duration_until_start_time",
            cfg.min_duration_until_start_time,
        ),
        attr("stream_creation_denom", cfg.stream_creation_denom),
        attr("stream_creation_fee", cfg.stream_creation_fee),
        attr("fee_collector", cfg.fee_collector),
    ];

    Ok(Response::default().add_attributes(attributes))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Stream { stream_id } => to_binary(&query_stream(deps, env, stream_id)?),
        QueryMsg::Position { stream_id, owner } => {
            to_binary(&query_position(deps, env, stream_id, owner)?)
        }
        QueryMsg::ListStreams { start_after, limit } => {
            to_binary(&list_streams(deps, start_after, limit)?)
        }
        QueryMsg::ListPositions {
            stream_id,
            start_after,
            limit,
        } => to_binary(&list_positions(deps, stream_id, start_after, limit)?),
        QueryMsg::AveragePrice { stream_id } => {
            to_binary(&query_average_price(deps, env, stream_id)?)
        }
        QueryMsg::LastStreamedPrice { stream_id } => {
            to_binary(&query_last_streamed_price(deps, env, stream_id)?)
        }
    }
}

pub fn query_stream(deps: Deps, _env: Env, stream_id: u64) -> StdResult<StreamResponse> {
    let stream = STREAMS.load(deps.storage, stream_id)?;
    let stream = StreamResponse {
        id: stream_id,
        treasury: stream.treasury.to_string(),
        token_in_denom: stream.in_denom,
        token_out_denom: stream.out_denom,
        token_out_supply: stream.out_supply,
        start_time: stream.start_time,
        end_time: stream.end_time,
        total_in_spent: stream.spent_in,
        latest_stage: stream.current_stage,
        dist_index: stream.dist_index,
        total_out_sold: stream.current_out,
        total_in_supply: stream.in_supply,
    };
    Ok(stream)
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

pub fn list_streams(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<StreamsResponse> {
    let start = start_after.map(Bound::exclusive);
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let streams: StdResult<Vec<StreamResponse>> = STREAMS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (stream_id, stream) = item?;
            let stream = StreamResponse {
                id: stream_id,
                treasury: stream.treasury.to_string(),
                token_in_denom: stream.in_denom,
                token_out_denom: stream.out_denom,
                token_out_supply: stream.out_supply,
                start_time: stream.start_time,
                end_time: stream.end_time,
                total_in_spent: stream.spent_in,
                latest_stage: stream.current_stage,
                dist_index: stream.dist_index,
                total_out_sold: stream.current_out,
                total_in_supply: stream.in_supply,
            };
            Ok(stream)
        })
        .collect();
    let streams = streams?;
    Ok(StreamsResponse { streams })
}

pub fn query_position(
    deps: Deps,
    _env: Env,
    stream_id: u64,
    owner: String,
) -> StdResult<PositionResponse> {
    let owner = deps.api.addr_validate(&owner)?;
    let position = POSITIONS.load(deps.storage, (stream_id, &owner))?;
    let res = PositionResponse {
        stream_id,
        owner: owner.to_string(),
        in_balance: position.in_balance,
        purchased: position.purchased,
        current_stage: position.current_stage,
        exited: position.exited,
        index: position.index,
        spent: position.spent,
    };
    Ok(res)
}

pub fn list_positions(
    deps: Deps,
    stream_id: u64,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<PositionsResponse> {
    let addr = maybe_addr(deps.api, start_after)?;
    let start = addr.as_ref().map(Bound::exclusive);
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let positions: StdResult<Vec<PositionResponse>> = POSITIONS
        .prefix(stream_id)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (owner, stream) = item?;
            let position = PositionResponse {
                stream_id,
                owner: owner.to_string(),
                index: stream.index,
                current_stage: stream.current_stage,
                purchased: stream.purchased,
                spent: stream.spent,
                in_balance: stream.in_balance,
                exited: stream.exited,
            };
            Ok(position)
        })
        .collect();
    let positions = positions?;
    Ok(PositionsResponse { positions })
}

pub fn query_average_price(
    deps: Deps,
    _env: Env,
    stream_id: u64,
) -> StdResult<AveragePriceResponse> {
    let stream = STREAMS.load(deps.storage, stream_id)?;
    let average_price = stream.current_out / stream.spent_in;
    Ok(AveragePriceResponse { average_price })
}

pub fn query_last_streamed_price(
    deps: Deps,
    _env: Env,
    stream_id: u64,
) -> StdResult<LatestStreamedPriceResponse> {
    let stream = STREAMS.load(deps.storage, stream_id)?;
    Ok(LatestStreamedPriceResponse {
        current_streamed_price: stream.current_streamed_price,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn basic() {}
}
