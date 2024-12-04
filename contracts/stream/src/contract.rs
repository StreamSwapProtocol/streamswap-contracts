use crate::helpers::{
    build_u128_bank_send_msg, check_name_and_url, get_decimals, validate_stream_times,
};
use crate::stream::{compute_shares_amount, sync_stream, sync_stream_status};
use crate::{circuit_ops, ContractError};
use core::str;
use cosmwasm_std::{
    attr, entry_point, to_json_binary, Addr, Attribute, BankMsg, Binary, Coin, CosmosMsg,
    Decimal256, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult, Timestamp, Uint128,
    Uint256,
};
use cw2::{ensure_from_older_version, set_contract_version};
use cw_storage_plus::Bound;
use cw_utils::{maybe_addr, must_pay, NativeBalance};
use std::env;
use streamswap_types::stream::{
    AveragePriceResponse, ExecuteMsg, FinalizedStatus, LatestStreamedPriceResponse,
    PositionResponse, PositionsResponse, QueryMsg, StreamResponse,
};
use streamswap_types::stream::{PostStreamActions, StreamInfo, StreamState};
use streamswap_utils::to_uint256;

use crate::pool::{pool_operations, pool_refund};
use crate::state::{
    CONTROLLER_PARAMS, CREATOR_VESTING, POSITIONS, POST_STREAM, STREAM_INFO, STREAM_STATE,
    SUBSCRIBER_VESTING, TOS, TOS_SIGNED,
};
use crate::vesting::vesting_operations;
use streamswap_types::controller::{CreatePool, Params as ControllerParams, PoolConfig};
use streamswap_types::controller::{CreateStreamMsg, MigrateMsg};
use streamswap_types::stream::{Position, Status};

// Version and contract info for migration
const CONTRACT_NAME: &str = "crates.io:streamswap-stream";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CreateStreamMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let params_query_msg = QueryMsg::Params {};
    let controller_params: ControllerParams = deps
        .querier
        .query_wasm_smart(info.sender.to_string(), &params_query_msg)?;
    // Controller parameters are collected at the time of stream creation
    // Any changes to controller parameters will not affect the stream
    CONTROLLER_PARAMS.save(deps.storage, &controller_params)?;

    let CreateStreamMsg {
        bootstraping_start_time,
        start_time,
        end_time,
        treasury,
        name,
        url,
        threshold,
        out_asset,
        in_denom,
        stream_admin,
        pool_config,
        subscriber_vesting,
        creator_vesting,
        salt: _,
        tos_version,
    } = msg;

    validate_stream_times(
        env.block.time,
        bootstraping_start_time,
        start_time,
        end_time,
        &controller_params,
    )?;

    if in_denom == out_asset.denom {
        return Err(ContractError::SameDenomOnEachSide {});
    }
    let stream_admin = deps.api.addr_validate(&stream_admin)?;
    let treasury = deps.api.addr_validate(&treasury)?;

    check_name_and_url(&name, &url)?;

    let stream_state = StreamState::new(
        env.block.time,
        out_asset.clone(),
        in_denom.clone(),
        bootstraping_start_time,
        start_time,
        end_time,
        threshold,
    );
    STREAM_STATE.save(deps.storage, &stream_state)?;

    let stream_info = StreamInfo::new(stream_admin.clone(), name.clone(), treasury.clone(), url);
    STREAM_INFO.save(deps.storage, &stream_info)?;

    let post_stream_actions =
        PostStreamActions::new(pool_config.clone(), subscriber_vesting, creator_vesting);
    POST_STREAM.save(deps.storage, &post_stream_actions)?;

    TOS.save(deps.storage, &tos_version)?;

    let mut attrs = vec![
        attr("action", "instantiate"),
        attr("name", name),
        attr("treasury", treasury),
        attr("stream_admin", stream_admin),
        attr("out_asset", out_asset.denom),
        attr("in_denom", in_denom),
        attr("start_time", start_time.to_string()),
        attr("end_time", end_time.to_string()),
        attr(
            "bootstrapping_start_time",
            bootstraping_start_time.to_string(),
        ),
    ];
    // if pool config is set, add attributes
    if let Some(pool_config) = pool_config {
        match pool_config {
            PoolConfig::ConcentratedLiquidity { out_amount_clp } => {
                let attributes = vec![
                    attr("pool_type", "clp".to_string()),
                    attr("pool_out_amount", out_amount_clp),
                ];
                attrs.extend(attributes);
            }
        }
    }

    // return response with attributes
    let res = Response::new().add_attributes(attrs);
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SyncPosition {} => execute_sync_position(deps, env, info),
        ExecuteMsg::SyncStream {} => execute_sync_stream(deps, env),
        ExecuteMsg::Subscribe {} => {
            let stream = STREAM_STATE.load(deps.storage)?;
            execute_subscribe(deps, env, info, stream)
        }
        ExecuteMsg::Withdraw { cap } => {
            let stream = STREAM_STATE.load(deps.storage)?;
            execute_withdraw(deps, env, info, stream, cap)
        }
        ExecuteMsg::FinalizeStream {
            new_treasury,
            create_pool,
            salt,
        } => execute_finalize_stream(deps, env, info, new_treasury, create_pool, salt),
        ExecuteMsg::ExitStream { salt } => execute_exit_stream(deps, env, info, salt),
        ExecuteMsg::CancelStream {} => circuit_ops::execute_cancel_stream(deps, env, info),
        ExecuteMsg::StreamAdminCancel {} => {
            circuit_ops::execute_stream_admin_cancel(deps, env, info)
        }
    }
}

/// Syncs stream to calculate released distribution and spent amount
pub fn execute_sync_stream(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let mut stream = STREAM_STATE.load(deps.storage)?;
    sync_stream_status(&mut stream, env.block.time);
    if stream.is_cancelled() {
        return Err(ContractError::OperationNotAllowed {
            current_status: stream.status_info.status.to_string(),
        });
    }
    sync_stream(&mut stream, env.block.time);
    STREAM_STATE.save(deps.storage, &stream)?;

    let attrs = vec![
        attr("action", "sync_stream"),
        attr("out_remaining", stream.out_remaining),
        attr("spent_in", stream.spent_in),
        attr(
            "current_streamed_price",
            stream.current_streamed_price.to_string(),
        ),
        attr("in_supply", stream.in_supply),
        attr("shares", stream.shares),
        attr("status_info", stream.status_info.status.to_string()),
        attr("dist_index", stream.dist_index.to_string()),
    ];
    let res = Response::new().add_attributes(attrs);
    Ok(res)
}

pub fn execute_sync_position(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut position = POSITIONS.load(deps.storage, &info.sender)?;

    let mut stream = STREAM_STATE.load(deps.storage)?;
    sync_stream_status(&mut stream, env.block.time);
    // check and return error if stream is cancelled
    if stream.is_cancelled() {
        return Err(ContractError::OperationNotAllowed {
            current_status: stream.status_info.status.to_string(),
        });
    }

    // sync stream
    sync_stream(&mut stream, env.block.time);
    STREAM_STATE.save(deps.storage, &stream)?;

    // updates position to latest distribution. Returns the amount of out tokens that has been purchased
    // and in tokens that has been spent.
    let (purchased, spent) = sync_position(
        stream.dist_index,
        stream.shares,
        stream.status_info.last_updated,
        stream.in_supply,
        &mut position,
    )?;
    POSITIONS.save(deps.storage, &position.owner, &position)?;

    // return response with attributes
    let res = Response::new().add_attributes(vec![
        attr("action", "sync_position"),
        attr("dist_index", stream.dist_index.to_string()),
        attr("status", stream.status_info.status.to_string()),
        attr("purchased", purchased),
        attr("spent", spent),
    ]);
    Ok(res)
}

// calculate the user purchase based on the positions index and the global index.
// returns purchased out amount and spent in amount
pub fn sync_position(
    stream_dist_index: Decimal256,
    stream_shares: Uint256,
    stream_last_updated_time: Timestamp,
    stream_in_supply: Uint256,
    position: &mut Position,
) -> Result<(Uint256, Uint256), ContractError> {
    // index difference represents the amount of distribution that has been received since last update
    let index_diff = stream_dist_index.checked_sub(position.index)?;

    let mut spent = Uint256::zero();
    let mut uint256_purchased = Uint256::zero();

    // if no shares available, means no distribution and no spent
    if !stream_shares.is_zero() {
        // purchased is index_diff * position.shares
        let purchased = Decimal256::from_ratio(position.shares, Uint256::one())
            .checked_mul(index_diff)?
            .checked_add(position.pending_purchase)?;
        // decimals is the amount of decimals that the out token has to be added to next distribution so that
        // the data do not get lost due to rounding
        let decimals = get_decimals(purchased)?;

        // calculates the remaining user balance using position.shares
        let in_remaining = stream_in_supply
            .checked_mul(position.shares)?
            .checked_div(stream_shares)?;

        // calculates the amount of spent tokens
        spent = position.in_balance.checked_sub(in_remaining)?;
        position.spent = position.spent.checked_add(spent)?;
        position.in_balance = in_remaining;
        position.pending_purchase = decimals;

        // floors the decimal points
        uint256_purchased = purchased * Uint256::one();
        position.purchased = position.purchased.checked_add(uint256_purchased)?;
    }

    position.index = stream_dist_index;
    position.last_updated = stream_last_updated_time;

    Ok((uint256_purchased, spent))
}

pub fn execute_subscribe(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut stream_state: StreamState,
) -> Result<Response, ContractError> {
    // Update stream status
    sync_stream_status(&mut stream_state, env.block.time);

    if !(stream_state.is_active() || stream_state.is_bootstrapping()) {
        return Err(ContractError::OperationNotAllowed {
            current_status: stream_state.status_info.status.clone().to_string(),
        });
    }

    let in_amount = must_pay(&info, &stream_state.in_denom)?;
    let uint256_in_amount = Uint256::from(in_amount.u128());
    let new_shares;

    let position = POSITIONS.may_load(deps.storage, &info.sender)?;
    match position {
        None => {
            // incoming tokens should not participate in prev distribution
            sync_stream(&mut stream_state, env.block.time);
            new_shares = compute_shares_amount(&stream_state, uint256_in_amount, false);
            // new positions do not update purchase as it has no effect on distribution
            let new_position = Position::new(
                info.sender.clone(),
                uint256_in_amount,
                new_shares,
                Some(stream_state.dist_index),
                env.block.time,
            );
            POSITIONS.save(deps.storage, &info.sender, &new_position)?;

            // Save signed TOS
            let tos_version = TOS.load(deps.storage)?;
            TOS_SIGNED.save(deps.storage, &info.sender, &tos_version)?;
        }
        Some(mut position) => {
            if position.owner != info.sender {
                return Err(ContractError::Unauthorized {});
            }
            // incoming tokens should not participate in prev distribution
            sync_stream(&mut stream_state, env.block.time);
            new_shares = compute_shares_amount(&stream_state, uint256_in_amount, false);
            sync_position(
                stream_state.dist_index,
                stream_state.shares,
                stream_state.status_info.last_updated,
                stream_state.in_supply,
                &mut position,
            )?;

            position.in_balance = position.in_balance.checked_add(uint256_in_amount)?;
            position.shares = position.shares.checked_add(new_shares)?;
            POSITIONS.save(deps.storage, &info.sender, &position)?;
        }
    }

    // increase in supply and shares
    stream_state.in_supply = stream_state.in_supply.checked_add(uint256_in_amount)?;
    stream_state.shares = stream_state.shares.checked_add(new_shares)?;
    STREAM_STATE.save(deps.storage, &stream_state)?;

    let res = Response::new()
        .add_attribute("action", "subscribe")
        .add_attribute("status info", stream_state.status_info.status.to_string())
        .add_attribute("in_supply", stream_state.in_supply)
        .add_attribute("in_amount", in_amount)
        .add_attribute("subscriber_shares", new_shares)
        .add_attribute("total_shares", stream_state.shares)
        .add_attribute("dist_index", stream_state.dist_index.to_string());

    Ok(res)
}

pub fn execute_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut stream: StreamState,
    cap: Option<Uint256>,
) -> Result<Response, ContractError> {
    sync_stream_status(&mut stream, env.block.time);
    if !(stream.is_active() || stream.is_bootstrapping()) {
        return Err(ContractError::OperationNotAllowed {
            current_status: stream.status_info.status.to_string(),
        });
    }

    let mut position = POSITIONS.load(deps.storage, &info.sender)?;

    sync_stream(&mut stream, env.block.time);
    sync_position(
        stream.dist_index,
        stream.shares,
        stream.status_info.last_updated,
        stream.in_supply,
        &mut position,
    )?;

    let withdraw_amount = cap.unwrap_or(position.in_balance);
    // if amount to withdraw more then deduced buy balance throw error
    if withdraw_amount > position.in_balance {
        return Err(ContractError::WithdrawAmountExceedsBalance(withdraw_amount));
    }

    if withdraw_amount.is_zero() {
        return Err(ContractError::InvalidWithdrawAmount {});
    }

    // decrease in supply and shares
    let shares_amount = if withdraw_amount == position.in_balance {
        position.shares
    } else {
        compute_shares_amount(&stream, withdraw_amount, true)
    };

    stream.in_supply = stream.in_supply.checked_sub(withdraw_amount)?;
    stream.shares = stream.shares.checked_sub(shares_amount)?;
    position.in_balance = position.in_balance.checked_sub(withdraw_amount)?;
    position.shares = position.shares.checked_sub(shares_amount)?;

    STREAM_STATE.save(deps.storage, &stream)?;
    POSITIONS.save(deps.storage, &position.owner, &position)?;

    let uint128_withdraw_amount = Uint128::try_from(withdraw_amount)?;
    let fund_transfer_msg: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom: stream.in_denom,
            amount: uint128_withdraw_amount,
        }],
    });
    // send funds to withdraw address or to the sender
    let res = Response::new()
        .add_message(fund_transfer_msg)
        .add_attribute("action", "withdraw")
        .add_attribute("withdraw_amount", withdraw_amount)
        .add_attribute("shares_amount", shares_amount)
        .add_attribute("status_info", stream.status_info.status.to_string());

    Ok(res)
}
pub fn execute_finalize_stream(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_treasury: Option<String>,
    create_pool: Option<CreatePool>,
    salt: Option<Binary>,
) -> Result<Response, ContractError> {
    let stream_info = STREAM_INFO.load(deps.storage)?;
    let controller_params = CONTROLLER_PARAMS.load(deps.storage)?;
    if stream_info.stream_admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let mut stream_state = STREAM_STATE.load(deps.storage)?;
    sync_stream(&mut stream_state, env.block.time);
    sync_stream_status(&mut stream_state, env.block.time);
    let treasury =
        maybe_addr(deps.api, new_treasury)?.unwrap_or_else(|| stream_info.treasury.clone());

    match (
        stream_state.status_info.clone().status,
        stream_state.check_threshold(),
    ) {
        (Status::Ended, true) => {
            let mut messages = vec![];
            let mut attributes = vec![];

            // last creator revenue = spent_in - swap_fee - in_clp;
            let mut creator_revenue = stream_state.spent_in;

            // Stream's swap fee collected at fixed rate from accumulated spent_in of positions(ie stream.spent_in)
            let swap_fee = Decimal256::from_ratio(stream_state.spent_in, Uint128::one())
                .checked_mul(controller_params.exit_fee_percent)?
                * Uint256::one();

            // extract swap_fee from last amount
            creator_revenue = creator_revenue.checked_sub(swap_fee)?;
            let creator_revenue_u128 = Uint128::try_from(creator_revenue)?;

            // In case the stream is ended without any shares in it. We need to refund the remaining
            // out tokens although that is unlikely to happen.
            if stream_state.out_remaining > Uint256::zero() {
                let remaining_out = stream_state.out_remaining;
                let uint128_remaining_out = Uint128::try_from(remaining_out)?;
                // Sub remaining out tokens from out_asset
                stream_state.out_asset.amount = stream_state
                    .out_asset
                    .amount
                    .checked_sub(uint128_remaining_out)?;
                let remaining_msg = CosmosMsg::Bank(BankMsg::Send {
                    to_address: treasury.to_string(),
                    amount: vec![Coin {
                        denom: stream_state.out_asset.denom.clone(),
                        amount: uint128_remaining_out,
                    }],
                });
                messages.push(remaining_msg);
            }

            // if vesting is not set we need to send the creator revenue to the treasury in outer scope
            let mut vesting_flag = false;
            // execute post stream actions
            let post_stream_actions = POST_STREAM.may_load(deps.storage)?;
            if let Some(post_stream_actions) = post_stream_actions {
                // if pool config and create pool is set, create a pool for the stream
                creator_revenue = match (post_stream_actions.pool_config, create_pool) {
                    (Some(pool_config), Some(create_pool)) => {
                        let (msgs, attrs, creator_revenue) = pool_operations(
                            &deps,
                            create_pool,
                            env.contract.address.clone(),
                            stream_state.in_denom.clone(),
                            stream_state.out_asset.denom.clone(),
                            stream_state.out_asset.amount,
                            creator_revenue,
                            pool_config,
                        )?;
                        messages.extend(msgs);
                        attributes.extend(attrs);
                        Ok(creator_revenue)
                    }
                    (None, None) => Ok(creator_revenue),
                    _ => Err(ContractError::InvalidPoolConfig {}),
                }?;

                // if subscriber vesting is set, instantiate a vested release contract for user and send
                if let Some(creator_vesting) = post_stream_actions.creator_vesting {
                    let vesting_checksum = deps
                        .querier
                        .query_wasm_code_info(controller_params.vesting_code_id)?
                        .checksum;
                    let (vesting_msgs, vesting_attributes, vesting_addr) = vesting_operations(
                        &deps,
                        env.contract.address,
                        vesting_checksum,
                        treasury.clone(),
                        salt,
                        stream_state.status_info.end_time,
                        controller_params.vesting_code_id,
                        creator_revenue_u128,
                        stream_state.in_denom.clone(),
                        creator_vesting,
                    )?;
                    messages.extend(vesting_msgs);
                    attributes.extend(vesting_attributes);
                    CREATOR_VESTING.save(deps.storage, &vesting_addr)?;
                    vesting_flag = true;
                }
            }

            if !vesting_flag {
                let send_msg = CosmosMsg::Bank(BankMsg::Send {
                    to_address: treasury.to_string(),
                    amount: vec![Coin {
                        denom: stream_state.in_denom.clone(),
                        amount: creator_revenue_u128,
                    }],
                });
                messages.push(send_msg);
            }

            let swap_fee_msg = build_u128_bank_send_msg(
                stream_state.in_denom.clone(),
                controller_params.fee_collector.to_string(),
                swap_fee,
            )?;
            messages.push(swap_fee_msg);

            stream_state.status_info.status = Status::Finalized(FinalizedStatus::ThresholdReached);
            STREAM_STATE.save(deps.storage, &stream_state)?;

            attributes.extend(vec![
                attr("action", "finalize_stream"),
                attr("treasury", treasury.to_string()),
                attr("fee_collector", controller_params.fee_collector.to_string()),
                attr("creators_revenue", creator_revenue),
                attr(
                    "refunded_out_remaining",
                    stream_state.out_remaining.to_string(),
                ),
                attr(
                    "total_sold",
                    to_uint256(stream_state.out_asset.amount)
                        .checked_sub(stream_state.out_remaining)?
                        .to_string(),
                ),
                attr("swap_fee", swap_fee),
                attr(
                    "creation_fee_amount",
                    controller_params.stream_creation_fee.amount.to_string(),
                ),
            ]);

            Ok(Response::new()
                .add_messages(messages)
                .add_attributes(attributes))
        }
        (Status::Ended, false) => {
            // if stream is ended and threshold is not reached, return all in tokens to treasury
            // Refund all out tokens to stream creator(treasury)
            let mut refund_coins = NativeBalance::default() + stream_state.out_asset.clone();

            // refund pool creation if any
            let post_stream_ops = POST_STREAM.may_load(deps.storage)?;
            if let Some(post_stream_ops) = post_stream_ops {
                let pool_refund_coins = pool_refund(
                    &deps,
                    post_stream_ops.pool_config,
                    stream_state.out_asset.denom.clone(),
                )?;
                for coin in pool_refund_coins {
                    refund_coins += coin;
                }
            }
            refund_coins.normalize();

            let funds_msgs: Vec<CosmosMsg> = refund_coins
                .into_vec()
                .iter()
                .map(|coin| {
                    CosmosMsg::Bank(BankMsg::Send {
                        to_address: stream_info.treasury.to_string(),
                        amount: vec![coin.clone()],
                    })
                })
                .collect();

            stream_state.status_info.status =
                Status::Finalized(FinalizedStatus::ThresholdNotReached);
            STREAM_STATE.save(deps.storage, &stream_state)?;

            Ok(Response::new()
                .add_attribute("action", "finalize_stream")
                .add_attribute("status", "threshold_not_reached")
                .add_attribute("treasury", stream_info.treasury.to_string())
                .add_messages(funds_msgs))
        }
        _ => {
            return Err(ContractError::OperationNotAllowed {
                current_status: stream_state.status_info.status.to_string(),
            });
        }
    }
}
pub fn execute_exit_stream(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    salt: Option<Binary>,
) -> Result<Response, ContractError> {
    let mut stream_state = STREAM_STATE.load(deps.storage)?;
    let controller_params = CONTROLLER_PARAMS.load(deps.storage)?;

    let mut position = POSITIONS.load(deps.storage, &info.sender)?;
    if position.exit_date != Timestamp::from_seconds(0) {
        return Err(ContractError::SubscriberAlreadyExited {});
    }
    sync_stream_status(&mut stream_state, env.block.time);
    sync_stream(&mut stream_state, env.block.time);

    match (
        stream_state.status_info.clone().status,
        stream_state.check_threshold(),
    ) {
        (Status::Ended, true) | (Status::Finalized(FinalizedStatus::ThresholdReached), _) => {
            let mut position = POSITIONS.load(deps.storage, &info.sender)?;
            sync_position(
                stream_state.dist_index,
                stream_state.shares,
                stream_state.status_info.last_updated,
                stream_state.in_supply,
                &mut position,
            )?;

            stream_state.shares = stream_state.shares.checked_sub(position.shares)?;

            STREAM_STATE.save(deps.storage, &stream_state)?;
            position.exit_date = env.block.time;
            POSITIONS.save(deps.storage, &position.owner, &position)?;

            let swap_fee = Decimal256::from_ratio(position.spent, Uint128::one())
                .checked_mul(controller_params.exit_fee_percent)?
                * Uint256::one();

            let mut messages: Vec<CosmosMsg> = vec![];
            let mut attributes: Vec<Attribute> = vec![];

            let uint128_purchased = Uint128::try_from(position.purchased)?;

            let mut vesting_flag = false;

            let post_stream_actions = POST_STREAM.may_load(deps.storage)?;

            if let Some(post_stream_actions) = post_stream_actions {
                if let Some(vesting_config) = post_stream_actions.subscriber_vesting {
                    let vesting_checksum = deps
                        .querier
                        .query_wasm_code_info(controller_params.vesting_code_id)?
                        .checksum;
                    let (vesting_msgs, vesting_attributes, vesting_addr) = vesting_operations(
                        &deps,
                        env.contract.address,
                        vesting_checksum,
                        info.sender.clone(),
                        salt,
                        stream_state.status_info.end_time,
                        controller_params.vesting_code_id,
                        uint128_purchased,
                        stream_state.out_asset.denom.clone(),
                        vesting_config,
                    )?;
                    messages.extend(vesting_msgs);
                    attributes.extend(vesting_attributes);
                    SUBSCRIBER_VESTING.save(deps.storage, info.sender.clone(), &vesting_addr)?;
                    vesting_flag = true;
                }
            }

            if !vesting_flag {
                let send_msg = CosmosMsg::Bank(BankMsg::Send {
                    to_address: info.sender.to_string(),
                    amount: vec![Coin {
                        denom: stream_state.out_asset.denom.to_string(),
                        amount: uint128_purchased,
                    }],
                });
                messages.push(send_msg);
            }

            if !position.in_balance.is_zero() {
                let unspent = position.in_balance;
                let uint128_unspent = Uint128::try_from(unspent)?;
                let unspent_msg = CosmosMsg::Bank(BankMsg::Send {
                    to_address: info.sender.to_string(),
                    amount: vec![Coin {
                        denom: stream_state.in_denom,
                        amount: uint128_unspent,
                    }],
                });
                messages.push(unspent_msg);
            }

            attributes.extend(vec![
                attr("action", "exit_stream"),
                attr("spent", position.spent.checked_sub(swap_fee)?),
                attr("purchased", position.purchased),
                attr("swap_fee_paid", swap_fee),
            ]);

            Ok(Response::new()
                .add_messages(messages)
                .add_attributes(attributes))
        }

        (Status::Ended, false)
        | (Status::Finalized(FinalizedStatus::ThresholdNotReached), _)
        | (Status::Cancelled, _) => {
            // no need to sync position here, we just need to return total balance
            let total_balance = position.in_balance + position.spent;
            // sync position exit date
            position.exit_date = env.block.time;
            position.last_updated = env.block.time;
            POSITIONS.save(deps.storage, &position.owner, &position)?;

            let send_msg = build_u128_bank_send_msg(
                stream_state.in_denom.clone(),
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
        _ => {
            return Err(ContractError::OperationNotAllowed {
                current_status: stream_state.status_info.status.to_string(),
            });
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    ensure_from_older_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Params {} => to_json_binary(&query_params(deps)?),
        QueryMsg::Stream {} => to_json_binary(&query_stream(deps, env)?),
        QueryMsg::Position { owner } => to_json_binary(&query_position(deps, env, owner)?),
        QueryMsg::ListPositions { start_after, limit } => {
            to_json_binary(&list_positions(deps, start_after, limit)?)
        }
        QueryMsg::AveragePrice {} => to_json_binary(&query_average_price(deps, env)?),
        QueryMsg::LastStreamedPrice {} => to_json_binary(&query_last_streamed_price(deps, env)?),
        QueryMsg::ToS { addr } => {
            if let Some(addr) = addr {
                to_json_binary(&query_tos_signed(
                    deps,
                    &deps.api.addr_validate(addr.as_str())?,
                )?)
            } else {
                to_json_binary(&query_tos(deps)?)
            }
        }
    }
}
pub fn query_params(deps: Deps) -> StdResult<ControllerParams> {
    let controller_params = CONTROLLER_PARAMS.load(deps.storage)?;
    Ok(controller_params)
}

pub fn query_stream(deps: Deps, _env: Env) -> StdResult<StreamResponse> {
    let stream = STREAM_STATE.load(deps.storage)?;
    let stream_info = STREAM_INFO.load(deps.storage)?;
    let stream = StreamResponse {
        treasury: stream_info.treasury.to_string(),
        in_denom: stream.in_denom,
        out_asset: stream.out_asset,
        start_time: stream.status_info.start_time,
        end_time: stream.status_info.end_time,
        last_updated: stream.status_info.last_updated,
        spent_in: stream.spent_in,
        dist_index: stream.dist_index,
        out_remaining: stream.out_remaining,
        in_supply: stream.in_supply,
        shares: stream.shares,
        status: stream.status_info.status,
        url: stream_info.url,
        current_streamed_price: stream.current_streamed_price,
        stream_admin: stream_info.stream_admin.into_string(),
    };
    Ok(stream)
}

pub fn query_position(deps: Deps, _env: Env, owner: String) -> StdResult<PositionResponse> {
    let owner = deps.api.addr_validate(&owner)?;
    let position = POSITIONS.load(deps.storage, &owner)?;
    let res = PositionResponse {
        owner: owner.to_string(),
        in_balance: position.in_balance,
        purchased: position.purchased,
        index: position.index,
        spent: position.spent,
        shares: position.shares,
        last_updated: position.last_updated,
        pending_purchase: position.pending_purchase,
        exit_date: position.exit_date,
    };
    Ok(res)
}

pub fn list_positions(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<PositionsResponse> {
    const MAX_LIMIT: u32 = 30;
    let start_addr = maybe_addr(deps.api, start_after)?;
    let start = start_addr.as_ref().map(Bound::exclusive);
    let limit = limit.unwrap_or(MAX_LIMIT).min(MAX_LIMIT) as usize;
    let positions: StdResult<Vec<PositionResponse>> = POSITIONS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (owner, position) = item?;
            let position = PositionResponse {
                owner: owner.to_string(),
                in_balance: position.in_balance,
                purchased: position.purchased,
                index: position.index,
                spent: position.spent,
                shares: position.shares,
                last_updated: position.last_updated,
                pending_purchase: position.pending_purchase,
                exit_date: position.exit_date,
            };
            Ok(position)
        })
        .collect();
    let positions = positions?;
    Ok(PositionsResponse { positions })
}

pub fn query_average_price(deps: Deps, _env: Env) -> StdResult<AveragePriceResponse> {
    let stream = STREAM_STATE.load(deps.storage)?;
    let total_purchased = to_uint256(stream.out_asset.amount) - stream.out_remaining;
    let average_price = Decimal256::from_ratio(stream.spent_in, total_purchased);
    Ok(AveragePriceResponse { average_price })
}

pub fn query_last_streamed_price(deps: Deps, _env: Env) -> StdResult<LatestStreamedPriceResponse> {
    let stream = STREAM_STATE.load(deps.storage)?;
    Ok(LatestStreamedPriceResponse {
        current_streamed_price: stream.current_streamed_price,
    })
}

pub fn query_tos(deps: Deps) -> StdResult<String> {
    let tos = TOS.load(deps.storage)?;
    Ok(tos)
}

pub fn query_tos_signed(deps: Deps, addr: &Addr) -> StdResult<String> {
    let tos = TOS_SIGNED.load(deps.storage, addr)?;
    Ok(tos)
}
