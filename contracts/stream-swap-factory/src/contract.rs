use cosmwasm_std::{
    entry_point, to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw_controllers::Admin;
use cw_utils::maybe_addr;

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg},
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
    } = msg;

    let admin = deps
        .api
        .addr_validate(&admin.unwrap_or(info.sender.to_string()))?;

    if exit_fee_percent > Decimal::percent(100) || exit_fee_percent < Decimal::percent(0) {
        return Err(ContractError::InvalidExitFeePercent {});
    }

    let params = Params {
        admin: admin.clone(),
        stream_creation_fee: stream_creation_fee.clone(),
        exit_fee_percent,
        stream_swap_code_id,
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
    match msg {}
}
