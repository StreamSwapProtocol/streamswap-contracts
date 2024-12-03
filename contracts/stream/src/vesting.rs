use crate::ContractError;
use cosmwasm_std::{
    attr, coin, to_json_binary, Addr, Attribute, Binary, CosmosMsg, DepsMut, HexBinary, Timestamp,
    Uint128, WasmMsg,
};
use cw_vesting::msg::InstantiateMsg as VestingInstantiateMsg;
use cw_vesting::UncheckedDenom;
use streamswap_types::controller::VestingConfig;

pub fn vesting_operations(
    deps: &DepsMut,
    stream_addr: Addr,
    vesting_checksum: HexBinary,
    recipient: Addr,
    salt: Option<Binary>,
    start_time: Timestamp,
    vesting_code_id: u64,
    amount: Uint128,
    denom: String,
    vesting_config: VestingConfig,
) -> Result<(Vec<CosmosMsg>, Vec<Attribute>, Addr), ContractError> {
    let salt = salt.ok_or(ContractError::InvalidSalt {})?;

    let vesting_title = format!("Stream addr {} released to {}", stream_addr, recipient);
    let vesting_instantiate_msg = VestingInstantiateMsg {
        owner: None,
        title: vesting_title,
        recipient: recipient.to_string(),
        description: None,
        total: amount,
        denom: UncheckedDenom::Native(denom.clone()),
        schedule: vesting_config.schedule,
        start_time: Some(start_time),
        vesting_duration_seconds: vesting_config.vesting_duration_seconds,
        unbonding_duration_seconds: vesting_config.unbonding_duration_seconds,
    };

    // Calculate the address of the new contract
    let vesting_address = deps.api.addr_humanize(&cosmwasm_std::instantiate2_address(
        vesting_checksum.as_ref(),
        &deps.api.addr_canonicalize(stream_addr.as_str())?,
        &salt,
    )?)?;

    let vesting_instantiate_msg = WasmMsg::Instantiate2 {
        admin: None,
        code_id: vesting_code_id,
        label: format!("{}-{}", denom.clone(), recipient),
        msg: to_json_binary(&vesting_instantiate_msg)?,
        funds: vec![coin(amount.u128(), denom.clone())],
        salt,
    };

    let messages = vec![vesting_instantiate_msg.into()];
    let attributes = vec![attr("vesting_address", vesting_address.clone())];

    Ok((messages, attributes, vesting_address))
}
