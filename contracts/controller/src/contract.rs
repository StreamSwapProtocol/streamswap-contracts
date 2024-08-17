use crate::error::ContractError;
use crate::helpers::get_pool_creation_fee;
use crate::state::{FREEZESTATE, LAST_STREAM_ID, PARAMS, STREAMS};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Coin, CosmosMsg, Decimal256, Deps, DepsMut, Env,
    MessageInfo, Order, Response, StdResult, Uint128, WasmMsg,
};
use cw2::ensure_from_older_version;
use cw_storage_plus::Bound;
use streamswap_types::controller::{
    CreateStreamMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, Params, PoolConfig, QueryMsg,
    StreamResponse, StreamsResponse,
};
use streamswap_utils::payment_checker::check_payment;

const CONTRACT_NAME: &str = "crates.io:streamswap-controller";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let InstantiateMsg {
        stream_contract_code_id,
        protocol_admin,
        stream_creation_fee,
        exit_fee_percent,
        accepted_in_denoms,
        fee_collector,
        vesting_code_id,
        min_waiting_duration,
        min_bootstrapping_duration,
        min_stream_duration,
    } = msg;

    let protocol_admin = deps
        .api
        .addr_validate(&protocol_admin.unwrap_or(info.sender.to_string()))?;
    let fee_collector = deps
        .api
        .addr_validate(&fee_collector.unwrap_or(info.sender.to_string()))?;

    if exit_fee_percent > Decimal256::percent(100) || exit_fee_percent < Decimal256::percent(0) {
        return Err(ContractError::InvalidExitFeePercent {});
    }
    if stream_creation_fee.amount.is_zero() {
        return Err(ContractError::InvalidStreamCreationFee {});
    }

    let params = Params {
        stream_creation_fee: stream_creation_fee.clone(),
        exit_fee_percent,
        stream_contract_code_id,
        vesting_code_id,
        accepted_in_denoms: accepted_in_denoms.clone(),
        fee_collector: fee_collector.clone(),
        protocol_admin: protocol_admin.clone(),
        min_waiting_duration,
        min_bootstrapping_duration,
        min_stream_duration,
    };
    PARAMS.save(deps.storage, &params)?;

    // Initialize Freezestate
    FREEZESTATE.save(deps.storage, &false)?;

    // Initialize Last Stream ID
    LAST_STREAM_ID.save(deps.storage, &0)?;

    let res = Response::new()
        .add_attributes(params.to_attributes())
        .add_attribute("action", "instantiate controller");
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
        ExecuteMsg::UpdateParams {
            min_waiting_duration,
            min_bootstrapping_duration,
            min_stream_duration,
            stream_creation_fee,
            fee_collector,
            accepted_in_denoms,
            exit_fee_percent,
        } => execute_update_params(
            deps,
            env,
            info,
            min_waiting_duration,
            min_bootstrapping_duration,
            min_stream_duration,
            stream_creation_fee,
            fee_collector,
            accepted_in_denoms,
            exit_fee_percent,
        ),
        ExecuteMsg::CreateStream { msg } => execute_create_stream(deps, env, info, *msg),
        ExecuteMsg::Freeze {} => execute_freeze(deps, info),
        ExecuteMsg::Unfreeze {} => execute_unfreeze(deps, info),
    }
}

pub fn execute_create_stream(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: CreateStreamMsg,
) -> Result<Response, ContractError> {
    let is_frozen = FREEZESTATE.load(deps.storage)?;

    if is_frozen {
        return Err(ContractError::ContractIsFrozen {});
    }

    let CreateStreamMsg {
        treasury,
        name,
        out_asset,
        start_time,
        end_time,
        in_denom,
        stream_admin: _,
        threshold: _,
        url: _,
        pool_config: create_pool,
        vesting: _,
        bootstraping_start_time: _,
        salt,
    } = msg.clone();

    let params = PARAMS.load(deps.storage)?;
    let stream_creation_fee = params.stream_creation_fee.clone();

    let accepted_in_denoms = params.accepted_in_denoms.clone();
    if !accepted_in_denoms.contains(&in_denom) {
        return Err(ContractError::InDenomIsNotAccepted {});
    }
    if out_asset.amount.is_zero() {
        return Err(ContractError::ZeroOutSupply {});
    }
    // These funds shall be sent to controller, after the control these funds will be distributed to the stream contract and fee_collector
    let mut expected_funds = vec![stream_creation_fee.clone(), out_asset.clone()];
    // These funds shall be sent to the stream contract
    let mut instantiate_funds: Vec<Coin> = vec![out_asset.clone()];

    if let Some(create_pool) = create_pool {
        match create_pool {
            PoolConfig::ConcentratedLiquidity { out_amount_clp } => {
                let uint128_pool_out_amount = Uint128::try_from(out_amount_clp)?;
                if uint128_pool_out_amount > out_asset.amount {
                    return Err(ContractError::InvalidPoolOutAmount {});
                }
                if uint128_pool_out_amount.is_zero() {
                    return Err(ContractError::InvalidPoolOutAmount {});
                }

                // Get pool creation fee vector
                let pool_creation_fee_vec = get_pool_creation_fee(&deps)?;
                // Pool out amount is separate from out asset to be streamed.
                let pool_out_amount = Coin {
                    denom: out_asset.denom.clone(),
                    amount: uint128_pool_out_amount,
                };
                // Add the pool out amount to instantiate funds as well
                instantiate_funds.push(pool_out_amount.clone());
                // Add the pool out amount to expected funds
                expected_funds.push(pool_out_amount);
                // Merge the pool creation fee with instantiate funds
                instantiate_funds.extend(pool_creation_fee_vec.clone());
                // Merge the pool creation fee with expected funds
                expected_funds.extend(pool_creation_fee_vec);
            }
        }
    }
    check_payment(&info.funds, &expected_funds)?;

    let last_stream_id = LAST_STREAM_ID.load(deps.storage)?;
    let stream_id = last_stream_id + 1;

    let stream_swap_inst_message: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Instantiate2 {
        code_id: params.stream_contract_code_id,
        admin: Some(params.protocol_admin.to_string()),
        label: format!("Stream Swap Stream {} - {}", name, stream_id),
        msg: to_json_binary(&msg)?,
        funds: instantiate_funds,
        salt: salt.clone(),
    });

    let checksum = deps
        .querier
        .query_wasm_code_info(params.stream_contract_code_id)?
        .checksum;
    let canonical_contract_addr = cosmwasm_std::instantiate2_address(
        checksum.as_slice(),
        &deps.api.addr_canonicalize(info.sender.as_ref())?,
        salt.as_slice(),
    )?;

    LAST_STREAM_ID.save(deps.storage, &stream_id)?;

    let contract_addr = deps.api.addr_humanize(&canonical_contract_addr)?;
    STREAMS.save(deps.storage, stream_id, &contract_addr)?;

    let mut msgs = vec![];

    msgs.push(stream_swap_inst_message.clone());
    if !stream_creation_fee.amount.is_zero() {
        msgs.push(CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
            to_address: params.fee_collector.to_string(),
            amount: vec![stream_creation_fee.clone()],
        }));
    }

    let res = Response::new()
        .add_messages(msgs)
        .add_attribute("action", "create_stream")
        .add_attribute("stream_id", stream_id.to_string())
        .add_attribute("stream_contract_addr", contract_addr.to_string())
        .add_attribute("stream_creation_fee", stream_creation_fee.to_string())
        .add_attribute("out_asset", out_asset.to_string())
        .add_attribute("in_denom", in_denom)
        .add_attribute(
            "bootstrapping_start_time",
            msg.bootstraping_start_time.to_string(),
        )
        .add_attribute("start_time", start_time.to_string())
        .add_attribute("end_time", end_time.to_string())
        .add_attribute("treasury", treasury.to_string())
        .add_attribute("name", name);
    Ok(res)
}

pub fn execute_update_params(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    min_waiting_duration: Option<u64>,
    min_bootstrapping_duration: Option<u64>,
    min_stream_duration: Option<u64>,
    stream_creation_fee: Option<Coin>,
    fee_collector: Option<String>,
    accepted_in_denoms: Option<Vec<String>>,
    exit_fee_percent: Option<Decimal256>,
) -> Result<Response, ContractError> {
    let mut params = PARAMS.load(deps.storage)?;
    if info.sender != params.protocol_admin {
        return Err(ContractError::Unauthorized {});
    }
    if let Some(stream_creation_fee) = stream_creation_fee {
        params.stream_creation_fee = stream_creation_fee;
    }

    if let Some(exit_fee_percent) = exit_fee_percent {
        if exit_fee_percent > Decimal256::percent(100) || exit_fee_percent < Decimal256::percent(0)
        {
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
    if let Some(min_waiting_duration) = min_waiting_duration {
        params.min_waiting_duration = min_waiting_duration;
    }
    if let Some(min_bootstrapping_duration) = min_bootstrapping_duration {
        params.min_bootstrapping_duration = min_bootstrapping_duration;
    }
    if let Some(min_stream_duration) = min_stream_duration {
        params.min_stream_duration = min_stream_duration;
    }

    PARAMS.save(deps.storage, &params)?;

    let res = Response::new()
        .add_attribute("action", "update_params")
        .add_attributes(params.to_attributes());
    Ok(res)
}

pub fn execute_freeze(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let params = PARAMS.load(deps.storage)?;
    if info.sender != params.protocol_admin {
        return Err(ContractError::Unauthorized {});
    }
    FREEZESTATE.save(deps.storage, &true)?;

    Ok(Response::new().add_attribute("action", "freeze"))
}

pub fn execute_unfreeze(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let params = PARAMS.load(deps.storage)?;
    if info.sender != params.protocol_admin {
        return Err(ContractError::Unauthorized {});
    }
    FREEZESTATE.save(deps.storage, &false)?;

    Ok(Response::new().add_attribute("action", "unfreeze"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Params {} => to_json_binary(&PARAMS.load(deps.storage)?),
        QueryMsg::Freezestate {} => to_json_binary(&FREEZESTATE.load(deps.storage)?),
        QueryMsg::LastStreamId {} => to_json_binary(&LAST_STREAM_ID.load(deps.storage)?),
        QueryMsg::ListStreams { start_after, limit } => {
            to_json_binary(&list_streams(deps, start_after, limit)?)
        }
    }
}

pub fn list_streams(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<StreamsResponse> {
    const MAX_LIMIT: u32 = 30;
    let start = start_after.map(Bound::exclusive);
    let limit = limit.unwrap_or(MAX_LIMIT).min(MAX_LIMIT) as usize;
    let streams: StdResult<Vec<StreamResponse>> = STREAMS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (id, address) = item?;
            let stream = StreamResponse {
                id,
                address: address.to_string(),
            };
            Ok(stream)
        })
        .collect();
    let streams = streams?;
    Ok(StreamsResponse { streams })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    ensure_from_older_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
