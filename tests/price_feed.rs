use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::{Addr, Coin, Decimal256, Timestamp, Uint256};
use cw_streamswap::contract::{execute, execute_create_stream, execute_update_stream};
use std::str::FromStr;

mod helpers;

#[test]
fn price_feed_initial_price_before_update() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors (addresses) and a helper to build MessageInfo
    let treasury: Addr = helpers::mock_info("creator", &[]).sender;
    let subscriber1: Addr = helpers::mock_info("creator1", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Create stream
    let start = Timestamp::from_seconds(1_000_000);
    let end = Timestamp::from_seconds(5_000_000);
    let out_supply = Uint256::from(1_000_000u128);
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
    execute_create_stream(
        deps.as_mut(),
        env,
        info,
        treasury.to_string(),
        "test".to_string(),
        Some("https://sample.url".to_string()),
        "in".to_string(),
        "out_denom".to_string(),
        out_supply,
        start,
        end,
        None,
        "v1".to_string(),
    )
    .unwrap();

    // First subscription
    let env = helpers::env_at(start.seconds() + 1_000_000);
    let info = mk_info(&subscriber1, vec![Coin::new(3_000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // Check current streamed price before update
    let env = helpers::env_at(start.seconds() + 2_000_000);
    let res = cw_streamswap::contract::query_last_streamed_price(deps.as_ref(), env, 1).unwrap();
    assert_eq!(res.current_streamed_price, Decimal256::zero());
}

#[test]
fn price_feed_price_after_first_update() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors (addresses) and a helper to build MessageInfo
    let treasury: Addr = helpers::mock_info("creator", &[]).sender;
    let subscriber1: Addr = helpers::mock_info("creator1", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Create stream
    let start = Timestamp::from_seconds(1_000_000);
    let end = Timestamp::from_seconds(5_000_000);
    let out_supply = Uint256::from(1_000_000u128);
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
    execute_create_stream(
        deps.as_mut(),
        env,
        info,
        treasury.to_string(),
        "test".to_string(),
        Some("https://sample.url".to_string()),
        "in".to_string(),
        "out_denom".to_string(),
        out_supply,
        start,
        end,
        None,
        "v1".to_string(),
    )
    .unwrap();

    // First subscription
    let env = helpers::env_at(start.seconds() + 1_000_000);
    let info = mk_info(&subscriber1, vec![Coin::new(3_000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // Check current streamed price after update
    let env = helpers::env_at(start.seconds() + 2_000_000);
    execute_update_stream(deps.as_mut(), env, 1).unwrap();
    let res = cw_streamswap::contract::query_last_streamed_price(
        deps.as_ref(),
        helpers::env_at(start.seconds() + 2_000_000),
        1,
    )
    .unwrap();
    // approx 1000/333333
    assert_eq!(
        res.current_streamed_price,
        Decimal256::from_str("0.002997002997002997").unwrap()
    );
}

#[test]
fn price_feed_price_with_two_subscribers() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors (addresses) and a helper to build MessageInfo
    let treasury: Addr = helpers::mock_info("creator", &[]).sender;
    let subscriber1: Addr = helpers::mock_info("creator1", &[]).sender;
    let subscriber2: Addr = helpers::mock_info("creator2", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Create stream
    let start = Timestamp::from_seconds(1_000_000);
    let end = Timestamp::from_seconds(5_000_000);
    let out_supply = Uint256::from(1_000_000u128);
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
    execute_create_stream(
        deps.as_mut(),
        env,
        info,
        treasury.to_string(),
        "test".to_string(),
        Some("https://sample.url".to_string()),
        "in".to_string(),
        "out_denom".to_string(),
        out_supply,
        start,
        end,
        None,
        "v1".to_string(),
    )
    .unwrap();

    // First subscription
    let env = helpers::env_at(start.seconds() + 1_000_000);
    let info = mk_info(&subscriber1, vec![Coin::new(3_000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // Update stream after first subscription
    let env = helpers::env_at(start.seconds() + 2_000_000);
    execute_update_stream(deps.as_mut(), env, 1).unwrap();

    // Second subscription
    let env = helpers::env_at(start.seconds() + 2_000_000);
    let info = mk_info(&subscriber2, vec![Coin::new(1_000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // Check current streamed price before update
    let env = helpers::env_at(start.seconds() + 3_000_000);
    let res = cw_streamswap::contract::query_last_streamed_price(deps.as_ref(), env, 1).unwrap();
    assert_eq!(
        res.current_streamed_price,
        Decimal256::from_str("0.002997002997002997").unwrap()
    );

    // Check current streamed price after update
    let env = helpers::env_at(start.seconds() + 3_000_000);
    execute_update_stream(deps.as_mut(), env, 1).unwrap();
    let res = cw_streamswap::contract::query_last_streamed_price(
        deps.as_ref(),
        helpers::env_at(start.seconds() + 3_000_000),
        1,
    )
    .unwrap();
    // approx 2000/333333
    assert_eq!(
        res.current_streamed_price,
        Decimal256::from_str("0.0045000045000045").unwrap()
    );
}

#[test]
fn price_feed_average_price() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors (addresses) and a helper to build MessageInfo
    let treasury: Addr = helpers::mock_info("creator", &[]).sender;
    let subscriber1: Addr = helpers::mock_info("creator1", &[]).sender;
    let subscriber2: Addr = helpers::mock_info("creator2", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Create stream
    let start = Timestamp::from_seconds(1_000_000);
    let end = Timestamp::from_seconds(5_000_000);
    let out_supply = Uint256::from(1_000_000u128);
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
    execute_create_stream(
        deps.as_mut(),
        env,
        info,
        treasury.to_string(),
        "test".to_string(),
        Some("https://sample.url".to_string()),
        "in".to_string(),
        "out_denom".to_string(),
        out_supply,
        start,
        end,
        None,
        "v1".to_string(),
    )
    .unwrap();

    // First subscription
    let env = helpers::env_at(start.seconds() + 1_000_000);
    let info = mk_info(&subscriber1, vec![Coin::new(3_000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // Update stream after first subscription
    let env = helpers::env_at(start.seconds() + 2_000_000);
    execute_update_stream(deps.as_mut(), env, 1).unwrap();

    // Second subscription
    let env = helpers::env_at(start.seconds() + 2_000_000);
    let info = mk_info(&subscriber2, vec![Coin::new(1_000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // Update stream after second subscription
    let env = helpers::env_at(start.seconds() + 3_000_000);
    execute_update_stream(deps.as_mut(), env, 1).unwrap();

    // Check average streamed price
    let env = helpers::env_at(start.seconds() + 3_000_000);
    let res = cw_streamswap::contract::query_average_price(deps.as_ref(), env, 1).unwrap();
    // approx 2500/333333
    assert_eq!(
        res.average_price,
        Decimal256::from_str("0.003748503748503748").unwrap()
    );
}

#[test]
fn price_feed_price_after_withdraw() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors (addresses) and a helper to build MessageInfo
    let treasury: Addr = helpers::mock_info("creator", &[]).sender;
    let subscriber1: Addr = helpers::mock_info("creator1", &[]).sender;
    let subscriber2: Addr = helpers::mock_info("creator2", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Create stream
    let start = Timestamp::from_seconds(1_000_000);
    let end = Timestamp::from_seconds(5_000_000);
    let out_supply = Uint256::from(1_000_000u128);
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
    execute_create_stream(
        deps.as_mut(),
        env,
        info,
        treasury.to_string(),
        "test".to_string(),
        Some("https://sample.url".to_string()),
        "in".to_string(),
        "out_denom".to_string(),
        out_supply,
        start,
        end,
        None,
        "v1".to_string(),
    )
    .unwrap();

    // First subscription
    let env = helpers::env_at(start.seconds() + 1_000_000);
    let info = mk_info(&subscriber1, vec![Coin::new(3_000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // Update stream after first subscription
    let env = helpers::env_at(start.seconds() + 2_000_000);
    execute_update_stream(deps.as_mut(), env, 1).unwrap();

    // Second subscription
    let env = helpers::env_at(start.seconds() + 2_000_000);
    let info = mk_info(&subscriber2, vec![Coin::new(1_000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // Update stream after second subscription
    let env = helpers::env_at(start.seconds() + 3_000_000);
    execute_update_stream(deps.as_mut(), env, 1).unwrap();

    // Withdraw creator 1
    let env = helpers::env_at(start.seconds() + 3_500_000);
    let info = mk_info(&subscriber1, vec![]);
    let msg = cw_streamswap::msg::ExecuteMsg::Withdraw {
        stream_id: 1,
        cap: None,
        operator_target: None,
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    let res = cw_streamswap::contract::query_last_streamed_price(
        deps.as_ref(),
        helpers::env_at(start.seconds() + 3_500_000),
        1,
    )
    .unwrap();
    assert_eq!(
        res.current_streamed_price,
        Decimal256::from_str("0.004499991000017999").unwrap()
    );

    // Test price after withdraw and update
    let env = helpers::env_at(start.seconds() + 3_750_000);
    execute_update_stream(deps.as_mut(), env, 1).unwrap();
    let res = cw_streamswap::contract::query_last_streamed_price(
        deps.as_ref(),
        helpers::env_at(start.seconds() + 3_750_000),
        1,
    )
    .unwrap();
    // approx 2500/333333
    assert_eq!(
        res.current_streamed_price,
        Decimal256::from_str("0.001500006000024000").unwrap()
    );
}
