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

    // Define actors
    let treasury = helpers::mock_info("creator", &[]);
    let user1 = helpers::mock_info("user1", &[]);

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
    let mut treasury_funded = treasury.clone();
    treasury_funded.funds = funds;
    execute(deps.as_mut(), env, treasury_funded, b.build()).unwrap();

    // Try to subscribe after stream has ended
    let env = helpers::env_at(4000); // After end time
    let mut user1_funded = user1.clone();
    user1_funded.funds = vec![Coin::new(1000u128, "in")];
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };

    let res = execute(deps.as_mut(), env, user1_funded, msg);
    assert_eq!(res.unwrap_err(), ContractError::StreamEnded {});
}

#[test]
fn subscribe_no_funds() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors
    let treasury = helpers::mock_info("creator", &[]);
    let user1 = helpers::mock_info("user1", &[]);

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
    let mut treasury_funded = treasury.clone();
    treasury_funded.funds = funds;
    execute(deps.as_mut(), env, treasury_funded, b.build()).unwrap();

    // Try to subscribe without funds
    let env = helpers::env_at(2500); // During stream
    let info = user1.clone(); // No funds
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

    // Define actors
    let treasury = helpers::mock_info("creator", &[]);
    let user1 = helpers::mock_info("user1", &[]);

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
    let mut treasury_funded = treasury.clone();
    treasury_funded.funds = funds;
    execute(deps.as_mut(), env, treasury_funded, b.build()).unwrap();

    // Try to subscribe with wrong denom
    let env = helpers::env_at(2500); // During stream
    let mut user1_funded = user1.clone();
    user1_funded.funds = vec![Coin::new(1000u128, "wrong_denom")];
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };

    let res = execute(deps.as_mut(), env, user1_funded, msg);
    assert_eq!(
        res.unwrap_err(),
        PaymentError::MissingDenom("in".to_string()).into()
    );
}

#[test]
fn subscribe_incorrect_tos_version() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors
    let treasury = helpers::mock_info("creator", &[]);
    let user1 = helpers::mock_info("user1", &[]);

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
    let mut treasury_funded = treasury.clone();
    treasury_funded.funds = funds;
    execute(deps.as_mut(), env, treasury_funded, b.build()).unwrap();

    // Try to subscribe with incorrect ToS version
    let env = helpers::env_at(2500); // During stream
    let mut user1_funded = user1.clone();
    user1_funded.funds = vec![Coin::new(1000u128, "in")];
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "random".to_string(), // Invalid ToS version
    };

    let res = execute(deps.as_mut(), env, user1_funded, msg);
    assert_eq!(res.unwrap_err(), ContractError::InvalidToSVersion {});
}

#[test]
fn subscribe_first_subscription() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors
    let treasury = helpers::mock_info("creator", &[]);
    let user1 = helpers::mock_info("user1", &[]);

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
    let mut treasury_funded = treasury.clone();
    treasury_funded.funds = funds;
    execute(deps.as_mut(), env, treasury_funded, b.build()).unwrap();

    // First subscription
    let env = helpers::env_at(2500); // During stream
    let mut user1_funded = user1.clone();
    user1_funded.funds = vec![Coin::new(1000u128, "in")];
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };

    let res = execute(deps.as_mut(), env, user1_funded.clone(), msg);
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
        user1_funded.sender.to_string(),
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

    // Define actors
    let treasury = helpers::mock_info("creator", &[]);
    let user1 = helpers::mock_info("user1", &[]);

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
    let mut treasury_funded = treasury.clone();
    treasury_funded.funds = funds;
    execute(deps.as_mut(), env, treasury_funded, b.build()).unwrap();

    // First subscription
    let env = helpers::env_at(2500); // During stream
    let mut user1_funded = user1.clone();
    user1_funded.funds = vec![Coin::new(1000u128, "in")];
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };

    let res = execute(deps.as_mut(), env, user1_funded.clone(), msg);
    assert!(res.is_ok());

    // Query stream after first subscription
    let query_env = helpers::env_at(2500);
    let stream =
        cw_streamswap::contract::query_stream(deps.as_ref(), query_env.clone(), 1).unwrap();
    assert_eq!(stream.dist_index, Decimal256::zero());
    assert_eq!(stream.in_supply, Uint256::from(1000u128));

    // Second subscription (increase) by the same user
    let env = helpers::env_at(3000); // Later during stream
    let mut user1_funded = user1.clone();
    user1_funded.funds = vec![Coin::new(500u128, "in")];
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };

    let res = execute(deps.as_mut(), env, user1_funded.clone(), msg);
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
        user1_funded.sender.to_string(),
    )
    .unwrap();
    assert!(position.in_balance < Uint256::from(1500u128)); // Total balance minus spent between subscriptions
}

#[test]
fn subscribe_pending_stream() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors
    let treasury = helpers::mock_info("creator", &[]);
    let user1 = helpers::mock_info("user1", &[]);

    // Create a stream that hasn't started yet
    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(5000)) // Start in the future
        .end_time(Timestamp::from_seconds(10000))
        .out_supply(Uint256::from(1_000_000u128));
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
    let mut treasury_funded = treasury.clone();
    treasury_funded.funds = funds;
    execute(deps.as_mut(), env, treasury_funded, b.build()).unwrap();

    // Subscribe before stream starts (pending subscription)
    // Note: Treasury cancel period is active from time 0 to 1000 (min_seconds_until_start_time)
    // So we need to subscribe after time 1000 but before start time 5000
    let env = helpers::env_at(2000); // After cancel period, before start time
    let mut user1_funded = user1.clone();
    user1_funded.funds = vec![Coin::new(1000u128, "in")];
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };

    let res = execute(deps.as_mut(), env, user1_funded.clone(), msg);
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
        user1_funded.sender.to_string(),
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

    // Define actors
    let treasury = helpers::mock_info("creator", &[]);
    let user1 = helpers::mock_info("user1", &[]);

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
    let mut treasury_funded = treasury.clone();
    treasury_funded.funds = funds;
    execute(deps.as_mut(), env, treasury_funded, b.build()).unwrap();

    // Try to subscribe during treasury cancel period (time 0 to 1000)
    // This should fail with TreasuryCancelPeriodActive
    let env = helpers::env_at(500); // During cancel period
    let mut user1_funded = user1.clone();
    user1_funded.funds = vec![Coin::new(1000u128, "in")];
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };

    let res = execute(deps.as_mut(), env, user1_funded, msg);
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
fn subscribe_pending_multiple_with_transition() {
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

    // First subscription before start time (pending)
    let env = helpers::env_at(2000); // After cancel period, before start time
    let info1 = helpers::mock_info("subscriber1", &[Coin::new(1000000u128, "in")]);
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

    // Query stream after first subscription
    let query_env = helpers::env_at(2000);
    let stream = cw_streamswap::contract::query_stream(deps.as_ref(), query_env, 1).unwrap();
    assert_eq!(stream.status, cw_streamswap::state::Status::Waiting);
    assert_eq!(stream.in_supply, Uint256::from(1000000u128));
    assert_eq!(stream.shares, Uint256::from(1000000u128));

    // Second subscription still waiting (by same user)
    let env = helpers::env_at(3000); // Still before start time
    let info1_second = helpers::mock_info("subscriber1", &[Coin::new(1000000u128, "in")]);
    let msg1_second = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };

    let res1_second = execute(deps.as_mut(), env, info1_second.clone(), msg1_second);
    assert!(res1_second.is_ok());
    let response1_second = res1_second.unwrap();
    assert_eq!(response1_second.attributes[0].key, "action");
    assert_eq!(response1_second.attributes[0].value, "subscribe_pending");

    // Query stream after second subscription
    let query_env = helpers::env_at(3000);
    let stream = cw_streamswap::contract::query_stream(deps.as_ref(), query_env, 1).unwrap();
    assert_eq!(stream.status, cw_streamswap::state::Status::Waiting);
    assert_eq!(stream.in_supply, Uint256::from(2000000u128)); // 1M + 1M

    // Before stream start time: 2 subscriptions made, stream is pending
    // Subscriber1 has 2 subscriptions and 2_000_000 in balance

    // Third subscription after start time (stream becomes active)
    let env = helpers::env_at(6000); // After start time (5000)
    let info2 = helpers::mock_info("subscriber2", &[Coin::new(1000000u128, "in")]);
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
    // Different action because stream is now active
    assert_eq!(response2.attributes[0].value, "subscribe");

    // Query stream after third subscription
    let query_env = helpers::env_at(6000);
    let stream =
        cw_streamswap::contract::query_stream(deps.as_ref(), query_env.clone(), 1).unwrap();
    assert_eq!(stream.status, cw_streamswap::state::Status::Active);
    // update_stream ran at t=6000 before adding creator2's 1,000,000, spending 400,000 from 2,000,000
    // then +1,000,000 added → 1,600,000 + 1,000,000 = 2,600,000
    assert_eq!(stream.in_supply, Uint256::from(3_000_000u128 - 400_000u128));
    assert_eq!(stream.spent_in, Uint256::from(400_000u128));

    // Update creator1 position to calculate spent/purchased amounts
    let update_msg = cw_streamswap::msg::ExecuteMsg::UpdatePosition {
        stream_id: 1,
        operator_target: None,
    };
    let update_info = helpers::mock_info("subscriber1", &[]);
    let update_env = helpers::env_at(6000);
    let update_res = execute(deps.as_mut(), update_env, update_info.clone(), update_msg);
    assert!(update_res.is_ok());

    // Query subscriber1 position after update
    let position1 = cw_streamswap::contract::query_position(
        deps.as_ref(),
        query_env.clone(),
        1,
        update_info.sender.to_string(),
    )
    .unwrap();

    // At 6000 seconds, subscriber1 should have spent 400,000
    assert_eq!(position1.spent, Uint256::from(400_000u128));

    // Query stream to see updated state
    let stream_after_update =
        cw_streamswap::contract::query_stream(deps.as_ref(), query_env.clone(), 1).unwrap();
    assert_eq!(
        stream_after_update.status,
        cw_streamswap::state::Status::Active
    );
    assert!(stream_after_update.spent_in > Uint256::zero());

    // Update subscriber1 position at 7500
    let update_msg_7500 = cw_streamswap::msg::ExecuteMsg::UpdatePosition {
        stream_id: 1,
        operator_target: None,
    };
    let update_info_7500 = helpers::mock_info("subscriber1", &[]);
    let update_env_7500 = helpers::env_at(7500);
    let update_res_7500 = execute(
        deps.as_mut(),
        update_env_7500,
        update_info_7500.clone(),
        update_msg_7500,
    );
    assert!(update_res_7500.is_ok());

    // Query position at 7500 for subscriber1
    let pos1_7500 = cw_streamswap::contract::query_position(
        deps.as_ref(),
        helpers::env_at(7500),
        1,
        update_info_7500.sender.to_string(),
    )
    .unwrap();
    // 184_615 + 200_000 = 384_615
    assert_eq!(
        pos1_7500.purchased,
        Uint256::from(184_615u128 + 200_000u128)
    );
    assert_eq!(pos1_7500.spent, Uint256::from(2_000_000u128 / 2u128));

    // Update creator2 position at 3500
    let update_msg2 = cw_streamswap::msg::ExecuteMsg::UpdatePosition {
        stream_id: 1,
        operator_target: None,
    };
    let update_info2 = helpers::mock_info("subscriber2", &[]);
    let update_env2 = helpers::env_at(3500);
    let update_res2 = execute(
        deps.as_mut(),
        update_env2,
        update_info2.clone(),
        update_msg2,
    );
    assert!(update_res2.is_ok());

    // Query position for creator2 at 3500
    let pos2_3500 = cw_streamswap::contract::query_position(
        deps.as_ref(),
        helpers::env_at(3500),
        1,
        update_info2.sender.to_string(),
    )
    .unwrap();
    assert_eq!(pos2_3500.purchased, Uint256::from(115_384u128));
    assert_eq!(
        pos2_3500.spent,
        Uint256::from(1_000_000u128 * 1_500u128 / 4_000u128)
    );

    // Query stream at 3500
    let stream_3500 =
        cw_streamswap::contract::query_stream(deps.as_ref(), helpers::env_at(3500), 1).unwrap();
    assert_eq!(stream_3500.status, cw_streamswap::state::Status::Active);
    assert_eq!(stream_3500.in_supply, Uint256::from(1_625_000u128));
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
