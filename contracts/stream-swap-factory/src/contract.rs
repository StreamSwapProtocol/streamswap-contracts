use cosmwasm_std::{
    entry_point, to_binary, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, WasmMsg,
};
use cw_controllers::Admin;
use cw_utils::maybe_addr;

use crate::{
    error::ContractError,
    msg::{self, CreateStreamMsg, ExecuteMsg, InstantiateMsg},
    payment_checker::{self, check_payment},
    state::{Params, PARAMS},
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let InstantiateMsg {
        stream_swap_code_id,
        admin,
        stream_creation_fee,
        exit_fee_percent,
        accepted_in_denoms,
        fee_collector,
        min_stream_blocks,
        min_blocks_until_start_block,
    } = msg;

    let admin = deps
        .api
        .addr_validate(&admin.unwrap_or(info.sender.to_string()))?;
    let fee_collector = deps
        .api
        .addr_validate(&fee_collector.unwrap_or(info.sender.to_string()))?;

    if exit_fee_percent > Decimal::percent(100) || exit_fee_percent < Decimal::percent(0) {
        return Err(ContractError::InvalidExitFeePercent {});
    }

    let params = Params {
        admin: admin.clone(),
        stream_creation_fee: stream_creation_fee.clone(),
        exit_fee_percent,
        stream_swap_code_id,
        accepted_in_denoms,
        fee_collector,
        min_stream_blocks,
        min_blocks_until_start_block,
    };
    PARAMS.save(deps.storage, &params)?;

    let res = Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", admin.to_string())
        .add_attribute(
            "stream_creation_fee",
            stream_creation_fee.amount.to_string(),
        )
        .add_attribute("exit_fee_percent", exit_fee_percent.to_string())
        .add_attribute("stream_swap_code_id", stream_swap_code_id.to_string());
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
        ExecuteMsg::UpdateConfig {
            min_stream_blocks,
            min_blocks_until_start_block,
            stream_creation_fee,
            fee_collector,
            accepted_in_denoms,
            exit_fee_percent,
        } => execute_update_config(
            deps,
            env,
            info,
            min_stream_blocks,
            min_blocks_until_start_block,
            stream_creation_fee,
            fee_collector,
            accepted_in_denoms,
            exit_fee_percent,
        ),
        ExecuteMsg::CreateStream(msg) => execute_create_stream(deps, env, info, msg),
    }
}

pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    min_stream_blocks: Option<u64>,
    min_blocks_until_start_block: Option<u64>,
    stream_creation_fee: Option<Coin>,
    fee_collector: Option<String>,
    accepted_in_denoms: Option<Vec<String>>,
    exit_fee_percent: Option<Decimal>,
) -> Result<Response, ContractError> {
    let mut params = PARAMS.load(deps.storage)?;

    if info.sender != params.admin {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(stream_creation_fee) = stream_creation_fee {
        params.stream_creation_fee = stream_creation_fee;
    }

    if let Some(exit_fee_percent) = exit_fee_percent {
        if exit_fee_percent > Decimal::percent(100) || exit_fee_percent < Decimal::percent(0) {
            return Err(ContractError::InvalidExitFeePercent {});
        }
        params.exit_fee_percent = exit_fee_percent;
    }

    if let Some(fee_collector) = fee_collector {
        params.fee_collector = deps.api.addr_validate(&fee_collector)?;
    }

    if let Some(accepted_in_denoms) = accepted_in_denoms {
        params.accepted_in_denoms = accepted_in_denoms;
    }

    if let Some(min_stream_blocks) = min_stream_blocks {
        params.min_stream_blocks = min_stream_blocks;
    }

    if let Some(min_blocks_until_start_block) = min_blocks_until_start_block {
        params.min_blocks_until_start_block = min_blocks_until_start_block;
    }

    PARAMS.save(deps.storage, &params)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

pub fn execute_create_stream(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CreateStreamMsg,
) -> Result<Response, ContractError> {
    let CreateStreamMsg {
        treasury,
        name,
        url,
        out_asset,
        start_block,
        end_block,
        threshold,
        in_denom,
    } = msg.clone();
    let params = PARAMS.load(deps.storage)?;
    let stream_creation_fee = params.stream_creation_fee.clone();
    let accepted_in_denoms = params.accepted_in_denoms.clone();
    let expected_funds = vec![stream_creation_fee.clone(), out_asset.clone()];
    check_payment(&info.funds, &expected_funds)?;

    if end_block - start_block < params.min_stream_blocks {
        return Err(ContractError::StreamDurationTooShort {});
    }

    if start_block - env.block.height < params.min_blocks_until_start_block {
        return Err(ContractError::StreamStartsTooSoon {});
    }
    if !accepted_in_denoms.contains(&in_denom) {
        return Err(ContractError::InDenomIsNotAccepted {});
    }

    if &in_denom == &out_asset.denom {
        return Err(ContractError::SameDenomOnEachSide {});
    }

    let stream_swap_inst_message: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Instantiate {
        code_id: params.stream_swap_code_id,
        admin: Some(params.admin.to_string()),
        label: "Stream swap instance".to_string(),
        msg: to_binary(&msg)?,
        funds: vec![],
    });
    // TODO: If stream cration fee is zero this will fail
    let fund_transfer_message: CosmosMsg = CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
        to_address: params.fee_collector.to_string(),
        amount: vec![stream_creation_fee],
    });

    let res = Response::new()
        .add_message(stream_swap_inst_message)
        .add_message(fund_transfer_message)
        .add_attribute("action", "create_stream")
        .add_attribute("name", name)
        .add_attribute("treasury", treasury)
        .add_attribute("url", url.unwrap_or_default())
        .add_attribute("out_asset", out_asset.to_string())
        .add_attribute("start_block", start_block.to_string())
        .add_attribute("end_block", end_block.to_string())
        .add_attribute("in_denom", in_denom)
        .add_attribute("threshold", threshold.unwrap_or_default().to_string());
    Ok(res)
}
