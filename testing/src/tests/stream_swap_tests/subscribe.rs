use std::str::FromStr;

use crate::helpers::utils::get_contract_address_from_res;
#[cfg(test)]
use crate::helpers::{
    mock_messages::{get_create_stream_msg, get_factory_inst_msg},
    setup::{setup, SetupResponse},
};
use cosmwasm_std::{coin, Addr, BlockInfo, Decimal256, Uint128};
use cw_multi_test::Executor;
use cw_streamswap::{
    msg::{
        ExecuteMsg as StreamSwapExecuteMsg, PositionResponse, QueryMsg as StreamSwapQueryMsg,
        StreamResponse,
    },
    state::Stream,
    threshold::ThresholdError,
    ContractError as StreamSwapError,
};
use cw_streamswap_factory::{
    error::ContractError as FactoryError, msg::QueryMsg as FactoryQueryMsg,
    payment_checker::CustomPaymentError,
};
use cw_utils::PaymentError;
#[test]
fn test_first_subcription() {
    let SetupResponse {
        mut app,
        test_accounts,
        stream_swap_code_id,
        stream_swap_factory_code_id,
    } = setup();
    let start_time = app.block_info().time.plus_seconds(100).into();
    let end_time = app.block_info().time.plus_seconds(200).into();

    let msg = get_factory_inst_msg(stream_swap_code_id, &test_accounts);
    let factory_address = app
        .instantiate_contract(
            stream_swap_factory_code_id,
            test_accounts.admin.clone(),
            &msg,
            &[],
            "Factory".to_string(),
            None,
        )
        .unwrap();

    let create_stream_msg = get_create_stream_msg(
        &"Stream Swap tests".to_string(),
        None,
        &test_accounts.creator.to_string(),
        coin(1_000_000, "out_denom"),
        "in_denom",
        start_time,
        end_time,
        None,
    );

    let res = app
        .execute_contract(
            test_accounts.creator.clone(),
            factory_address.clone(),
            &create_stream_msg,
            &[coin(100, "fee_token"), coin(1_000_000, "out_denom")],
        )
        .unwrap();
    let stream_swap_contract_address: String = get_contract_address_from_res(res);

    let stream_id: u64 = app
        .wrap()
        .query_wasm_smart(factory_address.clone(), &FactoryQueryMsg::LastStreamId {})
        .unwrap();
    assert_eq!(stream_id, 1);

    let subscribe_msg = StreamSwapExecuteMsg::Subscribe {
        operator_target: None,
        operator: None,
    };
    app.set_block(BlockInfo {
        height: 1_100,
        time: start_time,
        chain_id: "test".to_string(),
    });

    // No funds
    let res = app
        .execute_contract(
            test_accounts.subscriber.clone(),
            Addr::unchecked(stream_swap_contract_address.clone()),
            &subscribe_msg,
            &[],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<StreamSwapError>().unwrap();
    assert_eq!(error, &StreamSwapError::Payment(PaymentError::NoFunds {}));
    // Incorrect denom
    let res = app
        .execute_contract(
            test_accounts.subscriber.clone(),
            Addr::unchecked(stream_swap_contract_address.clone()),
            &subscribe_msg,
            &[coin(100, "wrong_denom")],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<StreamSwapError>().unwrap();
    assert_eq!(
        error,
        &StreamSwapError::Payment(PaymentError::MissingDenom("in_denom".to_string()))
    );

    // Subscribe
    let _res = app
        .execute_contract(
            test_accounts.subscriber.clone(),
            Addr::unchecked(stream_swap_contract_address.clone()),
            &subscribe_msg,
            &[coin(150, "in_denom")],
        )
        .unwrap();

    // Query Stream
    let stream: StreamResponse = app
        .wrap()
        .query_wasm_smart(
            Addr::unchecked(stream_swap_contract_address.clone()),
            &StreamSwapQueryMsg::Stream {},
        )
        .unwrap();
    // First subscription should set the stream to active
    assert_eq!(stream.status, cw_streamswap::state::Status::Active);
    // Dist index should be zero because no distribution has been made until last update
    assert_eq!(stream.dist_index, Decimal256::zero());
    // In supply should be updated
    assert_eq!(stream.in_supply, Uint128::new(150));
    // Position should be updated
    let position: PositionResponse = app
        .wrap()
        .query_wasm_smart(
            Addr::unchecked(stream_swap_contract_address.clone()),
            &StreamSwapQueryMsg::Position {
                owner: test_accounts.subscriber.to_string(),
            },
        )
        .unwrap();
    assert_eq!(position.index, Decimal256::zero());
    assert_eq!(position.in_balance, Uint128::new(150));
    assert_eq!(position.spent, Uint128::zero());

    // Update stream
    app.set_block(BlockInfo {
        height: 2_200,
        time: start_time.plus_seconds(20),
        chain_id: "test".to_string(),
    });
    let _res = app
        .execute_contract(
            test_accounts.subscriber.clone(),
            Addr::unchecked(stream_swap_contract_address.clone()),
            &StreamSwapExecuteMsg::UpdateStream {},
            &[],
        )
        .unwrap();
    // Dist index should be updated
    let stream: StreamResponse = app
        .wrap()
        .query_wasm_smart(
            Addr::unchecked(stream_swap_contract_address.clone()),
            &StreamSwapQueryMsg::Stream {},
        )
        .unwrap();
    assert_eq!(
        stream.dist_index,
        Decimal256::from_str("1333.333333333333333333").unwrap()
    );
    assert_eq!(stream.in_supply, Uint128::new(120));
    assert_eq!(stream.spent_in, Uint128::new(30));
    assert_eq!(stream.last_updated, start_time.plus_seconds(20));
    assert_eq!(stream.shares, Uint128::new(150));
}
#[test]
fn test_recurring_subscribe() {
    let SetupResponse {
        mut app,
        test_accounts,
        stream_swap_code_id,
        stream_swap_factory_code_id,
    } = setup();
    let start_time = app.block_info().time.plus_seconds(100).into();
    let end_time = app.block_info().time.plus_seconds(200).into();

    let msg = get_factory_inst_msg(stream_swap_code_id, &test_accounts);
    let factory_address = app
        .instantiate_contract(
            stream_swap_factory_code_id,
            test_accounts.admin.clone(),
            &msg,
            &[],
            "Factory".to_string(),
            None,
        )
        .unwrap();

    let create_stream_msg = get_create_stream_msg(
        &"Stream Swap tests".to_string(),
        None,
        &test_accounts.creator.to_string(),
        coin(1_000_000, "out_denom"),
        "in_denom",
        start_time,
        end_time,
        None,
    );

    let res = app
        .execute_contract(
            test_accounts.creator.clone(),
            factory_address.clone(),
            &create_stream_msg,
            &[coin(100, "fee_token"), coin(1_000_000, "out_denom")],
        )
        .unwrap();
    let stream_swap_contract_address: String = get_contract_address_from_res(res);

    let subscribe_msg = StreamSwapExecuteMsg::Subscribe {
        operator_target: None,
        operator: None,
    };
    app.set_block(BlockInfo {
        height: 1_100,
        time: start_time,
        chain_id: "test".to_string(),
    });

    // First subscription
    let _res = app
        .execute_contract(
            test_accounts.subscriber.clone(),
            Addr::unchecked(stream_swap_contract_address.clone()),
            &subscribe_msg,
            &[coin(150, "in_denom")],
        )
        .unwrap();

    // Query Stream
    let stream: StreamResponse = app
        .wrap()
        .query_wasm_smart(
            Addr::unchecked(stream_swap_contract_address.clone()),
            &StreamSwapQueryMsg::Stream {},
        )
        .unwrap();
    assert_eq!(stream.status, cw_streamswap::state::Status::Active);
    assert_eq!(stream.dist_index, Decimal256::zero());
    assert_eq!(stream.in_supply, Uint128::new(150));
    let position: PositionResponse = app
        .wrap()
        .query_wasm_smart(
            Addr::unchecked(stream_swap_contract_address.clone()),
            &StreamSwapQueryMsg::Position {
                owner: test_accounts.subscriber.to_string(),
            },
        )
        .unwrap();
    assert_eq!(position.index, Decimal256::zero());
    assert_eq!(position.in_balance, Uint128::new(150));
    assert_eq!(position.spent, Uint128::zero());

    // Non-operator tries to increase subscription
    let res = app
        .execute_contract(
            test_accounts.wrong_user.clone(),
            Addr::unchecked(stream_swap_contract_address.clone()),
            &StreamSwapExecuteMsg::Subscribe {
                operator_target: Some(test_accounts.subscriber.to_string()),
                operator: None,
            },
            &[coin(150, "in_denom")],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<StreamSwapError>().unwrap();
    assert_eq!(error, &StreamSwapError::Unauthorized {});

    // Subscriber increases subscription in same block_time
    let _res = app
        .execute_contract(
            test_accounts.subscriber.clone(),
            Addr::unchecked(stream_swap_contract_address.clone()),
            &StreamSwapExecuteMsg::Subscribe {
                operator_target: None,
                operator: None,
            },
            &[coin(150, "in_denom")],
        )
        .unwrap();

    // Query Stream
    let stream: StreamResponse = app
        .wrap()
        .query_wasm_smart(
            Addr::unchecked(stream_swap_contract_address.clone()),
            &StreamSwapQueryMsg::Stream {},
        )
        .unwrap();
    // There will be no distribution because the last update was in the same block
    assert_eq!(stream.dist_index, Decimal256::from_str("0").unwrap());

    let position: PositionResponse = app
        .wrap()
        .query_wasm_smart(
            Addr::unchecked(stream_swap_contract_address.clone()),
            &StreamSwapQueryMsg::Position {
                owner: test_accounts.subscriber.to_string(),
            },
        )
        .unwrap();
    assert_eq!(position.index, Decimal256::from_str("0").unwrap());
    assert_eq!(position.in_balance, Uint128::new(300));

    // Now simulate a block update
    app.set_block(BlockInfo {
        height: 1_200,
        time: start_time.plus_seconds(1),
        chain_id: "test".to_string(),
    });

    // Subscriber increases subscription
    let _res = app
        .execute_contract(
            test_accounts.subscriber.clone(),
            Addr::unchecked(stream_swap_contract_address.clone()),
            &StreamSwapExecuteMsg::Subscribe {
                operator_target: None,
                operator: None,
            },
            &[coin(150, "in_denom")],
        )
        .unwrap();

    // Query Stream
    let stream: StreamResponse = app
        .wrap()
        .query_wasm_smart(
            Addr::unchecked(stream_swap_contract_address.clone()),
            &StreamSwapQueryMsg::Stream {},
        )
        .unwrap();
    // Distribution index should be updated
    assert_eq!(
        stream.dist_index,
        Decimal256::from_str("33.333333333333333333").unwrap()
    );
    assert_eq!(stream.in_supply, Uint128::new(447));
    let position: PositionResponse = app
        .wrap()
        .query_wasm_smart(
            Addr::unchecked(stream_swap_contract_address.clone()),
            &StreamSwapQueryMsg::Position {
                owner: test_accounts.subscriber.to_string(),
            },
        )
        .unwrap();

    assert_eq!(
        position.index,
        Decimal256::from_str("33.333333333333333333").unwrap()
    );
    assert_eq!(position.in_balance, Uint128::new(447));
    assert_eq!(position.spent, Uint128::from(3u128));
}
//     #[test]
//     fn test_subscribe_pending() {
//         // instantiate
//         let treasury = Addr::unchecked("treasury");
//         let start = 5000;
//         let end = 10000;
//         let out_supply = Uint128::new(1_000_000);
//         let out_denom = "out_denom";

//         // instantiate
//         let mut deps = mock_dependencies();
//         let mut env = mock_env();
//         env.block.height = 100;
//         let msg = crate::msg::InstantiateMsg {
//             min_stream_blocks: 500,
//             min_blocks_until_start_block: 500,
//             stream_creation_denom: "fee".to_string(),
//             stream_creation_fee: Uint128::new(100),
//             exit_fee_percent: Decimal::percent(1),
//             fee_collector: "collector".to_string(),
//             protocol_admin: "protocol_admin".to_string(),
//             accepted_in_denom: "in".to_string(),
//         };
//         instantiate(deps.as_mut(), mock_env(), mock_info("creator", &[]), msg).unwrap();

//         // create stream
//         let mut env = mock_env();
//         env.block.height = 200;
//         let info = mock_info(
//             "creator1",
//             &[
//                 Coin::new(out_supply.u128(), out_denom),
//                 Coin::new(100, "fee"),
//             ],
//         );
//         execute_create_stream(
//             deps.as_mut(),
//             env,
//             info,
//             treasury.to_string(),
//             "test".to_string(),
//             Some("https://sample.url".to_string()),
//             "in".to_string(),
//             out_denom.to_string(),
//             out_supply,
//             start,
//             end,
//             None,
//         )
//         .unwrap();

//         // first subscribe
//         let mut env = mock_env();
//         env.block.height = 300;

//         let info = mock_info("creator1", &[Coin::new(1_000_000, "in")]);
//         let msg = crate::msg::ExecuteMsg::Subscribe {
//             stream_id: 1,
//             operator_target: None,
//             operator: None,
//         };
//         let res = execute(deps.as_mut(), env, info, msg).unwrap();
//         assert_eq!(res.attributes[0].key, "action");
//         assert_eq!(res.attributes[0].value, "subscribe_pending");
//         // query stream
//         let mut env = mock_env();
//         env.block.height = 350;
//         let stream = query_stream(deps.as_ref(), env, 1).unwrap();
//         assert_eq!(stream.status, Status::Waiting);
//         assert_eq!(stream.in_supply, Uint128::new(1000000));
//         assert_eq!(stream.shares, Uint128::new(1000000));

//         // second subscribe still waiting
//         let mut env = mock_env();
//         env.block.height = 500;
//         let info = mock_info("creator1", &[Coin::new(1_000_000, "in")]);
//         let msg = crate::msg::ExecuteMsg::Subscribe {
//             stream_id: 1,
//             operator_target: None,
//             operator: None,
//         };
//         let res = execute(deps.as_mut(), env, info, msg).unwrap();
//         assert_eq!(res.attributes[0].key, "action");
//         assert_eq!(res.attributes[0].value, "subscribe_pending");

//         // query stream
//         let mut env = mock_env();
//         env.block.height = 450;
//         let stream = query_stream(deps.as_ref(), env, 1).unwrap();
//         assert_eq!(stream.status, Status::Waiting);
//         assert_eq!(stream.in_supply, Uint128::new(2000000));

//         // Before stream start height, 2 subscriptions have been made and the stream is pending
//         // After stream start height plus 1000 blocks, one subscription is made and the stream is active
//         // Creator 1 has 2 subscriptions and 2_000_000 in balance
//         // Creator 2 has 1 subscription and 1_000_000 in balance
//         // At 6000 blocks, the stream is active and the balance to be distributed is ~2000000
//         // At 6000 blocks, creator 1 should have spent 2000000*1000/5000= 400000
//         // At 6000 blocks, creator 1 should get all 2000000 tokens
//         // At 6000 blocks, creator 2 should get 0 tokens
//         // At 7500 blocks, the stream is active and the balance to be distributed is 300000
//         // At 7500 blocks, creator 1 should get 300000*2000000/3250000 = 184615
//         // At 7500 blocks, creator 2 should get 300000*1250000/3250000 = 115384

//         // subscription after start height
//         let mut env = mock_env();
//         env.block.height = 6000;
//         let info = mock_info("creator2", &[Coin::new(1_000_000, "in")]);
//         let msg = crate::msg::ExecuteMsg::Subscribe {
//             stream_id: 1,
//             operator_target: None,
//             operator: None,
//         };
//         let res = execute(deps.as_mut(), env, info, msg).unwrap();
//         assert_eq!(res.attributes[0].key, "action");
//         // different action because the stream is active
//         assert_eq!(res.attributes[0].value, "subscribe");

//         // update creator 1 position
//         let mut env = mock_env();
//         env.block.height = 6000;
//         let update_msg = crate::msg::ExecuteMsg::UpdatePosition {
//             stream_id: 1,
//             operator_target: None,
//         };
//         let info = mock_info("creator1", &[]);
//         let _res = execute(deps.as_mut(), env.clone(), info, update_msg).unwrap();
//         let position = query_position(deps.as_ref(), env, 1, "creator1".to_string()).unwrap();
//         assert_eq!(position.spent, Uint128::new(400000));

//         // query stream
//         let mut env = mock_env();
//         env.block.height = 6000;
//         let stream = query_stream(deps.as_ref(), env, 1).unwrap();
//         assert_eq!(stream.status, Status::Active);
//         assert_eq!(stream.in_supply, Uint128::new(3000000 - 400000));
//         assert_eq!(stream.spent_in, Uint128::new(400000));

//         // update creator 1 position at 7500
//         let mut env = mock_env();
//         env.block.height = 7500;
//         let update_msg = crate::msg::ExecuteMsg::UpdatePosition {
//             stream_id: 1,
//             operator_target: None,
//         };
//         let info = mock_info("creator1", &[]);
//         let _res = execute(deps.as_mut(), env.clone(), info, update_msg).unwrap();

//         // query position
//         let res = query_position(deps.as_ref(), env, 1, "creator1".to_string()).unwrap();
//         assert_eq!(res.purchased, Uint128::new(184615 + 200000));
//         assert_eq!(res.spent, Uint128::new(2000000 / 2));

//         // update creator 2 position at 7500
//         let mut env = mock_env();
//         env.block.height = 7500;
//         let update_msg = crate::msg::ExecuteMsg::UpdatePosition {
//             stream_id: 1,
//             operator_target: None,
//         };
//         let info = mock_info("creator2", &[]);
//         let _res = execute(deps.as_mut(), env.clone(), info, update_msg).unwrap();

//         // query position
//         let res = query_position(deps.as_ref(), env, 1, "creator2".to_string()).unwrap();
//         assert_eq!(res.purchased, Uint128::new(115384));
//         // spent = in_supply * (now - last_updated) / (end - last_updated)
//         assert_eq!(res.spent, Uint128::new(1000000 * 1500 / 4000));
//         // query stream
//         let mut env = mock_env();
//         env.block.height = 3500;
//         let stream = query_stream(deps.as_ref(), env, 1).unwrap();
//         assert_eq!(stream.status, Status::Active);
//         // in supply = 3000000 - (positions.spent summed)
//         assert_eq!(stream.in_supply, Uint128::new(1625000));
//     }
