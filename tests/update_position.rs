use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::{Coin, Decimal256, Timestamp, Uint256};
use cw_streamswap::ContractError;
use cw_streamswap::{self, contract::execute};
use std::str::FromStr;

mod helpers;

#[test]
fn update_position_unauthorized_operator() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Create a stream
    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(1_000_000))
        .end_time(Timestamp::from_seconds(5_000_000));
    let env = helpers::env_at(1);
    let funds = vec![
        Coin {
            denom: "out_denom".to_string(),
            amount: b.out_supply,
        },
        Coin {
            denom: helpers::DEFAULT_STREAM_CREATION_DENOM.to_string(),
            amount: Uint256::from(100u128),
        },
    ];
    let info = helpers::mock_info("creator", &funds);
    execute(deps.as_mut(), env, info, b.build()).unwrap();

    // First subscription
    let env = helpers::env_at(1_000_000 + 100);
    let info = helpers::mock_info("creator1", &[Coin::new(1_000_000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();

    // Non-owner cannot update position (should fail because random user has no position)
    let env = helpers::env_at(1_000_000 + 3_000_000);
    let info = helpers::mock_info("random", &[]);
    let msg = cw_streamswap::msg::ExecuteMsg::UpdatePosition {
        stream_id: 1,
        operator_target: Some(helpers::mock_info("creator1", &[]).sender.to_string()),
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn update_position_basic_flow() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Create a stream
    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(1_000_000))
        .end_time(Timestamp::from_seconds(5_000_000));
    let env = helpers::env_at(1);
    let funds = vec![
        Coin {
            denom: "out_denom".to_string(),
            amount: b.out_supply,
        },
        Coin {
            denom: helpers::DEFAULT_STREAM_CREATION_DENOM.to_string(),
            amount: Uint256::from(100u128),
        },
    ];
    let info = helpers::mock_info("creator", &funds);
    execute(deps.as_mut(), env, info, b.build()).unwrap();

    // First subscription
    let env = helpers::env_at(1_000_000 + 100);
    let info = helpers::mock_info("creator1", &[Coin::new(1_000_000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();

    // Update position
    let env = helpers::env_at(1_000_000 + 3_000_000);
    let info = helpers::mock_info("creator1", &[]);
    let msg = cw_streamswap::msg::ExecuteMsg::UpdatePosition {
        stream_id: 1,
        operator_target: None,
    };
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();

    // Query position and verify values
    let query_env = helpers::env_at(1_000_000 + 3_000_000);
    let position = cw_streamswap::contract::query_position(
        deps.as_ref(),
        query_env.clone(),
        1,
        helpers::mock_info("creator1", &[]).sender.to_string(),
    )
    .unwrap();
    assert_eq!(
        position.index,
        Decimal256::from_str("0.749993000000000000").unwrap()
    );
    assert_eq!(position.purchased, Uint256::from(749_993u128));
    assert_eq!(position.spent, Uint256::from(749_993u128));
    assert_eq!(position.in_balance, Uint256::from(250_007u128));

    // Query stream and verify dist_index
    let stream = cw_streamswap::contract::query_stream(deps.as_ref(), query_env, 1).unwrap();
    assert_eq!(
        stream.dist_index,
        Decimal256::from_str("0.749993000000000000").unwrap()
    );
}

#[test]
fn update_position_after_stream_ends() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Create a stream
    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(1_000_000))
        .end_time(Timestamp::from_seconds(5_000_000));
    let env = helpers::env_at(1);
    let funds = vec![
        Coin {
            denom: "out_denom".to_string(),
            amount: b.out_supply,
        },
        Coin {
            denom: helpers::DEFAULT_STREAM_CREATION_DENOM.to_string(),
            amount: Uint256::from(100u128),
        },
    ];
    let info = helpers::mock_info("creator", &funds);
    execute(deps.as_mut(), env, info, b.build()).unwrap();

    // First subscription
    let env = helpers::env_at(1_000_000 + 100);
    let info = helpers::mock_info("creator1", &[Coin::new(1_000_000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();

    // Update position after stream ends
    let env = helpers::env_at(5_000_000 + 1);
    let info = helpers::mock_info("creator1", &[]);
    let msg = cw_streamswap::msg::ExecuteMsg::UpdatePosition {
        stream_id: 1,
        operator_target: None,
    };
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();

    // Query stream and verify final state
    let query_env = helpers::env_at(5_000_000 + 1);
    let stream =
        cw_streamswap::contract::query_stream(deps.as_ref(), query_env.clone(), 1).unwrap();
    assert_eq!(stream.dist_index, Decimal256::from_str("1").unwrap());
    assert_eq!(stream.in_supply, Uint256::zero());

    // Query position and verify final state
    let position = cw_streamswap::contract::query_position(
        deps.as_ref(),
        query_env,
        1,
        helpers::mock_info("creator1", &[]).sender.to_string(),
    )
    .unwrap();
    assert_eq!(position.index, Decimal256::from_str("1").unwrap());
    assert_eq!(position.spent, Uint256::from(1_000_000u128));
    assert_eq!(position.in_balance, Uint256::zero());
    assert_eq!(stream.out_supply, Uint256::from(1_000_000u128));
    assert_eq!(position.purchased, stream.out_supply);
}
