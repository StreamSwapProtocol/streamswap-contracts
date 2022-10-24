use cosmwasm_std::{
    attr, entry_point, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Timestamp, Uint128, Uint64,
};

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{next_sales_id, Position, Sale, POSITIONS, SALES};
use crate::ContractError;

use cw_utils::must_pay;
use std::ops::Mul;

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

    let state = Sale {
        latest_stage: Decimal::zero(),
        dist_index: Decimal::zero(),
        start_time: Uint64::new(msg.start_time.nanos()),
        end_time: Uint64::new(msg.end_time.nanos()),
        token_out_denom: msg.token_out_denom,
        token_out_supply: msg.token_out_supply,
        total_out_sold: Uint128::zero(),
        token_in_denom: msg.token_in_denom,
        total_in_supply: Uint128::zero(),
        total_in_spent: Uint128::zero(),
    };
    let id = next_sales_id(deps.storage)?;
    SALES.save(deps.storage, id, &state)?;

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
        ExecuteMsg::TriggerPositionPurchase { sale_id } => {
            execute_trigger_purchase(deps, env, info, sale_id)
        }
        ExecuteMsg::UpdateDistributionIndex { sale_id } => {
            execute_update_dist_index(deps, env, sale_id)
        }
        ExecuteMsg::Subscribe { sale_id } => execute_subscribe(deps, env, info, sale_id),
        ExecuteMsg::Withdraw {
            sale_id,
            cap,
            recipient,
        } => execute_withdraw(deps, env, info, cap, sale_id, recipient),
    }
}

/// Increase global_distribution_index with new distribution release
pub fn execute_update_dist_index(
    deps: DepsMut,
    env: Env,
    sale_id: u64,
) -> Result<Response, ContractError> {
    let mut sale = SALES.load(deps.storage, sale_id)?;
    let (_, dist_amount) = update_dist_index(env.block.time, &mut sale)?;
    SALES.save(deps.storage, sale_id, &sale)?;

    let attrs = vec![
        attr("action", "update_distribution_index"),
        attr("sale_id", sale_id.to_string()),
        attr("distribution_amount", dist_amount),
        attr("sale_dist_index", sale.dist_index.to_string()),
    ];
    let res = Response::new().add_attributes(attrs);
    Ok(res)
}

pub fn update_dist_index(
    now: Timestamp,
    sale: &mut Sale,
) -> Result<(Decimal, Uint128), ContractError> {
    // calculate the current distribution stage
    let numerator = Decimal::new(Uint128::from(now.nanos()) - Uint128::from(sale.start_time));
    let denominator = Decimal::new(Uint128::from(sale.end_time - sale.start_time));
    let current_dist_stage = numerator / denominator;

    // calculate new distribution
    let diff = current_dist_stage.checked_sub(sale.latest_stage)?;
    let new_distribution_balance = diff.mul(sale.token_out_supply);
    let spent_buy_side = diff.mul(sale.total_in_supply);

    let deduced_buy_supply = sale.total_in_supply.checked_sub(spent_buy_side)?;

    sale.dist_index += Decimal::from_ratio(new_distribution_balance, deduced_buy_supply);
    sale.latest_stage = current_dist_stage;
    sale.total_in_spent += spent_buy_side;
    sale.total_in_supply = deduced_buy_supply;

    Ok((diff, new_distribution_balance))
}

pub fn execute_trigger_purchase(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sale_id: u64,
) -> Result<Response, ContractError> {
    let addr = info.sender;

    let mut sale = SALES.load(deps.storage, sale_id)?;
    let (_, _) = update_dist_index(env.block.time, &mut sale)?;
    SALES.save(deps.storage, sale_id, &sale)?;

    let mut position = POSITIONS.load(deps.storage, (sale_id, &addr))?;
    let (purchased, spent) =
        trigger_update_purchase(sale.dist_index, sale.latest_stage, &mut position)?;
    POSITIONS.save(deps.storage, (sale_id, &position.owner), &position)?;

    Ok(Response::new()
        .add_attribute("action", "trigger_position_purchase")
        .add_attribute("recipient", addr)
        .add_attribute("purchased", purchased)
        .add_attribute("spent", spent))
}

// calculate the user purchase based on the positions index and the global index.
// returns purchase amount and spent amount
pub fn trigger_update_purchase(
    sale_dist_index: Decimal,
    sale_latest_stage: Decimal,
    position: &mut Position,
) -> Result<(Uint128, Uint128), ContractError> {
    let index_diff = sale_dist_index.checked_sub(position.index)?;
    let purchased = position.buy_balance.mul(index_diff);

    let spent_diff = sale_latest_stage - position.latest_dist_stage;
    let spent = spent_diff.mul(position.buy_balance);

    position.buy_balance -= spent;
    position.latest_dist_stage = sale_latest_stage;
    position.purchased += purchased;
    position.spent += spent;
    position.index = sale_dist_index;

    Ok((purchased, spent))
}

pub fn execute_subscribe(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sale_id: u64,
) -> Result<Response, ContractError> {
    let mut sale = SALES.load(deps.storage, sale_id)?;
    let funds = must_pay(&info, &sale.token_in_denom)?;

    let position = POSITIONS.may_load(deps.storage, (sale_id, &info.sender))?;
    match position {
        None => {
            let new_position = Position {
                owner: info.sender.clone(),
                buy_balance: funds,
                index: sale.dist_index,
                latest_dist_stage: Decimal::zero(),
                purchased: Uint128::zero(),
                spent: Uint128::zero(),
            };
            POSITIONS.save(deps.storage, (sale_id, &info.sender), &new_position)?;
        }
        Some(mut position) => {
            if position.owner != info.sender {
                return Err(ContractError::Unauthorized {});
            }

            update_dist_index(env.block.time, &mut sale)?;

            trigger_update_purchase(sale.dist_index, sale.latest_stage, &mut position)?;
            position.buy_balance += funds;
            POSITIONS.save(deps.storage, (sale_id, &info.sender), &position)?;
        }
    }

    sale.total_in_supply += funds;
    SALES.save(deps.storage, sale_id, &sale)?;

    // TODO: refactor attributes
    let res = Response::new()
        .add_attribute("action", "subscribe")
        .add_attribute("sale_id", sale_id.to_string())
        .add_attribute("owner", info.sender)
        .add_attribute("total_in_supply", sale.total_in_supply)
        .add_attribute("subscribe_amount", funds);

    Ok(res)
}

pub fn execute_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Option<Uint128>,
    sale_id: u64,
    recipient: Option<String>,
) -> Result<Response, ContractError> {
    let mut sale = SALES.load(deps.storage, sale_id)?;

    let mut position = POSITIONS.load(deps.storage, (sale_id, &info.sender))?;
    if position.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    update_dist_index(env.block.time, &mut sale)?;
    SALES.save(deps.storage, sale_id, &sale)?;

    let (purchased, spent) =
        trigger_update_purchase(sale.dist_index, sale.latest_stage, &mut position)?;
    POSITIONS.save(deps.storage, (sale_id, &position.owner), &position)?;
    let withdraw_amount = amount.unwrap_or(position.buy_balance - spent);

    // if amount to withdraw more then deduced buy balance throw error
    if withdraw_amount > position.buy_balance - spent {
        return Err(ContractError::DecreaseAmountExceeds(withdraw_amount));
    }

    sale.total_out_sold += purchased;
    sale.total_in_spent += spent;
    SALES.save(deps.storage, sale_id, &sale)?;

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
                denom: sale.token_in_denom,
                amount: withdraw_amount,
            }],
        }))
        .add_attributes(attributes);

    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}
