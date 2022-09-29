use cosmwasm_std::{attr, entry_point, from_binary, to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128, WasmMsg, WasmQuery, BalanceResponse, Uint64};

use crate::msg::{
    AccruedRewardsResponse, ExecuteMsg, PositionResponse, HoldersResponse, InstantiateMsg,
    MigrateMsg, QueryMsg, ReceiveMsg, StateResponse,
};
use crate::state::{list_positions, Position, State, CLAIMS, POSITIONS, STATE};
use crate::ContractError;
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, Cw20ReceiveMsg};
use cw_controllers::ClaimsResponse;
use std::ops::{Add, Div, Mul, Sub};
use std::str::FromStr;
use cw_utils::Scheduled;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    deps.api.addr_validate(&msg.cw20_token_addr.as_str())?;

    // check scheduled start and end are same types
    match (msg.start_time, msg.end_time) {
        (Scheduled::AtTime(_), Scheduled::AtHeight(_)) => Err(ContractError::DateInput {}),
        (Scheduled::AtHeight(_), Scheduled::AtTime(_)) => Err(ContractError::DateInput {}),
        (_,_) => Ok(())
    }?;

    let state = State {
        latest_distribution_stage: Uint64::zero(),
        global_distribution_index: Decimal::zero(),
        token_out_supply: Uint64::zero(),
        start_time: Uint64::new(msg.start_time.nanos()),
        end_time: Uint64::new(msg.end_time.nanos()),
        total_buy: Uint64::zero(),
        total_out_sold: Uint64::zero(),
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::TriggerPositionPurchase { addr: String } => execute_trigger_user_purchase(deps, env, info, addr),
        ExecuteMsg::UpdateDistributionIndex {} => execute_update_distribution_index(deps, env),
        ExecuteMsg::Subscribe {} => execute_subscribe(deps, env, info),
        ExecuteMsg::Withdraw { cap } => execute_withdraw(deps, env, info, cap),
    }
}

/// Increase global_distribution_index with new distribution release
pub fn execute_update_distribution_index(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;

    let current_distribution_stage = (Uint64::new(env.block.time.nanos()) - state.start_time) / (state.end_time - state.start_time);
    let diff = current_distribution_stage - state.latest_distribution_stage;

    let new_distribution_balance = diff.mul(state.token_out_supply);

    state.global_distribution_index = state
        .global_distribution_index
        .add(Decimal::from_ratio(new_distribution_balance, state.total_buy));

    state.latest_distribution_stage = current_distribution_stage;
    STATE.save(deps.storage, &state)?;

    let res = Response::new()
        .add_attribute("action", "update_distribution_index")
        .add_attribute("new_distribution_balance", new_distribution_balance)
        .add_attribute("global_release_index", state.global_distribution_index.to_string());

    Ok(res)
}

pub fn execute_trigger_user_purchase(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    addr: Option<String>
) -> Result<Response, ContractError> {
    let addr = match addr {
        Some(value) => deps.api.addr_validate(value.as_str())?,
        None => info.sender,
    };

    let mut state = STATE.load(deps.storage)?;
    let position = POSITIONS.load(deps.storage, &addr)?;

    let purchase =
        trigger_position_purchase(state.global_distribution_index, position.index, position.buy_balance)?;
    let all_purchased = purchase.add(position.purchased);
    let all_purchased_u64 = all_purchased * Uint64::new(1);

    if all_purchased_u64.is_zero() {
        return Err(ContractError::NoDistribution {});
    }

    let new_balance = (state.prev_reward_balance.checked_sub(rewards))?;
    state.prev_reward_balance = new_balance;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "claim_rewards")
        .add_attribute("recipient", recipient))
}

pub fn execute_subscribe(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    if info.funds.is_empty() {
        return Err(ContractError::NoFundsSent {});
    }

    let mut state = STATE.load(deps.storage)?;

    let mut position = POSITIONS.may_load(deps.storage, &addr)?.unwrap_or(Position {
        buy_balance: Uint128::zero(),
        index: Decimal::zero(),
        purchased: Uint128::zero(),
    });

    // get decimals
    let distribution = trigger_position_purchase(state.global_distribution_index, position.index, position.buy_balance)?;
    let remaining = state.total_buy;

    position.index = state.global_distribution_index;
    position.purchased = distribution.sub(position.purchased);
    position.buy_balance = amount;
    // save reward and index
    POSITIONS.save(deps.storage, &addr, &position)?;

    state.total_buy += amount;
    STATE.save(deps.storage, &state)?;

    let res = Response::new()
        .add_attribute("action", "bond_stake")
        .add_attribute("holder_address", holder_addr)
        .add_attribute("amount", amount);

    Ok(res)
}

pub fn execute_withdraw(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    cap: Option<Uint128>,
) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;

    if !info.funds.is_empty() {
        return Err(ContractError::NoFundsSent {});
    }
    if amount.is_zero() {
        return Err(ContractError::AmountRequired {});
    }

    let mut holder = POSITIONS.load(deps.storage, &info.sender)?;
    if holder.buy_balance < amount {
        return Err(ContractError::DecreaseAmountExceeds(holder.buy_balance));
    }

    let rewards = trigger_position_purchase(state.global_distribution_index, holder.index, holder.buy_balance)?;

    holder.index = state.global_distribution_index;
    holder.purchased = rewards.add(holder.purchased);
    holder.buy_balance = (holder.buy_balance.checked_sub(amount))?;
    state.token_out_supply = (state.token_out_supply.checked_sub(amount))?;

    STATE.save(deps.storage, &state)?;
    POSITIONS.save(deps.storage, &info.sender, &holder)?;

    let attributes = vec![
        attr("action", "unbound"),
        attr("holder_address", info.sender),
        attr("amount", amount),
    ];

    Ok(Response::new().add_attributes(attributes))
}

// calculate the user purchase based on the positions index and the global index.
// returns purchase amount and spent amount
pub fn trigger_position_purchase(
    global_index: Decimal,
    user_index: Decimal,
    user_balance: Uint128,
) -> StdResult<(Decimal, Decimal)> {
    let decimal_balance = Decimal::from_ratio(user_balance, Uint128::new(1));

    let diff = global_index.sub(user_index);

    let purchased = diff.mul(decimal_balance);
    let spent = diff.div(global_index).mul(decimal_balance);
    Ok((purchased, spent))
}

// calculate the reward with decimal
pub fn get_decimals(value: Decimal) -> StdResult<Decimal> {
    let stringed: &str = &*value.to_string();
    let parts: &[&str] = &*stringed.split('.').collect::<Vec<&str>>();
    match parts.len() {
        1 => Ok(Decimal::zero()),
        2 => {
            let decimals = Decimal::from_str(&*("0.".to_owned() + parts[1]))?;
            Ok(decimals)
        }
        _ => Err(StdError::generic_err("Unexpected number of dots")),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    /*
    match msg {
        QueryMsg::State {} => to_binary(&query_state(deps, _env, msg)?),
        QueryMsg::AccruedDistribution { address } => to_binary(&query_accrued_rewards(deps, address)?),
        QueryMsg::Holder { address } => to_binary(&query_holder(deps, address)?),
        QueryMsg::Holders { start_after, limit } => {
            to_binary(&query_holders(deps, start_after, limit)?)
        }
        QueryMsg::Claims { address } => to_binary(&query_claims(deps, address)?),
    }
     */
    unimplemented!()
}

/*
pub fn query_state(deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;

    Ok(StateResponse {
        cw20_token_addr: state.cw20_token_addr,
        unbonding_period: state.unbonding_period,
        global_index: state.global_distribution_index,
        total_balance: state.token_out_supply,
        prev_reward_balance: state.latest_distribution_stage,
    })
}

pub fn query_accrued_rewards(deps: Deps, address: String) -> StdResult<AccruedRewardsResponse> {
    let state = STATE.load(deps.storage)?;

    let addr = deps.api.addr_validate(address.as_str())?;
    let holder = POSITIONS.load(deps.storage, &addr)?;
    let reward_with_decimals =
        trigger_position_purchase(state.global_distribution_index, holder.index, holder.buy_balance)?;
    let all_reward_with_decimals = reward_with_decimals.add(holder.purchased);

    let rewards = all_reward_with_decimals * Uint128::new(1);

    Ok(AccruedRewardsResponse { rewards })
}

pub fn query_holder(deps: Deps, address: String) -> StdResult<HolderResponse> {
    let holder: Position = POSITIONS.load(deps.storage, &deps.api.addr_validate(address.as_str())?)?;
    Ok(HolderResponse {
        address,
        balance: holder.buy_balance,
        index: holder.index,
        pending_rewards: holder.purchased,
    })
}

pub fn query_holders(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<HoldersResponse> {
    let start_after = if let Some(start_after) = start_after {
        Some(deps.api.addr_validate(&start_after)?)
    } else {
        None
    };

    let holders: Vec<HolderResponse> = list_accrued_rewards(deps, start_after, limit)?;

    Ok(HoldersResponse { holders })
}

pub fn query_claims(deps: Deps, addr: String) -> StdResult<ClaimsResponse> {
    Ok(CLAIMS.query_claims(deps, &deps.api.addr_validate(addr.as_str())?)?)
}

 */
