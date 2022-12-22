use crate::msg::{
    AveragePriceResponse, ExecuteMsg, InstantiateMsg, LatestStreamedPriceResponse, MigrateMsg,
    PositionResponse, PositionsResponse, QueryMsg, StreamResponse, StreamsResponse, SudoMsg,
};
use crate::state::{next_stream_id, Config, Position, Stream, CONFIG, POSITIONS, STREAMS};
use crate::ContractError;
use cosmwasm_std::{
    attr, entry_point, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Decimal256,
    Deps, DepsMut, Env, Fraction, MessageInfo, Order, Response, StdResult, Timestamp, Uint128,
    Uint256, Uint64,
};

use crate::helpers::get_decimals;
use cw_storage_plus::Bound;
use cw_utils::{maybe_addr, must_pay};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        min_stream_seconds: msg.min_stream_seconds,
        min_seconds_until_start_time: msg.min_seconds_until_start_time,
        stream_creation_denom: msg.stream_creation_denom,
        stream_creation_fee: msg.stream_creation_fee,
        fee_collector: deps.api.addr_validate(&msg.fee_collector)?,
    };
    CONFIG.save(deps.storage, &config)?;

    let attrs = vec![
        attr("min_stream_seconds", msg.min_stream_seconds),
        attr(
            "min_seconds_until_start_time",
            msg.min_seconds_until_start_time,
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
            in_denom,
            out_denom,
            out_supply,
            start_time,
            end_time,
        } => execute_create_stream(
            deps, env, info, treasury, name, url, in_denom, out_denom, out_supply, start_time,
            end_time,
        ),
        ExecuteMsg::UpdateOperator {
            stream_id,
            operator,
        } => execute_update_operator(deps, env, info, stream_id, operator),

        ExecuteMsg::UpdatePosition {
            stream_id,
            position_owner,
        } => execute_update_position(deps, env, info, stream_id, position_owner),
        ExecuteMsg::UpdateStream { stream_id } => execute_update_stream(deps, env, stream_id),
        ExecuteMsg::Subscribe {
            stream_id,
            position_owner,
            operator,
        } => execute_subscribe(deps, env, info, stream_id, operator, position_owner),
        ExecuteMsg::Withdraw {
            stream_id,
            cap,
            position_owner,
        } => execute_withdraw(deps, env, info, stream_id, cap, position_owner),
        ExecuteMsg::FinalizeStream {
            stream_id,
            new_treasury,
        } => execute_finalize_stream(deps, env, info, stream_id, new_treasury),
        ExecuteMsg::ExitStream {
            stream_id,
            position_owner,
        } => execute_exit_stream(deps, env, info, stream_id, position_owner),
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
    if end_time < start_time {
        return Err(ContractError::StreamInvalidEndTime {});
    }
    if env.block.time > start_time {
        return Err(ContractError::StreamInvalidStartTime {});
    }
    if end_time.seconds() - start_time.seconds() < config.min_stream_seconds.u64() {
        return Err(ContractError::StreamDurationTooShort {});
    }

    if start_time.seconds() - env.block.time.seconds() < config.min_seconds_until_start_time.u64() {
        return Err(ContractError::StreamStartsTooSoon {});
    }

    if out_denom == config.stream_creation_denom {
        let total_funds = info
            .funds
            .iter()
            .find(|p| p.denom == config.stream_creation_denom)
            .ok_or(ContractError::NoFundsSent {})?;
        if total_funds.amount != config.stream_creation_fee + out_supply {
            return Err(ContractError::StreamOutSupplyFundsRequired {});
        }
    } else {
        let funds = info
            .funds
            .iter()
            .find(|p| p.denom == out_denom)
            .ok_or(ContractError::NoFundsSent {})?;
        if funds.amount != out_supply {
            return Err(ContractError::StreamOutSupplyFundsRequired {});
        }

        let creation_fee = info
            .funds
            .iter()
            .find(|p| p.denom == config.stream_creation_denom)
            .ok_or(ContractError::NoFundsSent {})?;
        if creation_fee.amount != config.stream_creation_fee {
            return Err(ContractError::StreamCreationFeeRequired {});
        }
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
        env.block.time,
    );
    let id = next_stream_id(deps.storage)?;
    STREAMS.save(deps.storage, id, &stream)?;

    let attr = vec![
        attr("action", "create_stream"),
        attr("id", id.to_string()),
        attr("treasury", treasury),
        attr("name", name),
        attr("url", url),
        attr("in_denom", in_denom),
        attr("out_denom", out_denom),
        attr("out_supply", out_supply),
        attr("start_time", start_time.to_string()),
        attr("end_time", end_time.to_string()),
    ];
    Ok(Response::default().add_attributes(attr))
}

/// Updates stream to calculate released distribution and spent amount
pub fn execute_update_stream(
    deps: DepsMut,
    env: Env,
    stream_id: u64,
) -> Result<Response, ContractError> {
    let mut stream = STREAMS.load(deps.storage, stream_id)?;
    let (_, dist_amount) = update_stream(env.block.time, &mut stream)?;
    STREAMS.save(deps.storage, stream_id, &stream)?;

    let attrs = vec![
        attr("action", "update_distribution"),
        attr("stream_id", stream_id.to_string()),
        attr("new_distribution_amount", dist_amount),
        attr("dist_index", stream.dist_index.to_string()),
    ];
    let res = Response::new().add_attributes(attrs);
    Ok(res)
}

pub fn update_stream(
    now: Timestamp,
    stream: &mut Stream,
) -> Result<(Decimal, Uint128), ContractError> {
    let (diff, last_updated) = calculate_diff(stream.end_time, stream.last_updated, now);

    let mut new_distribution_balance = Uint128::zero();

    if !stream.shares.is_zero() && !diff.is_zero() {
        new_distribution_balance = stream
            .out_remaining
            .multiply_ratio(diff.numerator(), diff.denominator());
        let spent_in = stream
            .in_supply
            .multiply_ratio(diff.numerator(), diff.denominator());

        stream.spent_in = stream.spent_in.checked_add(spent_in)?;
        stream.in_supply = stream.in_supply.checked_sub(spent_in)?;
        stream.out_remaining = stream.out_remaining.checked_sub(new_distribution_balance)?;
        stream.dist_index = stream.dist_index.checked_add(Decimal256::from_ratio(
            new_distribution_balance,
            stream.shares,
        ))?;

        if !new_distribution_balance.is_zero() {
            stream.current_streamed_price = Decimal::from_ratio(spent_in, new_distribution_balance)
        }
    }

    stream.last_updated = last_updated;

    Ok((diff, new_distribution_balance))
}

fn calculate_diff(
    end_time: Timestamp,
    last_updated: Timestamp,
    now: Timestamp,
) -> (Decimal, Timestamp) {
    let now = if now > end_time { end_time } else { now };
    let numerator = now.minus_nanos(last_updated.nanos());
    let denominator = end_time.minus_nanos(last_updated.nanos());
    if denominator.nanos() == 0 {
        (Decimal::zero(), now)
    } else {
        (
            Decimal::from_ratio(numerator.nanos(), denominator.nanos()),
            now,
        )
    }
}

pub fn execute_update_position(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    stream_id: u64,
    position_owner: Option<String>,
) -> Result<Response, ContractError> {
    // TODO: anyone can trigger?
    let position_owner = maybe_addr(deps.api, position_owner)?.unwrap_or(info.sender.clone());
    let mut position = POSITIONS.load(deps.storage, (stream_id, &position_owner))?;
    if info.sender != position.owner
        && position
            .operator
            .as_ref()
            .map_or(true, |o| o != &info.sender)
    {
        return Err(ContractError::Unauthorized {});
    }

    let mut stream = STREAMS.load(deps.storage, stream_id)?;
    update_stream(env.block.time, &mut stream)?;
    STREAMS.save(deps.storage, stream_id, &stream)?;

    let (purchased, spent) = update_position(
        stream.dist_index,
        stream.shares,
        stream.last_updated,
        stream.in_supply,
        &mut position,
    )?;
    POSITIONS.save(deps.storage, (stream_id, &position.owner), &position)?;

    Ok(Response::new()
        .add_attribute("action", "update_position")
        .add_attribute("stream_id", stream_id.to_string())
        .add_attribute("position_owner", position_owner)
        .add_attribute("purchased", purchased)
        .add_attribute("spent", spent))
}

// calculate the user purchase based on the positions index and the global index.
// returns purchased out amount and spent in amount
pub fn update_position(
    stream_dist_index: Decimal256,
    stream_shares: Uint128,
    stream_last_updated: Timestamp,
    stream_in_supply: Uint128,
    position: &mut Position,
) -> Result<(Uint128, Uint128), ContractError> {
    let index_diff = stream_dist_index.checked_sub(position.index)?;

    let mut spent = Uint128::zero();
    let mut purchased_uint128 = Uint128::zero();

    // if no shares available, means no distribution and no spent
    if !stream_shares.is_zero() {
        let purchased = Decimal256::from_ratio(position.shares, Uint256::one())
            .checked_mul(index_diff)?
            .checked_add(position.pending_purchase)?;
        let decimals = get_decimals(purchased)?;

        let in_remaining = stream_in_supply
            .checked_mul(position.shares)?
            .checked_div(stream_shares)?;

        spent = position.in_balance.checked_sub(in_remaining)?;
        position.spent = position.spent.checked_add(spent)?;
        position.in_balance = in_remaining;
        position.pending_purchase = decimals;

        // floors the decimal points
        purchased_uint128 = (purchased * Uint256::one()).try_into()?;
        position.purchased = position.purchased.checked_add(purchased_uint128)?;
    }

    position.index = stream_dist_index;
    position.last_updated = stream_last_updated;

    Ok((purchased_uint128, spent))
}

pub fn execute_subscribe(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    stream_id: u64,
    operator: Option<String>,
    position_owner: Option<String>,
) -> Result<Response, ContractError> {
    let mut stream = STREAMS.load(deps.storage, stream_id)?;
    if env.block.time < stream.start_time {
        return Err(ContractError::StreamNotStarted {});
    }
    if env.block.time > stream.end_time {
        return Err(ContractError::StreamEnded {});
    }

    let in_amount = must_pay(&info, &stream.in_denom)?;
    let new_shares = stream.compute_shares_amount(in_amount, false);

    let operator = maybe_addr(deps.api, operator)?;
    let position_owner = maybe_addr(deps.api, position_owner)?.unwrap_or(info.sender.clone());
    let position = POSITIONS.may_load(deps.storage, (stream_id, &position_owner))?;
    match position {
        None => {
            // operator cannot create a position in behalf of anyone
            if position_owner != info.sender {
                return Err(ContractError::Unauthorized {});
            }
            update_stream(env.block.time, &mut stream)?;
            // new positions do not update purchase as it has no effect on distribution
            let new_position = Position::new(
                info.sender.clone(),
                in_amount,
                new_shares,
                Some(stream.dist_index),
                env.block.time,
                operator,
            );
            POSITIONS.save(deps.storage, (stream_id, &info.sender), &new_position)?;
        }
        Some(mut position) => {
            if position.owner != info.sender
                && position
                    .operator
                    .as_ref()
                    .map_or(true, |o| o != &info.sender)
            {
                return Err(ContractError::Unauthorized {});
            }

            // incoming tokens should not participate in prev distribution
            update_stream(env.block.time, &mut stream)?;
            update_position(
                stream.dist_index,
                stream.shares,
                stream.last_updated,
                stream.in_supply,
                &mut position,
            )?;

            position.in_balance = position.in_balance.checked_add(in_amount)?;
            position.shares = position.shares.checked_add(new_shares)?;
            POSITIONS.save(deps.storage, (stream_id, &info.sender), &position)?;
        }
    }

    // increase in supply and shares
    stream.in_supply += in_amount;
    stream.shares += new_shares;
    STREAMS.save(deps.storage, stream_id, &stream)?;

    let res = Response::new()
        .add_attribute("action", "subscribe")
        .add_attribute("stream_id", stream_id.to_string())
        .add_attribute("owner", info.sender)
        .add_attribute("in_supply", stream.in_supply)
        .add_attribute("in_amount", in_amount);

    Ok(res)
}

pub fn execute_update_operator(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    stream_id: u64,
    operator: Option<String>,
) -> Result<Response, ContractError> {
    let mut position = POSITIONS.load(deps.storage, (stream_id, &info.sender))?;
    if position.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let operator = maybe_addr(deps.api, operator)?;
    position.operator = operator.clone();

    POSITIONS.save(deps.storage, (stream_id, &info.sender), &position)?;

    Ok(Response::new()
        .add_attribute("action", "update_operator")
        .add_attribute("stream_id", stream_id.to_string())
        .add_attribute("owner", info.sender)
        .add_attribute(
            "operator",
            operator.clone().unwrap_or(Addr::unchecked("")).to_string(),
        ))
}

pub fn execute_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    stream_id: u64,
    cap: Option<Uint128>,
    position_owner: Option<String>,
) -> Result<Response, ContractError> {
    let mut stream = STREAMS.load(deps.storage, stream_id)?;
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

    update_stream(env.block.time, &mut stream)?;
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
        attr("action", "withdraw"),
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
    if stream.last_updated < stream.end_time {
        return Err(ContractError::UpdateDistIndex {});
    }

    let treasury = maybe_addr(deps.api, new_treasury)?.unwrap_or(stream.treasury);

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
        .add_message(fee_send_msg)
        .add_message(send_msg)
        .add_attributes(attributes))
}

pub fn execute_exit_stream(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    stream_id: u64,
    position_owner: Option<String>,
) -> Result<Response, ContractError> {
    let mut stream = STREAMS.load(deps.storage, stream_id)?;
    if env.block.time < stream.end_time {
        return Err(ContractError::StreamNotEnded {});
    }
    if stream.last_updated < stream.end_time {
        return Err(ContractError::UpdateDistIndex {});
    }

    let position_owner = maybe_addr(deps.api, position_owner)?.unwrap_or(info.sender.clone());
    let mut position = POSITIONS.load(deps.storage, (stream_id, &position_owner))?;
    // TODO: maybe callable by everyone? if then remove new recipient
    if position.owner != info.sender
        && position
            .operator
            .as_ref()
            .map_or(true, |o| o != &info.sender)
    {
        return Err(ContractError::Unauthorized {});
    }

    // update position before exit
    update_position(
        stream.dist_index,
        stream.shares,
        stream.last_updated,
        stream.in_supply,
        &mut position,
    )?;

    let send_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: position_owner.to_string(),
        amount: vec![Coin {
            denom: stream.out_denom.to_string(),
            amount: position.purchased,
        }],
    });
    stream.shares -= position.shares;
    stream.in_supply -= position.in_balance;
    STREAMS.save(deps.storage, stream_id, &stream)?;
    POSITIONS.remove(deps.storage, (stream_id, &position.owner));

    let attributes = vec![
        attr("action", "exit_stream"),
        attr("stream_id", stream_id.to_string()),
        attr("purchased", position.purchased),
    ];
    Ok(Response::new()
        .add_message(send_msg)
        .add_attributes(attributes))
}

// TODO: finalize already sends back the fees, so this is not needed, but in case there is extra fees
// best to keep this
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

    cfg.min_stream_seconds = min_stream_duration.unwrap_or(cfg.min_stream_seconds);
    cfg.min_seconds_until_start_time =
        min_duration_until_start_time.unwrap_or(cfg.min_seconds_until_start_time);
    cfg.stream_creation_denom = stream_creation_denom.unwrap_or(cfg.stream_creation_denom);
    cfg.stream_creation_fee = stream_creation_fee.unwrap_or(cfg.stream_creation_fee);

    let collector = maybe_addr(deps.api, fee_collector)?.unwrap_or(cfg.fee_collector);
    cfg.fee_collector = collector;

    CONFIG.save(deps.storage, &cfg)?;

    let attributes = vec![
        attr("action", "update_config"),
        attr("min_stream_duration", cfg.min_stream_seconds),
        attr(
            "min_duration_until_start_time",
            cfg.min_seconds_until_start_time,
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
        in_denom: stream.in_denom,
        out_denom: stream.out_denom,
        out_supply: stream.out_supply,
        start_time: stream.start_time,
        end_time: stream.end_time,
        in_spent: stream.spent_in,
        dist_index: stream.dist_index,
        out_remaining: stream.out_remaining,
        in_supply: stream.in_supply,
        shares: stream.shares,
        last_updated: stream.last_updated,
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
                in_denom: stream.in_denom,
                out_denom: stream.out_denom,
                out_supply: stream.out_supply,
                start_time: stream.start_time,
                end_time: stream.end_time,
                in_spent: stream.spent_in,
                last_updated: stream.last_updated,
                dist_index: stream.dist_index,
                out_remaining: stream.out_remaining,
                in_supply: stream.in_supply,
                shares: stream.shares,
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
        index: position.index,
        spent: position.spent,
        shares: position.shares,
        operator: position.operator,
        last_updated: position.last_updated,
        pending_purchase: position.pending_purchase,
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
            let (owner, position) = item?;
            let position = PositionResponse {
                stream_id,
                owner: owner.to_string(),
                index: position.index,
                last_updated: position.last_updated,
                purchased: position.purchased,
                pending_purchase: position.pending_purchase,
                spent: position.spent,
                in_balance: position.in_balance,
                shares: position.shares,
                operator: position.operator,
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
    let total_purchased = stream.out_supply - stream.out_remaining;
    let average_price = Decimal::from_ratio(stream.spent_in, total_purchased);
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
