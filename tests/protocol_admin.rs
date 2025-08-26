use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::{Addr, Coin, Decimal256, Timestamp, Uint256, Uint64};
use cw_streamswap::contract::execute;
use cw_streamswap::msg::ExecuteMsg;
use cw_streamswap::ContractError;

mod helpers;

#[test]
fn protocol_admin_unauthorized_update() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors (addresses) and a helper to build MessageInfo
    let random: Addr = helpers::mock_info("random", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Random cannot update protocol admin
    let env = helpers::env_at(0);
    let msg = ExecuteMsg::UpdateProtocolAdmin {
        new_protocol_admin: helpers::mock_info("new_protocol_admin", &[])
            .sender
            .to_string(),
    };
    let info = mk_info(&random, vec![]);
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn protocol_admin_successful_update() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors (addresses) and a helper to build MessageInfo
    let protocol_admin: Addr = helpers::mock_info("protocol_admin", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Protocol admin can update
    let env = helpers::env_at(0);
    let msg = ExecuteMsg::UpdateProtocolAdmin {
        new_protocol_admin: helpers::mock_info("new_protocol_admin", &[])
            .sender
            .to_string(),
    };
    let info = mk_info(&protocol_admin, vec![]);
    execute(deps.as_mut(), env, info, msg).unwrap();

    let query = cw_streamswap::contract::query_config(deps.as_ref()).unwrap();
    assert_eq!(
        query.protocol_admin,
        helpers::mock_info("new_protocol_admin", &[])
            .sender
            .to_string()
    );
}

#[test]
fn config_initial_values() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Query config and check initial values
    let config_response = cw_streamswap::contract::query_config(deps.as_ref()).unwrap();
    assert_eq!(config_response.min_stream_seconds, Uint64::new(1000));
    assert_eq!(
        config_response.min_seconds_until_start_time,
        Uint64::new(1000)
    );
    assert_eq!(config_response.stream_creation_denom, "fee".to_string());
    assert_eq!(config_response.stream_creation_fee, Uint256::from(100u128));
    assert_eq!(
        config_response.fee_collector,
        helpers::mock_info("collector", &[]).sender.to_string()
    );
    assert_eq!(
        config_response.protocol_admin,
        helpers::mock_info("protocol_admin", &[]).sender.to_string()
    );
    assert_eq!(config_response.accepted_in_denom, "in".to_string());
}

#[test]
fn config_unauthorized_update() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors (addresses) and a helper to build MessageInfo
    let random: Addr = helpers::mock_info("random", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Random user can't update config
    let env = helpers::env_at(0);
    let msg = ExecuteMsg::UpdateConfig {
        min_stream_duration: Some(Uint64::new(2000)),
        min_duration_until_start_time: Some(Uint64::new(2000)),
        stream_creation_denom: Some("fee2".to_string()),
        stream_creation_fee: Some(Uint256::from(200u128)),
        fee_collector: Some("collector2".to_string()),
        accepted_in_denom: Some("new_denom".to_string()),
        exit_fee_percent: Some(Decimal256::percent(2)),
        tos_version: None,
    };
    let info = mk_info(&random, vec![]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(res, ContractError::Unauthorized {});
}

#[test]
fn config_invalid_stream_creation_fee() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors (addresses) and a helper to build MessageInfo
    let protocol_admin: Addr = helpers::mock_info("protocol_admin", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Wrong fee amount (zero)
    let env = helpers::env_at(0);
    let msg = ExecuteMsg::UpdateConfig {
        min_stream_duration: Some(Uint64::new(2000)),
        min_duration_until_start_time: Some(Uint64::new(2000)),
        stream_creation_denom: Some("fee2".to_string()),
        stream_creation_fee: Some(Uint256::from(0u128)),
        fee_collector: Some("collector2".to_string()),
        accepted_in_denom: Some("new_denom".to_string()),
        exit_fee_percent: Some(Decimal256::percent(2)),
        tos_version: None,
    };
    let info = mk_info(&protocol_admin, vec![]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(res, ContractError::InvalidStreamCreationFee {});
}

#[test]
fn config_invalid_exit_fee_percent() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors (addresses) and a helper to build MessageInfo
    let protocol_admin: Addr = helpers::mock_info("protocol_admin", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Wrong exit fee percent (101%)
    let env = helpers::env_at(0);
    let msg = ExecuteMsg::UpdateConfig {
        min_stream_duration: Some(Uint64::new(2000)),
        min_duration_until_start_time: Some(Uint64::new(2000)),
        stream_creation_denom: Some("fee2".to_string()),
        stream_creation_fee: Some(Uint256::from(200u128)),
        fee_collector: Some("collector2".to_string()),
        accepted_in_denom: Some("new_denom".to_string()),
        exit_fee_percent: Some(Decimal256::percent(101)),
        tos_version: None,
    };
    let info = mk_info(&protocol_admin, vec![]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(res, ContractError::InvalidExitFeePercent {});
}

#[test]
fn config_successful_update() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors (addresses) and a helper to build MessageInfo
    let protocol_admin: Addr = helpers::mock_info("protocol_admin", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Protocol admin can update config
    let env = helpers::env_at(0);
    let msg = ExecuteMsg::UpdateConfig {
        min_stream_duration: Some(Uint64::new(2000)),
        min_duration_until_start_time: Some(Uint64::new(2000)),
        stream_creation_denom: Some("fee2".to_string()),
        stream_creation_fee: Some(Uint256::from(200u128)),
        fee_collector: Some(helpers::mock_info("collector2", &[]).sender.to_string()),
        accepted_in_denom: Some("new_denom".to_string()),
        exit_fee_percent: Some(Decimal256::percent(2)),
        tos_version: None,
    };
    let info = mk_info(&protocol_admin, vec![]);
    execute(deps.as_mut(), env, info, msg).unwrap();

    // Query config and check updated values
    let config_response = cw_streamswap::contract::query_config(deps.as_ref()).unwrap();
    assert_eq!(config_response.min_stream_seconds, Uint64::new(2000));
    assert_eq!(
        config_response.min_seconds_until_start_time,
        Uint64::new(2000)
    );
    assert_eq!(config_response.stream_creation_denom, "fee2".to_string());
    assert_eq!(config_response.stream_creation_fee, Uint256::from(200u128));
    assert_eq!(
        config_response.fee_collector,
        helpers::mock_info("collector2", &[]).sender.to_string()
    );
    assert_eq!(
        config_response.protocol_admin,
        helpers::mock_info("protocol_admin", &[]).sender.to_string()
    );
    assert_eq!(config_response.accepted_in_denom, "new_denom".to_string());
    assert_eq!(config_response.exit_fee_percent, Decimal256::percent(2));
}

#[test]
fn config_update_during_stream() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors (addresses) and a helper to build MessageInfo
    let protocol_admin: Addr = helpers::mock_info("protocol_admin", &[]).sender;
    let creator: Addr = helpers::mock_info("creator1", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // First update config
    let env = helpers::env_at(0);
    let msg = ExecuteMsg::UpdateConfig {
        min_stream_duration: Some(Uint64::new(2000)),
        min_duration_until_start_time: Some(Uint64::new(2000)),
        stream_creation_denom: Some("fee2".to_string()),
        stream_creation_fee: Some(Uint256::from(200u128)),
        fee_collector: Some(helpers::mock_info("collector2", &[]).sender.to_string()),
        accepted_in_denom: Some("new_denom".to_string()),
        exit_fee_percent: Some(Decimal256::percent(2)),
        tos_version: None,
    };
    let info = mk_info(&protocol_admin, vec![]);
    execute(deps.as_mut(), env, info, msg).unwrap();

    // Create stream with updated config
    let out_supply = Uint256::from(1000u128);
    let out_denom = "out";
    let start = Timestamp::from_seconds(10000);
    let end = Timestamp::from_seconds(1000000);
    let treasury = helpers::mock_info("treasury", &[]).sender.to_string();
    let env = helpers::env_at(0);
    let info = mk_info(
        &creator,
        vec![
            Coin::new(out_supply.to_string().parse::<u128>().unwrap(), out_denom),
            Coin::new(200u128, "fee2"),
        ],
    );
    cw_streamswap::contract::execute_create_stream(
        deps.as_mut(),
        env,
        info,
        treasury,
        "test".to_string(),
        Some("https://sample.url".to_string()),
        "new_denom".to_string(),
        out_denom.to_string(),
        out_supply,
        start,
        end,
        None,
        "v1".to_string(),
    )
    .unwrap();

    // Update config during stream
    let env = helpers::env_at(100000);
    let msg = ExecuteMsg::UpdateConfig {
        min_stream_duration: Some(Uint64::new(3000)),
        min_duration_until_start_time: Some(Uint64::new(4000)),
        stream_creation_denom: Some("fee3".to_string()),
        stream_creation_fee: Some(Uint256::from(300u128)),
        fee_collector: Some(helpers::mock_info("collector3", &[]).sender.to_string()),
        accepted_in_denom: Some("new_denom2".to_string()),
        exit_fee_percent: Some(Decimal256::percent(5)),
        tos_version: None,
    };
    let info = mk_info(&protocol_admin, vec![]);
    execute(deps.as_mut(), env, info, msg).unwrap();

    // Query config and check updated values
    let config_response = cw_streamswap::contract::query_config(deps.as_ref()).unwrap();
    assert_eq!(config_response.min_stream_seconds, Uint64::new(3000));
    assert_eq!(
        config_response.min_seconds_until_start_time,
        Uint64::new(4000)
    );
    assert_eq!(config_response.stream_creation_denom, "fee3".to_string());
    assert_eq!(config_response.stream_creation_fee, Uint256::from(300u128));
    assert_eq!(
        config_response.fee_collector,
        helpers::mock_info("collector3", &[]).sender.to_string()
    );
    assert_eq!(
        config_response.protocol_admin,
        helpers::mock_info("protocol_admin", &[]).sender.to_string()
    );
    assert_eq!(config_response.accepted_in_denom, "new_denom2".to_string());
    assert_eq!(config_response.exit_fee_percent, Decimal256::percent(5));

    // Check stream still has old values
    let env = helpers::env_at(100000);
    let stream_response = cw_streamswap::contract::query_stream(deps.as_ref(), env, 1).unwrap();
    assert_eq!(stream_response.exit_fee_percent, Decimal256::percent(2));
    assert_eq!(stream_response.stream_creation_fee, Uint256::from(200u128));
}
