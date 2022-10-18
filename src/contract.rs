use cosmwasm_std::{
    attr, entry_point, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Storage, Timestamp, Uint128, Uint64,
};

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Position, State, POSITIONS, STATE};
use crate::ContractError;

use cw_utils::must_pay;
use std::ops::{Mul};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // TODO: check start_time end time

    let funds = must_pay(&info, msg.token_out_denom.as_str())?;
    if funds != msg.token_out_supply {
        return Err(ContractError::AmountRequired {});
    }

    let state = State {
        latest_dist_stage: Decimal::zero(),
        global_dist_index: Decimal::zero(),
        start_time: Uint64::new(msg.start_time.nanos()),
        end_time: Uint64::new(msg.end_time.nanos()),
        token_out_denom: msg.token_out_denom,
        token_out_supply: msg.token_out_supply,
        total_out_sold: Uint128::zero(),
        token_in_denom: msg.token_in_denom,
        total_in_supply: Uint128::zero(),
        total_in_spent: Uint128::zero(),
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
        ExecuteMsg::TriggerPositionPurchase {} => {
            execute_trigger_position_purchase(deps, env, info)
        }
        ExecuteMsg::UpdateDistributionIndex {} => execute_update_distribution_index(deps, env),
        ExecuteMsg::Subscribe {} => execute_subscribe(deps, env, info),
        ExecuteMsg::Withdraw { cap } => execute_withdraw(deps, env, info, cap),
    }
}

/// Increase global_distribution_index with new distribution release
pub fn execute_update_distribution_index(
    deps: DepsMut,
    env: Env,
) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;
    let (_, new_distribution_balance) = update_distribution_index(env.block.time, &mut state)?;
    // need new_distribution_balance, global_distribution_index,
    STATE.save(deps.storage, &state)?;

    let res = Response::new()
        .add_attribute("action", "update_distribution_index")
        .add_attribute("new_distribution_balance", new_distribution_balance)
        .add_attribute(
            "global_distribution_index",
            state.global_dist_index.to_string(),
        );

    Ok(res)
}

pub fn update_distribution_index(
    now: Timestamp,
    current_state: &mut State,
) -> Result<(Decimal, Uint128), ContractError> {
    // calculate the current distribution stage
    let numerator =
        Decimal::new(Uint128::from(now.nanos()) - Uint128::from(current_state.start_time));
    let denominator = Decimal::new(Uint128::from(
        current_state.end_time - current_state.start_time,
    ));
    // %30
    // user A creates position
    // treasury total: 2000
    // distribution_bal = 600
    // cds = 3/10
    // position_total: 1000
    // position_spent = 300
    // new position total = 700
    // new_dist_index = 600 / 700: 6/7
    // streaming_price =  700/600: 1.1

    // %60
    // user B creates position
    // distribution_bal = 600
    // position_total = 200
    // distribution_index = 6/7
    // position_spent =
    let current_dist_stage = numerator / denominator;

    // calculate new distribution
    let diff = current_dist_stage.checked_sub(current_state.latest_dist_stage)?;
    let new_distribution_balance = diff.mul(current_state.token_out_supply);
    let spent_buy_side = diff.mul(current_state.total_in_supply);

    // TODO: check calculation
    let deduced_buy_supply = current_state.total_in_supply.checked_sub(spent_buy_side)?;

    current_state.global_dist_index +=
        Decimal::from_ratio(new_distribution_balance, deduced_buy_supply);
    current_state.latest_dist_stage = current_dist_stage;
    current_state.total_in_spent += spent_buy_side;
    current_state.total_in_supply = deduced_buy_supply;

    Ok((diff, new_distribution_balance))
}

pub fn execute_trigger_position_purchase(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let addr = info.sender;

    let position = POSITIONS.load(deps.storage, &addr)?;
    let (purchased, spent) = update_position_sale(deps.storage, env.block.time, position)?;

    Ok(Response::new()
        .add_attribute("action", "trigger_position_purchase")
        .add_attribute("recipient", addr)
        .add_attribute("purchased", purchased)
        .add_attribute("spent", spent))
}

// calculate the user purchase based on the positions index and the global index.
// returns purchase amount and spent amount
pub fn update_position_sale(
    storage: &mut dyn Storage,
    now: Timestamp,
    // TODO: revisit design
    mut position: Position,
) -> Result<(Uint128, Uint128), ContractError> {
    let mut state = STATE.load(storage)?;
    // update distribution index
    let (_, _) = update_distribution_index(now, &mut state)?;

    STATE.save(storage, &state)?;

    let index_diff = state
        .global_dist_index
        .checked_sub(position.index)?;

    let spent_diff = state.latest_dist_stage - position.latest_dist_stage;
    let spent = spent_diff.mul(position.buy_balance);
    position.buy_balance -= spent;

    let purchased = position.buy_balance.mul(index_diff);

    position.latest_dist_stage = state.latest_dist_stage;
    position.purchased += purchased;
    position.spent += spent;
    position.index = state.global_dist_index;

    POSITIONS.save(storage, &position.owner, &position)?;

    Ok((purchased, spent))
}

pub fn execute_subscribe(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    let funds = must_pay(&info, &state.token_in_denom)?;

    let mut state = STATE.load(deps.storage)?;

    let position = POSITIONS.may_load(deps.storage, &info.sender)?;
    match position {
        None => {
            let new_position = Position {
                owner: info.sender.clone(),
                buy_balance: funds,
                index: state.global_dist_index,
                latest_dist_stage: Decimal::zero(),
                purchased: Uint128::zero(),
                spent: Uint128::zero(),
            };
            POSITIONS.save(deps.storage, &info.sender, &new_position)?;
        }
        Some(position) => {
            update_position_sale(deps.storage, env.block.time, position.clone())?;
        }
    }

    state.total_in_supply += funds;
    STATE.save(deps.storage, &state)?;

    let res = Response::new()
        .add_attribute("action", "subscribe")
        .add_attribute("position_owner", info.sender)
        .add_attribute("total_buy", state.total_in_supply)
        .add_attribute("buy_amount", funds);

    Ok(res)
}

pub fn execute_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;

    if !info.funds.is_empty() {
        return Err(ContractError::NoFundsSent {});
    }

    let mut position = POSITIONS.load(deps.storage, &info.sender)?;
    let amount_to_withdraw = amount.unwrap_or(position.buy_balance);

    let (purchased, spent) = update_position_sale(deps.storage, env.block.time, position.clone())?;

    // if amount to withdraw more then deduced buy balance throw error
    if amount_to_withdraw > position.buy_balance - spent {
        return Err(ContractError::DecreaseAmountExceeds(amount_to_withdraw));
    }

    position.index = state.global_dist_index;
    position.purchased += purchased;
    position.spent += spent;
    position.buy_balance -= spent;
    POSITIONS.save(deps.storage, &info.sender, &position)?;

    state.total_out_sold += purchased;
    state.total_in_spent += spent;
    STATE.save(deps.storage, &state)?;

    let attributes = vec![
        attr("action", "unbound"),
        attr("holder_address", info.sender),
        attr("withdraw_amount", amount_to_withdraw),
    ];

    Ok(Response::new().add_attributes(attributes))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
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
