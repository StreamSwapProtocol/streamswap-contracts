use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::{BankMsg, Coin, Decimal256, Uint256};
use cosmwasm_std::{CosmosMsg, Timestamp};
use cw_streamswap::contract::execute;
mod helpers;

#[test]
fn withdraw_pending_basic_flow() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Create a stream that hasn't started yet
    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(2000)) // Start in the future
        .end_time(Timestamp::from_seconds(1_000_000));
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

    // First subscribe before start time (pending)
    let env = helpers::env_at(2000 - 1); // Before start time
    let info = helpers::mock_info("subscriber1", &[Coin::new(1_000_000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    let res = execute(deps.as_mut(), env, info, msg);
    assert!(res.is_ok());

    // Update subscriber1 position - no distribution expected
    let env = helpers::env_at(2000 - 1);
    let update_msg = cw_streamswap::msg::ExecuteMsg::UpdatePosition {
        stream_id: 1,
        operator_target: None,
    };
    let update_info = helpers::mock_info("subscriber1", &[]);
    let res = execute(deps.as_mut(), env.clone(), update_info, update_msg);
    assert!(res.is_ok());

    let response = res.unwrap();
    assert_eq!(response.attributes[0].key, "action");
    assert_eq!(response.attributes[0].value, "update_position");
    assert_eq!(response.attributes[1].key, "stream_id");
    assert_eq!(response.attributes[1].value, "1");
    assert_eq!(response.attributes[2].key, "in_balance");
    assert_eq!(response.attributes[2].value, "1000000");
    assert_eq!(response.attributes[3].key, "shares");
    assert_eq!(response.attributes[3].value, "1000000");
    assert_eq!(response.attributes[4].key, "index");
    assert_eq!(response.attributes[4].value, "0");
    assert_eq!(response.attributes[5].key, "last_updated");
    assert_eq!(response.attributes[5].value, "2000.000000000"); // start_time
    assert_eq!(response.attributes[6].key, "pending_purchase");
    assert_eq!(response.attributes[6].value, "0");
    assert_eq!(response.attributes[7].key, "purchased");
    assert_eq!(response.attributes[7].value, "0");
    assert_eq!(response.attributes[8].key, "spent");
    assert_eq!(response.attributes[8].value, "0");

    // Query stream before withdraw
    let query_env = helpers::env_at(400);
    let stream = cw_streamswap::contract::query_stream(deps.as_ref(), query_env, 1).unwrap();
    assert_eq!(stream.id, 1);
    assert_eq!(stream.dist_index, Decimal256::zero());
    assert_eq!(stream.last_updated, Timestamp::from_seconds(2000));
    assert_eq!(stream.in_supply, Uint256::from(1_000_000u128));
    assert_eq!(stream.spent_in, Uint256::zero());
    assert_eq!(stream.shares, Uint256::from(1_000_000u128));

    // Withdraw before start time (pending withdraw)
    let env = helpers::env_at(2000 - 1);
    let info = helpers::mock_info("subscriber1", &[]);
    let msg = cw_streamswap::msg::ExecuteMsg::Withdraw {
        stream_id: 1,
        cap: Some(Uint256::from(500_000u128)),
        operator_target: None,
    };
    let res = execute(deps.as_mut(), env, info.clone(), msg);
    assert!(res.is_ok());

    let response = res.unwrap();
    assert_eq!(response.attributes[0].key, "action");
    assert_eq!(response.attributes[0].value, "withdraw_pending");
    assert_eq!(response.attributes[1].key, "stream_id");
    assert_eq!(response.attributes[1].value, "1");
    assert_eq!(response.attributes[3].key, "withdraw_amount");
    assert_eq!(response.attributes[3].value, "500000");

    // Check bank message for withdrawal
    assert_eq!(response.messages.len(), 1);
    let bank_msg = &response.messages[0].msg;
    assert_eq!(
        bank_msg,
        &CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin::new(500000u128, "in")],
        })
    );

    // Query stream after withdraw
    let query_env = helpers::env_at(2000 - 1);
    let stream = cw_streamswap::contract::query_stream(deps.as_ref(), query_env, 1).unwrap();
    assert_eq!(stream.id, 1);
    assert_eq!(stream.dist_index, Decimal256::zero());
    assert_eq!(stream.last_updated, Timestamp::from_seconds(2000));
    assert_eq!(stream.in_supply, Uint256::from(500_000u128)); // 1M - 500K
    assert_eq!(stream.spent_in, Uint256::zero());
    assert_eq!(stream.shares, Uint256::from(500_000u128)); // 1M - 500K
}
#[test]
fn withdraw_after_start_time() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Create a stream that hasn't started yet
    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(2000)) // Start in the future
        .end_time(Timestamp::from_seconds(1_000_000));
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

    // Subscribe before start time
    let env = helpers::env_at(2000 - 1);
    let info = helpers::mock_info("subscriber1", &[Coin::new(1_000_000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    let res = execute(deps.as_mut(), env, info, msg);
    assert!(res.is_ok());

    // Withdraw after start time (active withdraw)
    let env = helpers::env_at(3000); // After start time (2000)
    let info = helpers::mock_info("subscriber1", &[]);
    let msg = cw_streamswap::msg::ExecuteMsg::Withdraw {
        stream_id: 1,
        cap: Some(Uint256::from(400_000u128)),
        operator_target: None,
    };
    let res = execute(deps.as_mut(), env, info.clone(), msg);
    assert!(res.is_ok());

    let response = res.unwrap();
    assert_eq!(response.attributes[0].key, "action");
    assert_eq!(response.attributes[0].value, "withdraw");
    assert_eq!(response.attributes[1].key, "stream_id");
    assert_eq!(response.attributes[1].value, "1");
    assert_eq!(response.attributes[3].key, "withdraw_amount");
    assert_eq!(response.attributes[3].value, "400000");

    // Check bank message for withdrawal
    assert_eq!(response.messages.len(), 1);
    let bank_msg = &response.messages[0].msg;
    assert_eq!(
        bank_msg,
        &CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin::new(400000u128, "in")],
        })
    );

    // Query stream after withdraw
    let query_env = helpers::env_at(3000);
    let stream = cw_streamswap::contract::query_stream(deps.as_ref(), query_env, 1).unwrap();
    // Stream should still exist and be active
    assert_eq!(stream.id, 1);
}
