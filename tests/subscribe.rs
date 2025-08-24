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

#[test]
fn subscribe_increase_subscription() {
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

    let res = execute(deps.as_mut(), env, info, msg);
    assert!(res.is_ok());

    // Query stream after first subscription
    let query_env = helpers::env_at(2500);
    let stream =
        cw_streamswap::contract::query_stream(deps.as_ref(), query_env.clone(), 1).unwrap();
    assert_eq!(stream.dist_index, Decimal256::zero());
    assert_eq!(stream.in_supply, Uint256::from(1000u128));

    // Second subscription (increase) by the same user
    let env = helpers::env_at(3000); // Later during stream
    let info = helpers::mock_info("user1", &[Coin::new(500u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };

    let res = execute(deps.as_mut(), env, info.clone(), msg);
    assert!(res.is_ok());

    // Query stream after second subscription
    let query_env = helpers::env_at(3000);
    let stream =
        cw_streamswap::contract::query_stream(deps.as_ref(), query_env.clone(), 1).unwrap();

    // Distribution index should now be updated (non-zero)
    assert!(stream.dist_index > Decimal256::zero());

    // Query position to verify user's updated details
    let position = cw_streamswap::contract::query_position(
        deps.as_ref(),
        query_env,
        1,
        info.sender.to_string(),
    )
    .unwrap();
    assert!(position.in_balance < Uint256::from(1500u128)); // Total balance minus spent between subscriptions
}

#[test]
fn subscribe_pending_stream() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Create a stream that hasn't started yet
    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(5000)) // Start in the future
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

    // Subscribe before stream starts (pending subscription)
    // Note: Treasury cancel period is active from time 0 to 1000 (min_seconds_until_start_time)
    // So we need to subscribe after time 1000 but before start time 5000
    let env = helpers::env_at(2000); // After cancel period, before start time
    let info = helpers::mock_info("user1", &[Coin::new(1000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };

    let res = execute(deps.as_mut(), env, info.clone(), msg);
    assert!(res.is_ok());

    let response = res.unwrap();
    assert_eq!(response.attributes[0].key, "action");
    assert_eq!(response.attributes[0].value, "subscribe_pending");

    // Query stream to verify it's still in Waiting status
    let query_env = helpers::env_at(2000);
    let stream =
        cw_streamswap::contract::query_stream(deps.as_ref(), query_env.clone(), 1).unwrap();
    assert_eq!(stream.status, cw_streamswap::state::Status::Waiting);
    assert_eq!(stream.in_supply, Uint256::from(1000u128));
    assert_eq!(stream.shares, Uint256::from(1000u128));

    // Query position to verify user details
    let position = cw_streamswap::contract::query_position(
        deps.as_ref(),
        query_env,
        1,
        info.sender.to_string(),
    )
    .unwrap();
    assert_eq!(position.in_balance, Uint256::from(1000u128));
    assert_eq!(position.shares, Uint256::from(1000u128));
    assert_eq!(position.index, Decimal256::zero());
}

#[test]
fn subscribe_pending_treasury_cancel_period() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Create a stream that hasn't started yet
    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(5000)) // Start in the future
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

    // Try to subscribe during treasury cancel period (time 0 to 1000)
    // This should fail with TreasuryCancelPeriodActive
    let env = helpers::env_at(500); // During cancel period
    let info = helpers::mock_info("user1", &[Coin::new(1000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };

    let res = execute(deps.as_mut(), env, info, msg);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err(),
        cw_streamswap::ContractError::TreasuryCancelPeriodActive {}
    );

    // Verify the stream still has no subscriptions
    let query_env = helpers::env_at(500);
    let stream = cw_streamswap::contract::query_stream(deps.as_ref(), query_env, 1).unwrap();
    assert_eq!(stream.in_supply, Uint256::zero());
    assert_eq!(stream.shares, Uint256::zero());
}

#[test]
fn subscribe_pending_multiple() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Create a stream that hasn't started yet
    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(5000)) // Start in the future
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

    // First subscription after cancel period but before start time
    let env = helpers::env_at(2000); // After cancel period (0-1000), before start (5000)
    let info1 = helpers::mock_info("user1", &[Coin::new(1000u128, "in")]);
    let msg1 = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };

    let res1 = execute(deps.as_mut(), env, info1.clone(), msg1);
    assert!(res1.is_ok());
    let response1 = res1.unwrap();
    assert_eq!(response1.attributes[0].key, "action");
    assert_eq!(response1.attributes[0].value, "subscribe_pending");

    // Second subscription by different user while still waiting
    let env = helpers::env_at(3000); // Still before start time
    let info2 = helpers::mock_info("user2", &[Coin::new(500u128, "in")]);
    let msg2 = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };

    let res2 = execute(deps.as_mut(), env, info2.clone(), msg2);
    assert!(res2.is_ok());
    let response2 = res2.unwrap();
    assert_eq!(response2.attributes[0].key, "action");
    assert_eq!(response2.attributes[0].value, "subscribe_pending");

    // Query stream to verify both subscriptions are recorded
    let query_env = helpers::env_at(3000);
    let stream =
        cw_streamswap::contract::query_stream(deps.as_ref(), query_env.clone(), 1).unwrap();
    assert_eq!(stream.status, cw_streamswap::state::Status::Waiting);
    assert_eq!(stream.in_supply, Uint256::from(1500u128)); // 1000 + 500
    assert_eq!(stream.shares, Uint256::from(1500u128)); // 1000 + 500

    // Query both positions to verify they're created correctly
    let position1 = cw_streamswap::contract::query_position(
        deps.as_ref(),
        query_env.clone(),
        1,
        info1.sender.to_string(),
    )
    .unwrap();
    assert_eq!(position1.in_balance, Uint256::from(1000u128));
    assert_eq!(position1.shares, Uint256::from(1000u128));
    assert_eq!(position1.index, Decimal256::zero());

    let position2 = cw_streamswap::contract::query_position(
        deps.as_ref(),
        query_env,
        1,
        info2.sender.to_string(),
    )
    .unwrap();
    assert_eq!(position2.in_balance, Uint256::from(500u128));
    assert_eq!(position2.shares, Uint256::from(500u128));
    assert_eq!(position2.index, Decimal256::zero());
}

#[test]
fn subscribe_pending_to_active_transition() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Create a stream that hasn't started yet
    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(5000)) // Start in the future
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

    // Subscribe before stream starts (pending subscription)
    let env = helpers::env_at(2000); // After cancel period, before start time
    let info = helpers::mock_info("user1", &[Coin::new(1000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };

    let res = execute(deps.as_mut(), env, info.clone(), msg);
    assert!(res.is_ok());

    // Verify stream is in Waiting status after pending subscription
    let query_env = helpers::env_at(2000);
    let stream = cw_streamswap::contract::query_stream(deps.as_ref(), query_env, 1).unwrap();
    assert_eq!(stream.status, cw_streamswap::state::Status::Waiting);

    // Now subscribe after start time - this should trigger status change to Active
    let env = helpers::env_at(5000); // After start time (5000)
    let info2 = helpers::mock_info("user2", &[Coin::new(500u128, "in")]);
    let msg2 = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };

    let res2 = execute(deps.as_mut(), env, info2.clone(), msg2);
    assert!(res2.is_ok());
    let response2 = res2.unwrap();
    assert_eq!(response2.attributes[0].key, "action");
    assert_eq!(response2.attributes[0].value, "subscribe"); // Should be "subscribe", not "subscribe_pending"

    // Verify stream status has changed to Active
    let query_env = helpers::env_at(5000);
    let stream =
        cw_streamswap::contract::query_stream(deps.as_ref(), query_env.clone(), 1).unwrap();
    assert_eq!(stream.status, cw_streamswap::state::Status::Active);
    assert_eq!(stream.in_supply, Uint256::from(1500u128)); // 1000 + 500
    assert_eq!(stream.shares, Uint256::from(1500u128)); // 1000 + 500

    // Verify both positions exist
    let position1 = cw_streamswap::contract::query_position(
        deps.as_ref(),
        query_env.clone(),
        1,
        info.sender.to_string(),
    )
    .unwrap();
    assert_eq!(position1.in_balance, Uint256::from(1000u128));
    assert_eq!(position1.shares, Uint256::from(1000u128));

    let position2 = cw_streamswap::contract::query_position(
        deps.as_ref(),
        query_env,
        1,
        info2.sender.to_string(),
    )
    .unwrap();
    assert_eq!(position2.in_balance, Uint256::from(500u128));
    assert_eq!(position2.shares, Uint256::from(500u128));
}
