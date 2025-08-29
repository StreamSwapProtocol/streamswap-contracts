use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::{Addr, BankMsg, Coin, CosmosMsg, Timestamp, Uint256};
use cw_streamswap::contract::{execute, execute_exit_stream, execute_update_stream};
use cw_streamswap::ContractError;

mod helpers;

#[test]
fn exit_cannot_before_stream_ends() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors (addresses) and a helper to build MessageInfo
    let treasury: Addr = helpers::mock_info("creator", &[]).sender;
    let subscriber1: Addr = helpers::mock_info("subscriber1", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Create stream
    let start = Timestamp::from_seconds(1_000_000);
    let end = Timestamp::from_seconds(5_000_000);
    let out_supply = Uint256::from(1_000_000_000_000u128);
    let b = helpers::CreateStreamBuilder::default()
        .start_time(start)
        .end_time(end)
        .out_supply(out_supply)
        .out_denom("out_denom");
    let env = helpers::env_at(0);
    let funds = vec![
        Coin {
            denom: "out_denom".to_string(),
            amount: out_supply,
        },
        Coin {
            denom: helpers::DEFAULT_STREAM_CREATION_DENOM.to_string(),
            amount: Uint256::from(100u128),
        },
    ];
    let info = mk_info(&treasury, funds);
    execute(deps.as_mut(), env, info, b.build()).unwrap();

    // First subscription
    let env = helpers::env_at(start.seconds() + 1_000_000);
    let info = mk_info(&subscriber1, vec![Coin::new(2_000_000_000_000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // Try to exit during stream
    let env = helpers::env_at(start.seconds() + 2_000_000);
    let info = mk_info(&subscriber1, vec![]);
    let err = execute_exit_stream(deps.as_mut(), env, info, 1, None).unwrap_err();
    assert_eq!(err, ContractError::StreamNotEnded {});
}

#[test]
fn exit_unauthorized_on_behalf_fails() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors (addresses) and a helper to build MessageInfo
    let treasury: Addr = helpers::mock_info("creator", &[]).sender;
    let subscriber1: Addr = helpers::mock_info("subscriber1", &[]).sender;
    let unauthorized: Addr = helpers::mock_info("unauthorized", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Create stream
    let start = Timestamp::from_seconds(1_000_000);
    let end = Timestamp::from_seconds(5_000_000);
    let out_supply = Uint256::from(1_000_000_000_000u128);
    let b = helpers::CreateStreamBuilder::default()
        .start_time(start)
        .end_time(end)
        .out_supply(out_supply)
        .out_denom("out_denom");
    let env = helpers::env_at(0);
    let funds = vec![
        Coin {
            denom: "out_denom".to_string(),
            amount: out_supply,
        },
        Coin {
            denom: helpers::DEFAULT_STREAM_CREATION_DENOM.to_string(),
            amount: Uint256::from(100u128),
        },
    ];
    let info = mk_info(&treasury, funds);
    execute(deps.as_mut(), env, info, b.build()).unwrap();

    // First subscription
    let env = helpers::env_at(start.seconds() + 1_000_000);
    let info = mk_info(&subscriber1, vec![Coin::new(2_000_000_000_000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // After end, random cannot exit on behalf
    let env = helpers::env_at(end.seconds() + 3_000_000);
    let info = mk_info(&unauthorized, vec![]);
    let err = execute_exit_stream(deps.as_mut(), env, info, 1, Some(subscriber1.to_string()))
        .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn exit_owner_success_and_position_deleted() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors (addresses) and a helper to build MessageInfo
    let treasury: Addr = helpers::mock_info("creator", &[]).sender;
    let subscriber1: Addr = helpers::mock_info("subscriber1", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Create stream
    let start = Timestamp::from_seconds(1_000_000);
    let end = Timestamp::from_seconds(5_000_000);
    let out_supply = Uint256::from(1_000_000_000_000u128);
    let b = helpers::CreateStreamBuilder::default()
        .start_time(start)
        .end_time(end)
        .out_supply(out_supply)
        .out_denom("out_denom");
    let env = helpers::env_at(0);
    let funds = vec![
        Coin {
            denom: "out_denom".to_string(),
            amount: out_supply,
        },
        Coin {
            denom: helpers::DEFAULT_STREAM_CREATION_DENOM.to_string(),
            amount: Uint256::from(100u128),
        },
    ];
    let info = mk_info(&treasury, funds);
    execute(deps.as_mut(), env, info, b.build()).unwrap();

    // First subscription
    let env = helpers::env_at(start.seconds() + 1_000_000);
    let info = mk_info(&subscriber1, vec![Coin::new(2_000_000_000_000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // Exit after end
    let env = helpers::env_at(end.seconds() + 3_000_000);
    let info = mk_info(&subscriber1, vec![]);
    let res = execute_exit_stream(deps.as_mut(), env, info, 1, None).unwrap();
    assert_eq!(
        res.messages,
        vec![cosmwasm_std::SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: subscriber1.to_string(),
            amount: vec![Coin::new(1_000_000_000_000u128, "out_denom")],
        }))]
    );

    // Position deleted
    let env = helpers::env_at(end.seconds() + 4_000_000);
    let err =
        cw_streamswap::contract::query_position(deps.as_ref(), env, 1, subscriber1.to_string())
            .unwrap_err();
    // We expect a not found error; comparing by string to avoid importing StdError variants here
    assert!(err.to_string().contains("not found"));
}

#[test]
fn exit_withdraw_all_before_exit_two_users() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors (addresses) and a helper to build MessageInfo
    let treasury: Addr = helpers::mock_info("creator", &[]).sender;
    let subscriber1: Addr = helpers::mock_info("subscriber1", &[]).sender;
    let subscriber2: Addr = helpers::mock_info("subscriber2", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Create stream
    let start = Timestamp::from_seconds(1_000_000);
    let end = Timestamp::from_seconds(5_000_000);
    let out_supply = Uint256::from(1_000_000_000_000u128);
    let b = helpers::CreateStreamBuilder::default()
        .start_time(start)
        .end_time(end)
        .out_supply(out_supply)
        .out_denom("out_denom");
    let env = helpers::env_at(0);
    let funds = vec![
        Coin {
            denom: "out_denom".to_string(),
            amount: out_supply,
        },
        Coin {
            denom: helpers::DEFAULT_STREAM_CREATION_DENOM.to_string(),
            amount: Uint256::from(100u128),
        },
    ];
    let info = mk_info(&treasury, funds);
    execute(deps.as_mut(), env, info, b.build()).unwrap();

    // Subscriptions
    let env = helpers::env_at(start.seconds() + 1_000_000);
    let info = mk_info(&subscriber1, vec![Coin::new(2_000_000_000_000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    let env = helpers::env_at(start.seconds() + 1_000_000);
    let info = mk_info(&subscriber2, vec![Coin::new(1_000_000_000_000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // Withdraw all before end for both
    let env = helpers::env_at(end.seconds() - 1_000_000);
    let info = mk_info(&subscriber1, vec![]);
    let msg = cw_streamswap::msg::ExecuteMsg::Withdraw {
        stream_id: 1,
        cap: None,
        operator_target: None,
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    let env = helpers::env_at(end.seconds() - 1_000_000);
    let info = mk_info(&subscriber2, vec![]);
    let msg = cw_streamswap::msg::ExecuteMsg::Withdraw {
        stream_id: 1,
        cap: None,
        operator_target: None,
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // Ensure stream updated and both can exit after end
    let env = helpers::env_at(end.seconds() + 1_000_000);
    execute_update_stream(deps.as_mut(), env, 1).unwrap();

    let env = helpers::env_at(end.seconds() + 1_000_001);
    let info = mk_info(&subscriber1, vec![]);
    execute_exit_stream(deps.as_mut(), env, info, 1, None).unwrap();

    let env = helpers::env_at(end.seconds() + 1_000_002);
    let info = mk_info(&subscriber2, vec![]);
    execute_exit_stream(deps.as_mut(), env, info, 1, None).unwrap();
}
