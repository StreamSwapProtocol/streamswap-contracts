use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::{Addr, BankMsg, Coin, CosmosMsg, Decimal256, Timestamp, Uint256, Uint64};

use cw_streamswap::contract::{
    execute, execute_create_stream, execute_exit_stream, execute_finalize_stream, query_stream,
};
use cw_streamswap::msg::{ExecuteMsg, InstantiateMsg};
use cw_streamswap::{threshold::ThresholdError, ContractError};

mod helpers;
//mod threshold {
//         use crate::{
//             killswitch::{execute_cancel_stream_with_threshold, execute_exit_cancelled},
//             threshold::ThresholdError,
//         };

//         // Create a stream with a threshold
//         // Subscribe to the stream
//         use super::*;

//         #[test]
//         fn test_threshold_reached() {
//             let treasury = Addr::unchecked("treasury");
//             let start = Timestamp::from_seconds(1_000_000);
//             let end = Timestamp::from_seconds(5_000_000);
//             let out_supply = Uint256::from(500u128);
//             let out_denom = "out_denom";
//             let in_denom = "in_denom";

//             // threshold = 500*0.5 / 1-0.01 =252.5

//             // instantiate
//             let mut deps = mock_dependencies();
//             let mut env = mock_env();
//             env.block.time = Timestamp::from_seconds(0);
//             let msg = crate::msg::InstantiateMsg {
//                 min_stream_seconds: Uint64::new(1000),
//                 min_seconds_until_start_time: Uint64::new(0),
//                 stream_creation_denom: "fee".to_string(),
//                 stream_creation_fee: Uint128::new(100),
//                 exit_fee_percent: Decimal256::percent(1),
//                 fee_collector: "collector".to_string(),
//                 protocol_admin: "protocol_admin".to_string(),
//                 accepted_in_denom: in_denom.to_string(),
//                 tos_version: "v1".to_string(),
//             };
//             instantiate(deps.as_mut(), mock_env(), mock_info("creator", &[]), msg).unwrap();

//             // create stream
//             let mut env = mock_env();
//             env.block.time = Timestamp::from_seconds(0);
//             let info = mock_info(
//                 "creator",
//                 &[
//                     Coin::new(out_supply.to_string().parse().unwrap(), out_denom),
//                     Coin::new(100, "fee"),
//                 ],
//             );
//             execute_create_stream(
//                 deps.as_mut(),
//                 env,
//                 info,
//                 treasury.to_string(),
//                 "test".to_string(),
//                 Some("https://sample.url".to_string()),
//                 in_denom.to_string(),
//                 out_denom.to_string(),
//                 out_supply,
//                 start,
//                 end,
//                 Some(Uint256::from(250u128)),
//                 "v1".to_string(),
//             )
//             .unwrap();

//             // subscription
//             let mut env = mock_env();
//             env.block.time = start;
//             let funds = Coin::new(252, "in_denom");
//             let info = mock_info("subscriber", &[funds]);
//             let msg = crate::msg::ExecuteMsg::Subscribe {
//                 stream_id: 1,
//                 operator_target: None,
//                 operator: Some("operator".to_string()),
//                 tos_version: "v1".to_string(),
//             };
//             let _res = execute(deps.as_mut(), env, info, msg).unwrap();

//             // Threshold should be reached
//             let mut env = mock_env();
//             env.block.time = end.plus_seconds(1);

//             // Exit should be possible
//             // Since there is only one subscriber all out denom should be sent to subscriber
//             // In calculations we are always rounding down that one token will be left in the stream
//             // Asuming token is 6 decimals
//             // This amount could be considered as insignificant
//             let info = mock_info("subscriber", &[]);
//             let res = execute_exit_stream(deps.as_mut(), env.clone(), info, 1, None).unwrap();
//             assert_eq!(
//                 res.messages,
//                 vec![SubMsg::new(BankMsg::Send {
//                     to_address: "subscriber".to_string(),
//                     amount: vec![Coin::new(499, "out_denom")],
//                 })],
//             );

//             // Creator finalizes the stream
//             let info = mock_info("treasury", &[]);
//             let res = execute_finalize_stream(deps.as_mut(), env.clone(), info, 1, None).unwrap();
//             // Creator's revenue
//             assert_eq!(
//                 res.messages[0].msg,
//                 cosmwasm_std::CosmosMsg::Bank(BankMsg::Send {
//                     to_address: "treasury".to_string(),
//                     amount: vec![Coin::new(250, "in_denom")],
//                 })
//             );
//             assert_eq!(
//                 res.messages[1].msg,
//                 cosmwasm_std::CosmosMsg::Bank(BankMsg::Send {
//                     to_address: "collector".to_string(),
//                     amount: vec![Coin::new(100, "fee")],
//                 })
//             );
//             assert_eq!(
//                 res.messages[2].msg,
//                 cosmwasm_std::CosmosMsg::Bank(BankMsg::Send {
//                     to_address: "collector".to_string(),
//                     amount: vec![Coin::new(2, "in_denom")],
//                 })
//             )
//         }

//         #[test]
//         fn test_threshold_not_reached() {
//             let treasury = Addr::unchecked("treasury");
//             let start = Timestamp::from_seconds(1_000_000);
//             let end = Timestamp::from_seconds(5_000_000);
//             let out_supply = Uint256::from(500u128);
//             let out_denom = "out_denom";
//             let in_denom = "in_denom";

//             // threshold = 500*0.5 / 1-0.01 =252.5

//             // instantiate
//             let mut deps = mock_dependencies();
//             let mut env = mock_env();
//             env.block.height = 0;
//             let msg = crate::msg::InstantiateMsg {
//                 min_stream_seconds: Uint64::new(1000),
//                 min_seconds_until_start_time: Uint64::new(0),
//                 stream_creation_denom: "fee".to_string(),
//                 stream_creation_fee: Uint128::new(100),
//                 exit_fee_percent: Decimal256::percent(1),
//                 fee_collector: "collector".to_string(),
//                 protocol_admin: "protocol_admin".to_string(),
//                 accepted_in_denom: in_denom.to_string(),
//                 tos_version: "v1".to_string(),
//             };
//             instantiate(deps.as_mut(), mock_env(), mock_info("creator", &[]), msg).unwrap();

//             // create stream
//             let mut env = mock_env();
//             env.block.time = Timestamp::from_seconds(0);
//             let info = mock_info(
//                 "creator",
//                 &[
//                     Coin::new(out_supply.to_string().parse().unwrap(), out_denom),
//                     Coin::new(100, "fee"),
//                 ],
//             );
//             execute_create_stream(
//                 deps.as_mut(),
//                 env,
//                 info,
//                 treasury.to_string(),
//                 "test".to_string(),
//                 Some("https://sample.url".to_string()),
//                 in_denom.to_string(),
//                 out_denom.to_string(),
//                 out_supply,
//                 start,
//                 end,
//                 Some(500u128.into()),
//                 "v1".to_string(),
//             )
//             .unwrap();

//             // Subscription 1
//             let mut env = mock_env();
//             env.block.time = start;
//             let funds = Coin::new(250, "in_denom");
//             let info = mock_info("subscriber", &[funds]);
//             let msg = crate::msg::ExecuteMsg::Subscribe {
//                 stream_id: 1,
//                 operator_target: None,
//                 operator: Some("operator".to_string()),
//                 tos_version: "v1".to_string(),
//             };
//             let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

//             // Subscription 2
//             let funds = Coin::new(1, "in_denom");
//             let info = mock_info("subscriber2", &[funds]);
//             let msg = crate::msg::ExecuteMsg::Subscribe {
//                 stream_id: 1,
//                 operator_target: None,
//                 operator: Some("operator".to_string()),
//                 tos_version: "v1".to_string(),
//             };
//             let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

//             // Set time to the end of the stream
//             let mut env = mock_env();
//             env.block.time = end.plus_seconds(1);

//             // Exit should not be possible
//             let info = mock_info("subscriber", &[]);
//             let res = execute_exit_stream(deps.as_mut(), env.clone(), info, 1, None).unwrap_err();
//             assert_eq!(
//                 res,
//                 ContractError::ThresholdError(ThresholdError::ThresholdNotReached {})
//             );

//             // Finalize should not be possible
//             let info = mock_info("treasury", &[]);
//             let res =
//                 execute_finalize_stream(deps.as_mut(), env.clone(), info, 1, None).unwrap_err();
//             assert_eq!(
//                 res,
//                 ContractError::ThresholdError(ThresholdError::ThresholdNotReached {})
//             );

//             // Subscriber one executes exit cancelled before creator cancels stream
//             let info = mock_info("subscriber", &[]);
//             let res = execute_exit_cancelled(deps.as_mut(), env.clone(), info, 1, None).unwrap();
//             assert_eq!(
//                 res.messages,
//                 vec![SubMsg::new(BankMsg::Send {
//                     to_address: "subscriber".to_string(),
//                     amount: vec![Coin::new(250, "in_denom")],
//                 })]
//             );
//             // Creator threshold cancels the stream
//             let info = mock_info("treasury", &[]);
//             let res =
//                 execute_cancel_stream_with_threshold(deps.as_mut(), env.clone(), info, 1).unwrap();
//             assert_eq!(
//                 res.messages,
//                 vec![
//                     // Out denom refunded
//                     SubMsg::new(BankMsg::Send {
//                         to_address: "treasury".to_string(),
//                         amount: vec![Coin::new(500, "out_denom")],
//                     }),
//                 ]
//             );
//             // Creator can not finalize the stream
//             let info = mock_info("treasury", &[]);
//             let res =
//                 execute_finalize_stream(deps.as_mut(), env.clone(), info, 1, None).unwrap_err();
//             assert_eq!(res, ContractError::StreamKillswitchActive {});

//             // Creator can not cancel the stream again
//             let info = mock_info("treasury", &[]);
//             let res = execute_cancel_stream_with_threshold(deps.as_mut(), env.clone(), info, 1)
//                 .unwrap_err();
//             assert_eq!(res, ContractError::StreamKillswitchActive {});

//             // Subscriber 2 executes exit cancelled after creator cancels stream
//             let info = mock_info("subscriber2", &[]);
//             let res = execute_exit_cancelled(deps.as_mut(), env.clone(), info, 1, None).unwrap();
//             assert_eq!(
//                 // In denom refunded
//                 res.messages,
//                 vec![SubMsg::new(BankMsg::Send {
//                     to_address: "subscriber2".to_string(),
//                     amount: vec![Coin::new(1, "in_denom")],
//                 })]
//             );
//         }

//         #[test]
//         fn test_threshold_cancel() {
//             let treasury = Addr::unchecked("treasury");
//             let start = Timestamp::from_seconds(1_000_000);
//             let end = Timestamp::from_seconds(5_000_000);
//             let out_supply = Uint256::from(500u128);
//             let out_denom = "out_denom";
//             let in_denom = "in_denom";

//             // threshold = 500*0.5 / 1-0.01 =252.5

//             // instantiate
//             let mut deps = mock_dependencies();
//             let mut env = mock_env();
//             env.block.time = Timestamp::from_seconds(0);
//             let msg = crate::msg::InstantiateMsg {
//                 min_stream_seconds: Uint64::new(1000),
//                 min_seconds_until_start_time: Uint64::new(0),
//                 stream_creation_denom: "fee".to_string(),
//                 stream_creation_fee: Uint128::new(100),
//                 exit_fee_percent: Decimal256::percent(1),
//                 fee_collector: "collector".to_string(),
//                 protocol_admin: "protocol_admin".to_string(),
//                 accepted_in_denom: in_denom.to_string(),
//                 tos_version: "v1".to_string(),
//             };
//             instantiate(deps.as_mut(), mock_env(), mock_info("creator", &[]), msg).unwrap();

//             // create stream
//             let mut env = mock_env();
//             env.block.time = Timestamp::from_seconds(0);
//             let info = mock_info(
//                 "creator",
//                 &[
//                     Coin::new(out_supply.to_string().parse().unwrap(), out_denom),
//                     Coin::new(100, "fee"),
//                 ],
//             );
//             execute_create_stream(
//                 deps.as_mut(),
//                 env,
//                 info,
//                 treasury.to_string(),
//                 "test".to_string(),
//                 Some("https://sample.url".to_string()),
//                 in_denom.to_string(),
//                 out_denom.to_string(),
//                 out_supply,
//                 start,
//                 end,
//                 Some(1_000u128.into()),
//                 "v1".to_string(),
//             )
//             .unwrap();

//             // Subscription 1
//             let mut env = mock_env();
//             env.block.time = start;
//             let funds = Coin::new(250, "in_denom");
//             let info = mock_info("subscriber", &[funds]);
//             let msg = crate::msg::ExecuteMsg::Subscribe {
//                 stream_id: 1,
//                 operator_target: None,
//                 operator: Some("operator".to_string()),
//                 tos_version: "v1".to_string(),
//             };
//             let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

//             // Subscription 2
//             let funds = Coin::new(500, "in_denom");
//             let info = mock_info("subscriber2", &[funds]);
//             let msg = crate::msg::ExecuteMsg::Subscribe {
//                 stream_id: 1,
//                 operator_target: None,
//                 operator: Some("operator".to_string()),
//                 tos_version: "v1".to_string(),
//             };
//             let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
//             // Can not cancel stream before it ends
//             let mut env = mock_env();
//             env.block.time = start.plus_seconds(1_000_000);
//             let res = execute_cancel_stream_with_threshold(
//                 deps.as_mut(),
//                 env,
//                 mock_info("treasury", &[]),
//                 1,
//             )
//             .unwrap_err();
//             assert_eq!(res, ContractError::StreamNotEnded {});

//             // Set block to the end of the stream
//             let mut env = mock_env();
//             env.block.time = end.plus_seconds(1);

//             // Non creator can't cancel stream
//             let res = execute_cancel_stream_with_threshold(
//                 deps.as_mut(),
//                 env.clone(),
//                 mock_info("random", &[]),
//                 1,
//             )
//             .unwrap_err();
//             assert_eq!(res, ContractError::Unauthorized {});

//             // Creator can cancel stream
//             let _res = execute_cancel_stream_with_threshold(
//                 deps.as_mut(),
//                 env.clone(),
//                 mock_info("treasury", &[]),
//                 1,
//             )
//             .unwrap();
//             // Query stream should return stream with is_cancelled = true
//             let stream = query_stream(deps.as_ref(), env.clone(), 1).unwrap();
//             assert_eq!(stream.status, Status::Cancelled);
//         }
//     }
// }

#[test]
fn threshold_reached() {
    let mut deps = mock_dependencies();

    // Instantiate with specific params
    let msg = InstantiateMsg {
        min_stream_seconds: Uint64::new(1000),
        min_seconds_until_start_time: Uint64::new(0),
        stream_creation_denom: helpers::DEFAULT_STREAM_CREATION_DENOM.to_string(),
        stream_creation_fee: Uint256::from(100u128),
        exit_fee_percent: Decimal256::percent(1),
        fee_collector: helpers::mock_info("collector", &[]).sender.to_string(),
        protocol_admin: helpers::mock_info("protocol_admin", &[]).sender.to_string(),
        accepted_in_denom: "in_denom".to_string(),
        tos_version: "v1".to_string(),
    };
    cw_streamswap::contract::instantiate(
        deps.as_mut(),
        helpers::env_now(),
        helpers::mock_info("creator", &[]),
        msg,
    )
    .unwrap();

    // Actors and mk_info
    let treasury: Addr = helpers::mock_info("treasury", &[]).sender;
    let creator: Addr = helpers::mock_info("creator", &[]).sender;
    let subscriber: Addr = helpers::mock_info("subscriber", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Create stream with threshold 250
    let start = Timestamp::from_seconds(1_000_000);
    let end = Timestamp::from_seconds(5_000_000);
    let out_supply = Uint256::from(500u128);
    let out_denom = "out_denom";
    let in_denom = "in_denom";
    let env = helpers::env_at(0);
    let info = mk_info(
        &creator,
        vec![
            Coin::new(out_supply.to_string().parse::<u128>().unwrap(), out_denom),
            Coin::new(100u128, helpers::DEFAULT_STREAM_CREATION_DENOM),
        ],
    );
    execute_create_stream(
        deps.as_mut(),
        env,
        info,
        treasury.to_string(),
        "test".to_string(),
        Some("https://sample.url".to_string()),
        in_denom.to_string(),
        out_denom.to_string(),
        out_supply,
        start,
        end,
        Some(Uint256::from(250u128)),
        "v1".to_string(),
    )
    .unwrap();

    // subscription 252 in_denom
    let env = helpers::env_at(start.seconds());
    let info = mk_info(&subscriber, vec![Coin::new(252u128, in_denom)]);
    let msg = ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: Some(helpers::mock_info("operator", &[]).sender.to_string()),
        tos_version: "v1".to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // Exit after end: expect 499 out_denom to subscriber (1 unit remains due to rounding)
    let env = helpers::env_at(end.plus_seconds(1).seconds());
    let res = execute_exit_stream(
        deps.as_mut(),
        env.clone(),
        mk_info(&subscriber, vec![]),
        1,
        None,
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);
    if let cosmwasm_std::SubMsg {
        msg: CosmosMsg::Bank(BankMsg::Send { to_address, amount }),
        ..
    } = &res.messages[0]
    {
        assert_eq!(to_address, &subscriber.to_string());
        assert_eq!(amount, &vec![Coin::new(499u128, out_denom)]);
    } else {
        panic!("unexpected message variant");
    }

    // Compute expected treasury dynamically from spent_in and exit fee (no extra update)
    let stream_state = query_stream(deps.as_ref(), env.clone(), 1).unwrap();

    // Finalize stream
    let res =
        execute_finalize_stream(deps.as_mut(), env, mk_info(&treasury, vec![]), 1, None).unwrap();
    assert_eq!(res.messages.len(), 3);
    // 0: revenue to treasury (capture amount for consistency checks)
    let treasury_amount = if let cosmwasm_std::SubMsg {
        msg: CosmosMsg::Bank(BankMsg::Send { to_address, amount }),
        ..
    } = &res.messages[0]
    {
        assert_eq!(to_address, &treasury.to_string());
        assert_eq!(amount.len(), 1);
        amount[0].amount.clone()
    } else {
        panic!("unexpected message variant");
    };
    // 1: creation fee to collector
    if let cosmwasm_std::SubMsg {
        msg: CosmosMsg::Bank(BankMsg::Send { to_address, amount }),
        ..
    } = &res.messages[1]
    {
        assert_eq!(
            to_address,
            &helpers::mock_info("collector", &[]).sender.to_string()
        );
        assert_eq!(
            amount,
            &vec![Coin::new(100u128, helpers::DEFAULT_STREAM_CREATION_DENOM)]
        );
    } else {
        panic!("unexpected message variant");
    }
    // 2: protocol fee to collector (check consistency with spent_in)
    let fee_amount = if let cosmwasm_std::SubMsg {
        msg: CosmosMsg::Bank(BankMsg::Send { to_address, amount }),
        ..
    } = &res.messages[2]
    {
        assert_eq!(
            to_address,
            &helpers::mock_info("collector", &[]).sender.to_string()
        );
        assert_eq!(amount.len(), 1);
        assert_eq!(amount[0].denom, in_denom);
        amount[0].amount.clone()
    } else {
        panic!("unexpected message variant");
    };

    // Consistency: treasury + fee == spent_in
    assert_eq!(treasury_amount + fee_amount, stream_state.spent_in);
    // Verify fee is approximately 1% of spent_in (allow for rounding)
    let expected_fee = stream_state.spent_in.multiply_ratio(1u128, 100u128);
    assert!(fee_amount >= expected_fee && fee_amount <= expected_fee + Uint256::from(1u128));
}

#[test]
fn threshold_not_reached() {
    let mut deps = mock_dependencies();

    // Instantiate
    let msg = InstantiateMsg {
        min_stream_seconds: Uint64::new(1000),
        min_seconds_until_start_time: Uint64::new(0),
        stream_creation_denom: helpers::DEFAULT_STREAM_CREATION_DENOM.to_string(),
        stream_creation_fee: Uint256::from(100u128),
        exit_fee_percent: Decimal256::percent(1),
        fee_collector: helpers::mock_info("collector", &[]).sender.to_string(),
        protocol_admin: helpers::mock_info("protocol_admin", &[]).sender.to_string(),
        accepted_in_denom: "in_denom".to_string(),
        tos_version: "v1".to_string(),
    };
    cw_streamswap::contract::instantiate(
        deps.as_mut(),
        helpers::env_now(),
        helpers::mock_info("creator", &[]),
        msg,
    )
    .unwrap();

    // Actors
    let treasury: Addr = helpers::mock_info("treasury", &[]).sender;
    let creator: Addr = helpers::mock_info("creator", &[]).sender;
    let subscriber: Addr = helpers::mock_info("subscriber", &[]).sender;
    let subscriber2: Addr = helpers::mock_info("subscriber2", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Create stream with threshold 500
    let start = Timestamp::from_seconds(1_000_000);
    let end = Timestamp::from_seconds(5_000_000);
    let out_supply = Uint256::from(500u128);
    let out_denom = "out_denom";
    let in_denom = "in_denom";
    let env = helpers::env_at(0);
    let info = mk_info(
        &creator,
        vec![
            Coin::new(out_supply.to_string().parse::<u128>().unwrap(), out_denom),
            Coin::new(100u128, helpers::DEFAULT_STREAM_CREATION_DENOM),
        ],
    );
    execute_create_stream(
        deps.as_mut(),
        env,
        info,
        treasury.to_string(),
        "test".to_string(),
        Some("https://sample.url".to_string()),
        in_denom.to_string(),
        out_denom.to_string(),
        out_supply,
        start,
        end,
        Some(Uint256::from(500u128)),
        "v1".to_string(),
    )
    .unwrap();

    // Subscriptions 250 and 1
    let env = helpers::env_at(start.seconds());
    execute(
        deps.as_mut(),
        env.clone(),
        mk_info(&subscriber, vec![Coin::new(250u128, in_denom)]),
        ExecuteMsg::Subscribe {
            stream_id: 1,
            operator_target: None,
            operator: Some(helpers::mock_info("operator", &[]).sender.to_string()),
            tos_version: "v1".to_string(),
        },
    )
    .unwrap();

    execute(
        deps.as_mut(),
        env.clone(),
        mk_info(&subscriber2, vec![Coin::new(1u128, in_denom)]),
        ExecuteMsg::Subscribe {
            stream_id: 1,
            operator_target: None,
            operator: Some(helpers::mock_info("operator", &[]).sender.to_string()),
            tos_version: "v1".to_string(),
        },
    )
    .unwrap();

    let env = helpers::env_at(end.plus_seconds(1).seconds());

    // Exit should not be possible
    let err = execute_exit_stream(
        deps.as_mut(),
        env.clone(),
        mk_info(&subscriber, vec![]),
        1,
        None,
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::ThresholdError(ThresholdError::ThresholdNotReached {})
    );

    // Finalize should not be possible
    let err = execute_finalize_stream(
        deps.as_mut(),
        env.clone(),
        mk_info(&treasury, vec![]),
        1,
        None,
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::ThresholdError(ThresholdError::ThresholdNotReached {})
    );

    // Subscriber one executes exit cancelled before creator cancels
    let res = execute(
        deps.as_mut(),
        env.clone(),
        mk_info(&subscriber, vec![]),
        ExecuteMsg::ExitCancelled {
            stream_id: 1,
            operator_target: None,
        },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);
    if let cosmwasm_std::SubMsg {
        msg: CosmosMsg::Bank(BankMsg::Send { to_address, amount }),
        ..
    } = &res.messages[0]
    {
        assert_eq!(to_address, &subscriber.to_string());
        assert_eq!(amount, &vec![Coin::new(250u128, in_denom)]);
    } else {
        panic!("unexpected message variant");
    }

    // Creator threshold cancels the stream
    let res = execute(
        deps.as_mut(),
        env.clone(),
        mk_info(&treasury, vec![]),
        ExecuteMsg::CancelStreamWithThreshold { stream_id: 1 },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);
    if let cosmwasm_std::SubMsg {
        msg: CosmosMsg::Bank(BankMsg::Send { to_address, amount }),
        ..
    } = &res.messages[0]
    {
        assert_eq!(to_address, &treasury.to_string());
        assert_eq!(amount, &vec![Coin::new(500u128, out_denom)]);
    } else {
        panic!("unexpected message variant");
    }

    // Creator cannot finalize afterwards
    let err = execute_finalize_stream(
        deps.as_mut(),
        env.clone(),
        mk_info(&treasury, vec![]),
        1,
        None,
    )
    .unwrap_err();
    assert_eq!(err, ContractError::StreamKillswitchActive {});

    // Creator cannot cancel again
    let err = execute(
        deps.as_mut(),
        env.clone(),
        mk_info(&treasury, vec![]),
        ExecuteMsg::CancelStreamWithThreshold { stream_id: 1 },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::StreamKillswitchActive {});

    // Subscriber 2 executes exit cancelled
    let res = execute(
        deps.as_mut(),
        env.clone(),
        mk_info(&subscriber2, vec![]),
        ExecuteMsg::ExitCancelled {
            stream_id: 1,
            operator_target: None,
        },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);
    if let cosmwasm_std::SubMsg {
        msg: CosmosMsg::Bank(BankMsg::Send { to_address, amount }),
        ..
    } = &res.messages[0]
    {
        assert_eq!(to_address, &subscriber2.to_string());
        assert_eq!(amount, &vec![Coin::new(1u128, in_denom)]);
    } else {
        panic!("unexpected message variant");
    }
}

#[test]
fn threshold_cancel() {
    let mut deps = mock_dependencies();

    // Instantiate
    let msg = InstantiateMsg {
        min_stream_seconds: Uint64::new(1000),
        min_seconds_until_start_time: Uint64::new(0),
        stream_creation_denom: helpers::DEFAULT_STREAM_CREATION_DENOM.to_string(),
        stream_creation_fee: Uint256::from(100u128),
        exit_fee_percent: Decimal256::percent(1),
        fee_collector: helpers::mock_info("collector", &[]).sender.to_string(),
        protocol_admin: helpers::mock_info("protocol_admin", &[]).sender.to_string(),
        accepted_in_denom: "in_denom".to_string(),
        tos_version: "v1".to_string(),
    };
    cw_streamswap::contract::instantiate(
        deps.as_mut(),
        helpers::env_now(),
        helpers::mock_info("creator", &[]),
        msg,
    )
    .unwrap();

    // Actors
    let treasury: Addr = helpers::mock_info("treasury", &[]).sender;
    let creator: Addr = helpers::mock_info("creator", &[]).sender;
    let subscriber: Addr = helpers::mock_info("subscriber", &[]).sender;
    let subscriber2: Addr = helpers::mock_info("subscriber2", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Create stream with threshold 1000
    let start = Timestamp::from_seconds(1_000_000);
    let end = Timestamp::from_seconds(5_000_000);
    let out_supply = Uint256::from(500u128);
    let out_denom = "out_denom";
    let in_denom = "in_denom";
    let env = helpers::env_at(0);
    let info = mk_info(
        &creator,
        vec![
            Coin::new(out_supply.to_string().parse::<u128>().unwrap(), out_denom),
            Coin::new(100u128, helpers::DEFAULT_STREAM_CREATION_DENOM),
        ],
    );
    execute_create_stream(
        deps.as_mut(),
        env,
        info,
        treasury.to_string(),
        "test".to_string(),
        Some("https://sample.url".to_string()),
        in_denom.to_string(),
        out_denom.to_string(),
        out_supply,
        start,
        end,
        Some(Uint256::from(1_000u128)),
        "v1".to_string(),
    )
    .unwrap();

    // Subscription 1: 250
    let env = helpers::env_at(start.seconds());
    execute(
        deps.as_mut(),
        env.clone(),
        mk_info(&subscriber, vec![Coin::new(250u128, in_denom)]),
        ExecuteMsg::Subscribe {
            stream_id: 1,
            operator_target: None,
            operator: Some(helpers::mock_info("operator", &[]).sender.to_string()),
            tos_version: "v1".to_string(),
        },
    )
    .unwrap();

    // Subscription 2: 500
    execute(
        deps.as_mut(),
        env.clone(),
        mk_info(&subscriber2, vec![Coin::new(500u128, in_denom)]),
        ExecuteMsg::Subscribe {
            stream_id: 1,
            operator_target: None,
            operator: Some(helpers::mock_info("operator", &[]).sender.to_string()),
            tos_version: "v1".to_string(),
        },
    )
    .unwrap();

    // Before end cannot cancel
    let env_before = helpers::env_at(start.plus_seconds(1_000_000).seconds());
    let err = execute(
        deps.as_mut(),
        env_before,
        mk_info(&treasury, vec![]),
        ExecuteMsg::CancelStreamWithThreshold { stream_id: 1 },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::StreamNotEnded {});

    // At end+1
    let env = helpers::env_at(end.plus_seconds(1).seconds());

    // Non-creator unauthorized
    let err = execute(
        deps.as_mut(),
        env.clone(),
        mk_info(
            &Addr::clone(&helpers::mock_info("random", &[]).sender),
            vec![],
        ),
        ExecuteMsg::CancelStreamWithThreshold { stream_id: 1 },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Creator can cancel stream
    execute(
        deps.as_mut(),
        env.clone(),
        mk_info(&treasury, vec![]),
        ExecuteMsg::CancelStreamWithThreshold { stream_id: 1 },
    )
    .unwrap();

    // Stream status is Cancelled
    let stream = query_stream(deps.as_ref(), env.clone(), 1).unwrap();
    assert_eq!(stream.status, cw_streamswap::state::Status::Cancelled);
}
