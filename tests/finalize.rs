use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::{BankMsg, Coin, CosmosMsg, Timestamp, Uint256};
use cw_streamswap::contract::{execute, execute_finalize_stream};
use cw_streamswap::msg::ExecuteMsg;
use cw_streamswap::ContractError;

mod helpers;

#[test]
fn finalize_unauthorized_access() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors
    let treasury = helpers::mock_info("creator", &[]);
    let subscriber1 = helpers::mock_info("creator1", &[]);
    let unauthorized = helpers::mock_info("random", &[]);

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
    let env = helpers::env_at(1_000_000 + 1_000_000);
    let funds = Coin::new(2_000_000_000_000u128, "in");
    let mut subscriber1_funded = subscriber1.clone();
    subscriber1_funded.funds = vec![funds];
    let msg = ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    let _res = execute(deps.as_mut(), env, subscriber1_funded, msg).unwrap();

    // Random user attempts to finalize (should fail with Unauthorized)
    let env = helpers::env_at(5_000_000 + 1); // After stream ends
    let res = execute_finalize_stream(deps.as_mut(), env, unauthorized.clone(), 1, None);
    assert_eq!(res.unwrap_err(), ContractError::Unauthorized {});
}

#[test]
fn finalize_timing_validation() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors
    let treasury = helpers::mock_info("creator", &[]);
    let subscriber1 = helpers::mock_info("creator1", &[]);

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
    let env = helpers::env_at(1_000_000 + 1_000_000);
    let funds = Coin::new(2_000_000_000_000u128, "in");
    let mut subscriber1_funded = subscriber1.clone();
    subscriber1_funded.funds = vec![funds];
    let msg = ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    let _res = execute(deps.as_mut(), env, subscriber1_funded, msg).unwrap();

    // Test 1: Can't finalize before stream ends
    let env = helpers::env_at(1_000_000 + 1); // During stream
    let res = execute_finalize_stream(deps.as_mut(), env, treasury.clone(), 1, None);
    assert_eq!(res.unwrap_err(), ContractError::StreamNotEnded {});

    // Test 2: Can finalize after stream ends
    let env = helpers::env_at(5_000_000 + 1); // After stream ends
    let res = execute_finalize_stream(deps.as_mut(), env, treasury.clone(), 1, None);
    assert!(res.is_ok()); // Should succeed
}

#[test]
fn finalize_happy_path() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors
    let treasury = helpers::mock_info("creator", &[]);
    let subscriber1 = helpers::mock_info("creator1", &[]);
    let collector = helpers::mock_info("collector", &[]);

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
    let env = helpers::env_at(1_000_000 + 1_000_000);
    let funds = Coin::new(2_000_000_000_000u128, "in");
    let mut subscriber1_funded = subscriber1.clone();
    subscriber1_funded.funds = vec![funds];
    let msg = ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    let _res = execute(deps.as_mut(), env, subscriber1_funded, msg).unwrap();

    // Update stream before finalization
    let env = helpers::env_at(5_000_000 + 1);
    helpers::mock_info("creator", &[]); // Creator is the treasury
                                        // Note: execute_update_stream function call would go here if available

    // Happy path finalization
    let res = execute_finalize_stream(deps.as_mut(), env, treasury.clone(), 1, None).unwrap();

    // Verify finalization attributes
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "finalize_stream");
    assert_eq!(res.attributes[1].key, "stream_id");
    assert_eq!(res.attributes[1].value, "1");
    assert_eq!(res.attributes[2].key, "stream_dist_index");
    assert_eq!(res.attributes[2].value, "0.0000005");
    assert_eq!(res.attributes[3].key, "stream_shares");
    assert_eq!(res.attributes[3].value, "2000000000000");
    assert_eq!(res.attributes[4].key, "stream_last_updated");
    assert_eq!(res.attributes[4].value, "5000001.000000000");
    assert_eq!(res.attributes[5].key, "stream_in_supply");
    assert_eq!(res.attributes[5].value, "0");
    assert_eq!(res.attributes[6].key, "stream_out_remaining");
    assert_eq!(res.attributes[6].value, "0");
    assert_eq!(res.attributes[7].key, "stream_status");
    assert_eq!(res.attributes[7].value, "Finalized");
    assert_eq!(res.attributes[8].key, "stream_exit_fee_percent");
    assert_eq!(res.attributes[8].value, "0.01");
    assert_eq!(res.attributes[9].key, "treasury");
    assert_eq!(res.attributes[9].value, treasury.sender.to_string());
    assert_eq!(res.attributes[10].key, "creators_revenue");
    assert_eq!(res.attributes[10].value, "1980000000000");
    assert_eq!(res.attributes[11].key, "refunded_out_remaining");
    assert_eq!(res.attributes[11].value, "0");
    assert_eq!(res.attributes[12].key, "total_sold");
    assert_eq!(res.attributes[12].value, "1000000");
    assert_eq!(res.attributes[13].key, "fee_collector");
    assert_eq!(res.attributes[13].value, collector.sender.to_string());
    assert_eq!(res.attributes[14].key, "swap_fee");
    assert_eq!(res.attributes[14].value, "20000000000");
    assert_eq!(res.attributes[15].key, "creation_fee");
    assert_eq!(res.attributes[15].value, "100");

    // Verify bank messages
    assert_eq!(res.messages.len(), 3);

    // Message 1: Treasury payment
    let msg1 = &res.messages[0].msg;
    assert_eq!(
        msg1,
        &CosmosMsg::Bank(BankMsg::Send {
            to_address: treasury.sender.to_string(),
            amount: vec![Coin::new(1_980_000_000_000u128, "in")],
        })
    );

    // Message 2: Creation fee to collector
    let msg2 = &res.messages[1].msg;
    assert_eq!(
        msg2,
        &CosmosMsg::Bank(BankMsg::Send {
            to_address: collector.sender.to_string(),
            amount: vec![Coin::new(100u128, "fee")],
        })
    );

    // Message 3: Swap fee to collector
    let msg3 = &res.messages[2].msg;
    assert_eq!(
        msg3,
        &CosmosMsg::Bank(BankMsg::Send {
            to_address: collector.sender.to_string(),
            amount: vec![Coin::new(20_000_000_000u128, "in")],
        })
    );
}

#[test]
fn finalize_duplicate_prevention() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors
    let treasury = helpers::mock_info("creator", &[]);
    let subscriber1 = helpers::mock_info("creator1", &[]);

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
    let env = helpers::env_at(1_000_000 + 1_000_000); // After start time
    let funds = Coin::new(2_000_000_000_000u128, "in");
    let mut subscriber1_funded = subscriber1.clone();
    subscriber1_funded.funds = vec![funds];
    let msg = ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    let _res = execute(deps.as_mut(), env, subscriber1_funded, msg).unwrap();

    // First finalization (should succeed)
    let env = helpers::env_at(5_000_000 + 1); // After stream ends
    let res =
        execute_finalize_stream(deps.as_mut(), env.clone(), treasury.clone(), 1, None).unwrap();

    // Verify first finalization succeeded by checking response attributes
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "finalize_stream");

    // Second finalization (should fail with StreamAlreadyFinalized)
    let res = execute_finalize_stream(deps.as_mut(), env, treasury, 1, None);
    assert_eq!(res.unwrap_err(), ContractError::StreamAlreadyFinalized {});
}
