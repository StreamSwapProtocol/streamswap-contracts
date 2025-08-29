use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::{BankMsg, Coin, Decimal256, Uint256};
use cosmwasm_std::{CosmosMsg, Timestamp};
use cw_streamswap::contract::execute;
use std::str::FromStr;
mod helpers;

#[test]
fn withdraw_pending_basic_flow() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors (MessageInfo with empty funds)
    let treasury = helpers::mock_info("creator", &[]);
    let subscriber1 = helpers::mock_info("subscriber1", &[]);
    let _unauthorized = helpers::mock_info("unauthorized", &[]);

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
    let mut treasury_funded = treasury.clone();
    treasury_funded.funds = funds;
    execute(deps.as_mut(), env, treasury_funded, b.build()).unwrap();

    // First subscribe before start time (pending)
    let env = helpers::env_at(2000 - 1); // Before start time
    let mut subscriber1_funded = subscriber1.clone();
    subscriber1_funded.funds = vec![Coin::new(1_000_000u128, "in")];
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    let res = execute(deps.as_mut(), env, subscriber1_funded, msg);
    assert!(res.is_ok());

    // Update subscriber1 position - no distribution expected
    let env = helpers::env_at(2000 - 1);
    let update_msg = cw_streamswap::msg::ExecuteMsg::UpdatePosition {
        stream_id: 1,
        operator_target: None,
    };
    let update_info = subscriber1.clone();
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
    let info = subscriber1.clone();
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
            to_address: subscriber1.sender.to_string(),
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

    // Define actors (MessageInfo with empty funds)
    let treasury = helpers::mock_info("creator", &[]);
    let subscriber1 = helpers::mock_info("subscriber1", &[]);
    let _unauthorized = helpers::mock_info("unauthorized", &[]);

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
    let mut treasury_funded = treasury.clone();
    treasury_funded.funds = funds;
    execute(deps.as_mut(), env, treasury_funded, b.build()).unwrap();

    // Subscribe before start time
    let env = helpers::env_at(2000 - 1);
    let mut subscriber1_funded = subscriber1.clone();
    subscriber1_funded.funds = vec![Coin::new(1_000_000u128, "in")];
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    let res = execute(deps.as_mut(), env, subscriber1_funded, msg);
    assert!(res.is_ok());

    // Withdraw after start time (active withdraw)
    let env = helpers::env_at(3000); // After start time (2000)
    let info = subscriber1.clone();
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

#[test]
fn withdraw_invalid_amounts() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define all actors upfront
    let treasury = helpers::mock_info("creator", &[]);
    let subscriber1 = helpers::mock_info("subscriber1", &[]);
    let _unauthorized = helpers::mock_info("unauthorized", &[]);

    // Create a stream
    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(1_000_000))
        .end_time(Timestamp::from_seconds(5_000_000));
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
    treasury_funded.funds = funds.clone();
    execute(deps.as_mut(), env, treasury_funded, b.build()).unwrap();

    // Subscribe to the stream
    let env = helpers::env_at(1_000_000);
    let funds = Coin::new(2_000_000_000_000u128, "in");
    let mut subscriber1_funded = subscriber1.clone();
    subscriber1_funded.funds = vec![funds];
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    let _res = execute(deps.as_mut(), env, subscriber1_funded, msg).unwrap();

    // Test 1: Withdraw with cap = 0 (should fail)
    let env = helpers::env_at(1_000_000 + 5000);
    let info = subscriber1.clone();
    let cap = Uint256::zero();
    let msg = cw_streamswap::msg::ExecuteMsg::Withdraw {
        stream_id: 1,
        cap: Some(cap),
        operator_target: None,
    };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
    assert_eq!(
        res.unwrap_err(),
        cw_streamswap::ContractError::InvalidWithdrawAmount {}
    );

    // Test 2: Withdraw with cap > available balance (should fail)
    let cap = Uint256::from(2_250_000_000_000u128);
    let msg = cw_streamswap::msg::ExecuteMsg::Withdraw {
        stream_id: 1,
        cap: Some(cap),
        operator_target: None,
    };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
    assert_eq!(
        res.unwrap_err(),
        cw_streamswap::ContractError::WithdrawAmountExceedsBalance(Uint256::from(
            2250000000000u128
        ))
    );
}

#[test]
fn withdraw_with_valid_cap() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define all actors upfront
    let treasury = helpers::mock_info("creator", &[]);
    let subscriber1 = helpers::mock_info("subscriber1", &[]);
    let _unauthorized = helpers::mock_info("unauthorized", &[]);

    // Create a stream
    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(1_000_000))
        .end_time(Timestamp::from_seconds(5_000_000));
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

    // Subscribe to the stream
    let env = helpers::env_at(1_000_000);
    let funds = Coin::new(2_000_000_000_000u128, "in");
    let mut subscriber1_funded = subscriber1.clone();
    subscriber1_funded.funds = vec![funds.clone()];
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    let _res = execute(deps.as_mut(), env, subscriber1_funded, msg).unwrap();

    // Withdraw with valid cap
    let env = helpers::env_at(1_000_000 + 5000);
    let info = subscriber1.clone();
    let cap = Uint256::from(25_000_000u128);
    let msg = cw_streamswap::msg::ExecuteMsg::Withdraw {
        stream_id: 1,
        cap: Some(cap),
        operator_target: None,
    };
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();

    // Verify position state after withdrawal
    let position = cw_streamswap::contract::query_position(
        deps.as_ref(),
        helpers::env_at(1_000_000 + 5000),
        1,
        subscriber1.sender.to_string(),
    )
    .unwrap();

    // Verify position state after withdrawal (exact values from original test)
    assert_eq!(position.in_balance, Uint256::from(1_997_475_000_000u128));
    assert_eq!(position.spent, Uint256::from(2_500_000_000u128));
    assert_eq!(position.purchased, Uint256::from(1250u128));

    // Verify total funds consistency: in_balance + spent + cap = original_funds
    let total_after_withdrawal = position.in_balance + position.spent + cap;
    let original_funds = Uint256::from_str(funds.amount.to_string().as_str()).unwrap();
    assert_eq!(total_after_withdrawal, original_funds);
}

#[test]
fn withdraw_full_balance() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define all actors upfront
    let treasury = helpers::mock_info("creator", &[]);
    let subscriber = helpers::mock_info("subscriber1", &[]);
    let _unauthorized = helpers::mock_info("unauthorized", &[]);

    // Create a stream
    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(1_000_000))
        .end_time(Timestamp::from_seconds(5_000_000));
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
    let info = cosmwasm_std::MessageInfo {
        sender: treasury.sender.clone(),
        funds,
    };
    execute(deps.as_mut(), env, info, b.build()).unwrap();

    // Subscribe to the stream
    let env = helpers::env_at(1_000_000);
    let funds = Coin::new(2_000_000_000_000u128, "in");
    let info = cosmwasm_std::MessageInfo {
        sender: subscriber.sender.clone(),
        funds: vec![funds.clone()],
    };
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();

    // Withdraw full balance (cap = None)
    let env = helpers::env_at(1_000_000 + 1_000_000);
    let info = cosmwasm_std::MessageInfo {
        sender: subscriber.sender.clone(),
        funds: vec![],
    };
    let msg = cw_streamswap::msg::ExecuteMsg::Withdraw {
        stream_id: 1,
        cap: None,
        operator_target: None,
    };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();

    // Verify final position state
    let position = cw_streamswap::contract::query_position(
        deps.as_ref(),
        helpers::env_at(1_000_000 + 1_000_000),
        1,
        subscriber.sender.to_string(),
    )
    .unwrap();

    assert_eq!(position.in_balance, Uint256::zero());
    assert_eq!(position.spent, Uint256::from(500000000000u128));
    assert_eq!(position.purchased, Uint256::from(250000u128));
    assert_eq!(position.shares, Uint256::zero());

    // Verify bank message for full withdrawal
    let msg = res.messages.first().unwrap();
    assert_eq!(
        msg.msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: subscriber.sender.to_string(),
            amount: vec![Coin::new(1500000000000u128, "in")]
        })
    );
}

#[test]
fn withdraw_after_stream_ends() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define all actors upfront
    let treasury = helpers::mock_info("creator", &[]);
    let subscriber = helpers::mock_info("subscriber1", &[]);
    let _unauthorized = helpers::mock_info("unauthorized", &[]);

    // Create a stream
    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(1_000_000))
        .end_time(Timestamp::from_seconds(5_000_000));
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
    let info = cosmwasm_std::MessageInfo {
        sender: treasury.sender.clone(),
        funds,
    };
    execute(deps.as_mut(), env, info, b.build()).unwrap();

    // Subscribe to the stream
    let env = helpers::env_at(1_000_000);
    let funds = Coin::new(2_000_000_000_000u128, "in");
    let info = cosmwasm_std::MessageInfo {
        sender: subscriber.sender.clone(),
        funds: vec![funds.clone()],
    };
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();

    // Attempt withdrawal after stream ends (should fail)
    let env = helpers::env_at(5_000_000 + 1); // After end time
    let info = cosmwasm_std::MessageInfo {
        sender: subscriber.sender.clone(),
        funds: vec![],
    };
    let msg = cw_streamswap::msg::ExecuteMsg::Withdraw {
        stream_id: 1,
        cap: None,
        operator_target: None,
    };
    let res = execute(deps.as_mut(), env, info, msg);
    assert_eq!(
        res.unwrap_err(),
        cw_streamswap::ContractError::StreamEnded {}
    );
}
