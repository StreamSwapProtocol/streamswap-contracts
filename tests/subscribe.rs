use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::Timestamp;
use cosmwasm_std::{Coin, Decimal256, Uint256};
use cw_streamswap::{contract::execute, ContractError};
use cw_utils::PaymentError;
mod helpers;

#[test]
fn subscribe_stream_ended() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Create a stream first
    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(2000))
        .end_time(Timestamp::from_seconds(3000));
    let env = helpers::env_at(0);
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

    // Try to subscribe after stream has ended
    let env = helpers::env_at(4000); // After end time
    let info = helpers::mock_info("user1", &[Coin::new(1000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };

    let res = execute(deps.as_mut(), env, info, msg);
    assert_eq!(res.unwrap_err(), ContractError::StreamEnded {});
}

#[test]
fn subscribe_no_funds() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Create a stream first
    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(2000))
        .end_time(Timestamp::from_seconds(10000));
    let env = helpers::env_at(0);
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

    // Try to subscribe without funds
    let env = helpers::env_at(2500); // During stream
    let info = helpers::mock_info("user1", &[]); // No funds
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };

    let res = execute(deps.as_mut(), env, info, msg);
    assert_eq!(res.unwrap_err(), PaymentError::NoFunds {}.into());
}

#[test]
fn subscribe_incorrect_denom() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Create a stream first
    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(2000))
        .end_time(Timestamp::from_seconds(10000));
    let env = helpers::env_at(0);
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

    // Try to subscribe with wrong denom
    let env = helpers::env_at(2500); // During stream
    let info = helpers::mock_info("user1", &[Coin::new(1000u128, "wrong_denom")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };

    let res = execute(deps.as_mut(), env, info, msg);
    assert_eq!(
        res.unwrap_err(),
        PaymentError::MissingDenom("in".to_string()).into()
    );
}

#[test]
fn subscribe_incorrect_tos_version() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Create a stream first
    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(2000))
        .end_time(Timestamp::from_seconds(10000));
    let env = helpers::env_at(0);
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

    // Try to subscribe with incorrect ToS version
    let env = helpers::env_at(2500); // During stream
    let info = helpers::mock_info("user1", &[Coin::new(1000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "random".to_string(), // Invalid ToS version
    };

    let res = execute(deps.as_mut(), env, info, msg);
    assert_eq!(res.unwrap_err(), ContractError::InvalidToSVersion {});
}

#[test]
fn subscribe_first_subscription() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Create a stream first
    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(2000))
        .end_time(Timestamp::from_seconds(10000));
    let env = helpers::env_at(0);
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
    let env = helpers::env_at(2500); // During stream
    let info = helpers::mock_info("user1", &[Coin::new(1000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };

    let res = execute(deps.as_mut(), env, info.clone(), msg);
    assert!(res.is_ok());

    // Verify the response attributes
    let response = res.unwrap();
    assert_eq!(response.attributes[0].key, "action");
    assert_eq!(response.attributes[0].value, "subscribe");

    // Query stream to verify status changed to Active
    let query_env = helpers::env_at(2500);
    let stream =
        cw_streamswap::contract::query_stream(deps.as_ref(), query_env.clone(), 1).unwrap();
    assert_eq!(stream.status, cw_streamswap::state::Status::Active);
    assert_eq!(stream.in_supply, Uint256::from(1000u128));

    // Verify distribution index is still 0 on first subscription (no distribution yet)
    assert_eq!(stream.dist_index, Decimal256::zero());

    // Query position to verify user details
    let position = cw_streamswap::contract::query_position(
        deps.as_ref(),
        query_env,
        1,
        info.sender.to_string(),
    )
    .unwrap();
    assert_eq!(position.index, Decimal256::zero());
    assert_eq!(position.in_balance, Uint256::from(1000u128));
    assert_eq!(position.shares, Uint256::from(1000u128));
    assert_eq!(position.pending_purchase, Decimal256::zero());
    assert_eq!(position.purchased, Uint256::zero());
    assert_eq!(position.spent, Uint256::zero());
}
