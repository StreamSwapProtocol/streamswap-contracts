use crate::msg::{
    AveragePriceResponse, ExecuteMsg, InstantiateMsg, LatestStreamedPriceResponse, MigrateMsg,
    PositionResponse, PositionsResponse, QueryMsg, SaleResponse, SalesResponse, SudoMsg,
};
use crate::state::{next_sales_id, Config, Position, Sale, CONFIG, POSITIONS, SALES};
use crate::ContractError;
use cosmwasm_std::{
    attr, entry_point, to_binary, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MessageInfo, Order, Response, StdResult, Timestamp, Uint128, Uint64,
};

use cw_storage_plus::Bound;
use cw_utils::{maybe_addr, must_pay};
use std::ops::Mul;
use crate::math::{decimal_multiplication_in_256, decimal_subtraction_in_256};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        min_sale_duration: msg.min_sale_duration,
        min_duration_until_start_time: msg.min_duration_until_start_time,
        sale_creation_denom: msg.sale_creation_denom,
        sale_creation_fee: msg.sale_creation_fee,
        fee_collector: deps.api.addr_validate(&msg.fee_collector)?,
    };
    CONFIG.save(deps.storage, &config)?;

    let attrs = vec![
        attr("min_sale_duration", msg.min_sale_duration),
        attr(
            "min_duration_until_start_time",
            msg.min_duration_until_start_time,
        ),
        attr("sale_creation_fee", msg.sale_creation_fee),
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
        ExecuteMsg::CreateSale {
            treasury,
            name,
            url,
            token_in_denom,
            token_out_denom,
            token_out_supply,
            start_time,
            end_time,
        } => execute_create_sale(
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
        ExecuteMsg::FinalizeSale {
            sale_id,
            new_treasury,
        } => execute_finalize_sale(deps, env, info, sale_id, new_treasury),
        ExecuteMsg::ExitSale { sale_id, recipient } => {
            execute_finalize_sale(deps, env, info, sale_id, recipient)
        }
    }
}

pub fn execute_create_sale(
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
    if end_time.nanos() - start_time.nanos() < config.min_sale_duration.u64() {
        return Err(ContractError::SaleDurationTooShort {});
    }

    if start_time.nanos() - env.block.time.nanos() < config.min_duration_until_start_time.u64() {
        return Err(ContractError::SaleStartsTooSoon {});
    }

    let funds = must_pay(&info, out_denom.as_str())?;
    if funds != out_supply {
        return Err(ContractError::AmountRequired {});
    }

    let creation_fee = must_pay(&info, config.sale_creation_denom.as_str())?;
    if creation_fee != config.sale_creation_fee {
        return Err(ContractError::CreationFeeRequired {});
    }

    let state = Sale {
        treasury: deps.api.addr_validate(&treasury)?,
        current_stage: Decimal::zero(),
        dist_index: Decimal::zero(),
        start_time: Uint64::new(start_time.nanos()),
        end_time: Uint64::new(end_time.nanos()),
        out_denom: out_denom.clone(),
        out_supply: out_supply.clone(),
        current_out: Uint128::zero(),
        in_denom: in_denom.clone(),
        in_supply: Uint128::zero(),
        current_in: Uint128::zero(),
        latest_streamed_price: Uint128::zero(),
    };
    let id = next_sales_id(deps.storage)?;
    SALES.save(deps.storage, id, &state)?;

    let attr = vec![
        attr("action", "create_sale"),
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
    // if now is after end_time, set now to end_time
    let now = if now.nanos() > sale.end_time.u64() {
        sale.end_time.u64()
    } else {
        now.nanos()
    };

    // calculate the current distribution stage
    // dist stage is the (now - start) / (end - start), gives %0-100 percentage
    let numerator = now - sale.start_time.u64();
    let denominator = sale.end_time - sale.start_time;
    let current_dist_stage = Decimal::from_ratio(numerator, denominator);

    // calculate new distribution
    let stage_diff = decimal_subtraction_in_256(current_dist_stage, sale.current_stage);

    let new_distribution_balance = stage_diff.mul(sale.out_supply);
    let spent_in = stage_diff.mul(sale.in_supply);
    let deducted_in_supply = sale.in_supply.checked_sub(spent_in)?;

    sale.dist_index += Decimal::from_ratio(new_distribution_balance, deducted_in_supply);
    sale.current_stage = current_dist_stage;
    sale.current_in += spent_in;
    sale.in_supply = deducted_in_supply;

    sale.latest_streamed_price = new_distribution_balance / spent_in;

    Ok((stage_diff, new_distribution_balance))
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
        trigger_purchase(sale.dist_index, sale.current_stage, &mut position)?;
    POSITIONS.save(deps.storage, (sale_id, &position.owner), &position)?;

    Ok(Response::new()
        .add_attribute("action", "trigger_position_purchase")
        .add_attribute("recipient", addr)
        .add_attribute("purchased", purchased)
        .add_attribute("spent", spent))
}

// calculate the user purchase based on the positions index and the global index.
// returns purchased out amount and spent in amount
pub fn trigger_purchase(
    sale_dist_index: Decimal,
    sale_latest_stage: Decimal,
    position: &mut Position,
) -> Result<(Uint128, Uint128), ContractError> {
    let index_diff = decimal_subtraction_in_256(index_diff, position.index);
    let spent_diff = decimal_subtraction_in_256(sale_latest_stage, position.latest_dist_stage);

    let spent = spent_diff.mul(position.in_balance);

    // update buy balance with spent tokens before calculating purchase, to correct for supply reduce
    // on update distribution index
    position.in_balance -= spent;
    let purchased = position.in_balance.mul(index_diff)?;

    position.index = sale_dist_index;
    position.in_balance -= spent;
    position.latest_dist_stage = sale_latest_stage;
    position.purchased += purchased;
    position.spent += spent;

    Ok((purchased, spent))
}

pub fn execute_subscribe(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sale_id: u64,
) -> Result<Response, ContractError> {
    let mut sale = SALES.load(deps.storage, sale_id)?;
    let in_amount = must_pay(&info, &sale.in_denom)?;

    // if option exists, update the distribution index
    // else create subscription
    let position = POSITIONS.may_load(deps.storage, (sale_id, &info.sender))?;
    match position {
        None => {
            let new_position = Position {
                owner: info.sender.clone(),
                in_balance: in_amount,
                index: sale.dist_index,
                latest_dist_stage: Decimal::zero(),
                purchased: Uint128::zero(),
                spent: Uint128::zero(),
                exited: false,
            };
            POSITIONS.save(deps.storage, (sale_id, &info.sender), &new_position)?;
        }
        Some(mut position) => {
            if position.owner != info.sender {
                return Err(ContractError::Unauthorized {});
            }

            update_dist_index(env.block.time, &mut sale)?;

            trigger_purchase(sale.dist_index, sale.current_stage, &mut position)?;
            position.in_balance += in_amount;
            POSITIONS.save(deps.storage, (sale_id, &info.sender), &position)?;
        }
    }

    // increase in supply
    sale.in_supply += in_amount;
    SALES.save(deps.storage, sale_id, &sale)?;

    let res = Response::new()
        .add_attribute("action", "subscribe")
        .add_attribute("sale_id", sale_id.to_string())
        .add_attribute("owner", info.sender)
        .add_attribute("in_supply", sale.in_supply)
        .add_attribute("in_amount", in_amount);

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
        trigger_purchase(sale.dist_index, sale.current_stage, &mut position)?;
    POSITIONS.save(deps.storage, (sale_id, &position.owner), &position)?;
    let withdraw_amount = amount.unwrap_or(position.in_balance - spent);

    // if amount to withdraw more then deduced buy balance throw error
    if withdraw_amount > position.in_balance - spent {
        return Err(ContractError::DecreaseAmountExceeds(withdraw_amount));
    }

    sale.current_out += purchased;
    sale.current_in += spent;
    sale.in_supply -= withdraw_amount;
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
                denom: sale.in_denom,
                amount: withdraw_amount,
            }],
        }))
        .add_attributes(attributes);

    Ok(res)
}

pub fn execute_finalize_sale(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sale_id: u64,
    new_treasury: Option<String>,
) -> Result<Response, ContractError> {
    let sale = SALES.load(deps.storage, sale_id)?;

    if sale.treasury != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    if env.block.time.nanos() < sale.end_time.u64() {
        return Err(ContractError::SaleNotEnded {});
    }

    if sale.current_stage < Decimal::one() {
        return Err(ContractError::UpdateDistIndex {});
    }

    let treasury = new_treasury
        .map(|t| deps.api.addr_validate(&t))
        .transpose()?
        .unwrap_or(sale.treasury);

    let send_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: treasury.to_string(),
        amount: vec![Coin {
            denom: sale.in_denom,
            amount: sale.current_in,
        }],
    });

    let config = CONFIG.load(deps.storage)?;
    let fee_send_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: config.fee_collector.to_string(),
        amount: vec![Coin {
            denom: config.sale_creation_denom,
            amount: config.sale_creation_fee,
        }],
    });

    let attributes = vec![
        attr("action", "finalize_sale"),
        attr("sale_id", sale_id.to_string()),
        attr("treasury", treasury.as_str()),
        attr("total_in_spent", sale.current_in),
    ];

    Ok(Response::new()
        .add_message(send_msg)
        .add_message(fee_send_msg)
        .add_attributes(attributes))
}

pub fn execute_exit_sale(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sale_id: u64,
    recipient: Option<String>,
) -> Result<Response, ContractError> {
    let sale = SALES.load(deps.storage, sale_id)?;
    if env.block.time.nanos() < sale.end_time.u64() {
        return Err(ContractError::SaleNotEnded {});
    }

    if sale.current_stage < Decimal::one() {
        return Err(ContractError::UpdateDistIndex {});
    }

    let mut position = POSITIONS.load(deps.storage, (sale_id, &info.sender))?;

    if position.latest_dist_stage < Decimal::one() {
        return Err(ContractError::TriggerPositionPurchase {});
    }

    if position.exited {
        return Err(ContractError::PositionAlreadyExited {});
    }

    position.exited = true;
    POSITIONS.save(deps.storage, (sale_id, &position.owner), &position)?;

    let recipient = recipient
        .map(|r| deps.api.addr_validate(&r))
        .transpose()?
        .unwrap_or(position.owner.clone());

    let send_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: recipient.to_string(),
        amount: vec![Coin {
            denom: sale.out_denom,
            amount: position.purchased,
        }],
    });
    let attributes = vec![
        attr("action", "exit_sale"),
        attr("recipient", recipient.as_str()),
        attr("purchased", position.purchased),
    ];
    Ok(Response::new()
        .add_message(send_msg)
        .add_attributes(attributes))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::UpdateConfig {
            min_sale_duration,
            min_duration_until_start_time,
            sale_creation_denom,
            sale_creation_fee,
            fee_collector,
        } => sudo_update_config(
            deps,
            env,
            min_sale_duration,
            min_duration_until_start_time,
            sale_creation_denom,
            sale_creation_fee,
            fee_collector,
        ),
    }
}

pub fn sudo_update_config(
    deps: DepsMut,
    _env: Env,
    min_sale_duration: Option<Uint64>,
    min_duration_until_start_time: Option<Uint64>,
    sale_creation_denom: Option<String>,
    sale_creation_fee: Option<Uint128>,
    fee_collector: Option<String>,
) -> Result<Response, ContractError> {
    let mut cfg = CONFIG.load(deps.storage)?;

    cfg.min_sale_duration = min_sale_duration.unwrap_or(cfg.min_sale_duration);
    cfg.min_duration_until_start_time =
        min_duration_until_start_time.unwrap_or(cfg.min_duration_until_start_time);
    cfg.sale_creation_denom = sale_creation_denom.unwrap_or(cfg.sale_creation_denom);
    cfg.sale_creation_fee = sale_creation_fee.unwrap_or(cfg.sale_creation_fee);

    let collector = fee_collector
        .map(|r| deps.api.addr_validate(&r))
        .transpose()?
        .unwrap_or(cfg.fee_collector);
    cfg.fee_collector = collector;

    CONFIG.save(deps.storage, &cfg)?;
    let attributes = vec![
        attr("action", "update_config"),
        attr("min_sale_duration", cfg.min_sale_duration),
        attr(
            "min_duration_until_start_time",
            cfg.min_duration_until_start_time,
        ),
        attr("sale_creation_denom", cfg.sale_creation_denom),
        attr("sale_creation_fee", cfg.sale_creation_fee),
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
        QueryMsg::Sale { sale_id } => to_binary(&query_sale(deps, env, sale_id)?),
        QueryMsg::Position { sale_id, owner } => {
            to_binary(&query_position(deps, env, sale_id, owner)?)
        }
        QueryMsg::ListSales { start_after, limit } => {
            to_binary(&list_sales(deps, start_after, limit)?)
        }
        QueryMsg::ListPositions {
            sale_id,
            start_after,
            limit,
        } => to_binary(&list_positions(deps, sale_id, start_after, limit)?),
        QueryMsg::AveragePrice { sale_id } => to_binary(&query_average_price(deps, env, sale_id)?),
        QueryMsg::LastStreamedPrice { sale_id } => {
            to_binary(&query_last_streamed_price(deps, env, sale_id)?)
        }
    }
}

pub fn query_sale(deps: Deps, _env: Env, sale_id: u64) -> StdResult<SaleResponse> {
    let sale = SALES.load(deps.storage, sale_id)?;
    let sale = SaleResponse {
        id: sale_id,
        treasury: sale.treasury.to_string(),
        token_in_denom: sale.in_denom,
        token_out_denom: sale.out_denom,
        token_out_supply: sale.out_supply,
        start_time: Timestamp::from_nanos(sale.start_time.u64()),
        end_time: Timestamp::from_nanos(sale.end_time.u64()),
        total_in_spent: sale.current_in,
        latest_stage: sale.current_stage,
        dist_index: sale.dist_index,
        total_out_sold: sale.current_out,
        total_in_supply: sale.in_supply,
    };
    Ok(sale)
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

pub fn list_sales(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<SalesResponse> {
    let start = start_after.map(Bound::exclusive);
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let sales: StdResult<Vec<SaleResponse>> = SALES
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (sale_id, sale) = item?;
            let sale = SaleResponse {
                id: sale_id,
                treasury: sale.treasury.to_string(),
                token_in_denom: sale.in_denom,
                token_out_denom: sale.out_denom,
                token_out_supply: sale.out_supply,
                start_time: Timestamp::from_nanos(sale.start_time.u64()),
                end_time: Timestamp::from_nanos(sale.end_time.u64()),
                total_in_spent: sale.current_in,
                latest_stage: sale.current_stage,
                dist_index: sale.dist_index,
                total_out_sold: sale.current_out,
                total_in_supply: sale.in_supply,
            };
            Ok(sale)
        })
        .collect();
    let sales = sales?;
    Ok(SalesResponse { sales })
}

pub fn query_position(
    deps: Deps,
    _env: Env,
    sale_id: u64,
    owner: String,
) -> StdResult<PositionResponse> {
    let owner = deps.api.addr_validate(&owner)?;
    let position = POSITIONS.load(deps.storage, (sale_id, &owner))?;
    let res = PositionResponse {
        sale_id,
        owner: owner.to_string(),
        buy_balance: position.in_balance,
        purchased: position.purchased,
        latest_dist_stage: position.latest_dist_stage,
        exited: position.exited,
        index: position.index,
        spent: position.spent,
    };
    Ok(res)
}

pub fn list_positions(
    deps: Deps,
    sale_id: u64,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<PositionsResponse> {
    let addr = maybe_addr(deps.api, start_after)?;
    let start = addr.as_ref().map(Bound::exclusive);
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let positions: StdResult<Vec<PositionResponse>> = POSITIONS
        .prefix(sale_id)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (owner, sale) = item?;
            let position = PositionResponse {
                sale_id,
                owner: owner.to_string(),
                index: sale.index,
                latest_dist_stage: sale.latest_dist_stage,
                purchased: sale.purchased,
                spent: sale.spent,
                buy_balance: sale.in_balance,
                exited: sale.exited,
            };
            Ok(position)
        })
        .collect();
    let positions = positions?;
    Ok(PositionsResponse { positions })
}

pub fn query_average_price(deps: Deps, _env: Env, sale_id: u64) -> StdResult<AveragePriceResponse> {
    let sale = SALES.load(deps.storage, sale_id)?;
    let average_price = sale.current_out / sale.current_in;
    Ok(AveragePriceResponse { average_price })
}

pub fn query_last_streamed_price(
    deps: Deps,
    _env: Env,
    sale_id: u64,
) -> StdResult<LatestStreamedPriceResponse> {
    let sale = SALES.load(deps.storage, sale_id)?;
    Ok(LatestStreamedPriceResponse {
        lastest_streamed_price: sale.latest_streamed_price,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn basic() {}
}
