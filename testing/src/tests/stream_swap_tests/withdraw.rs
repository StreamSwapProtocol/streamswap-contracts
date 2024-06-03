#[cfg(test)]
mod withdraw_tests {

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
        ContractError as StreamSwapError,
    };
    use cw_streamswap_factory::msg::QueryMsg as FactoryQueryMsg;
    use cw_utils::PaymentError;

    #[test]
    fn test_withdraw_pending() {
        let setup_res = setup();
        let test_accounts = setup_res.test_accounts;
        let mut app = setup_res.app;

        // Instantiate stream swap
        let stream_swap_code_id = setup_res.stream_swap_code_id;
        let stream_swap_factory_code_id = setup_res.stream_swap_factory_code_id;
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
        let start_time = app.block_info().time.plus_seconds(100).into();
        let end_time = app.block_info().time.plus_seconds(200).into();

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
                factory_address,
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(1_000_000, "out_denom")],
            )
            .unwrap();
        let stream_swap_contract_address = get_contract_address_from_res(res);

        // Subscribe to stream
        let subscribe_msg = StreamSwapExecuteMsg::Subscribe {
            operator_target: None,
            operator: None,
        };
        let subscriber_1_balance_before = app
            .wrap()
            .query_balance(test_accounts.subscriber.clone(), "in_denom")
            .unwrap();

        let res = app
            .execute_contract(
                test_accounts.subscriber.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &subscribe_msg,
                &[coin(1_000, "in_denom")],
            )
            .unwrap();
        let subscriber_1_balance_after = app
            .wrap()
            .query_balance(test_accounts.subscriber.clone(), "in_denom")
            .unwrap();
        // Subscriber balance should be reduced by 1_000 after subscription
        assert_eq!(
            subscriber_1_balance_before
                .amount
                .checked_sub(Uint128::new(1_000))
                .unwrap(),
            subscriber_1_balance_after.amount
        );
        // Update position
        let _res = app
            .execute_contract(
                test_accounts.subscriber.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &StreamSwapExecuteMsg::UpdatePosition {
                    operator_target: None,
                },
                &[],
            )
            .unwrap();

        // Query position
        let position: PositionResponse = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(stream_swap_contract_address.clone()),
                &StreamSwapQueryMsg::Position {
                    owner: test_accounts.subscriber.clone().into_string(),
                },
            )
            .unwrap();
        assert_eq!(position.purchased, Uint128::zero());
        assert_eq!(position.spent, Uint128::zero());
        assert_eq!(position.shares, Uint128::new(1_000));

        // Withdraw before start time
        let subscriber_1_balance_before = app
            .wrap()
            .query_balance(test_accounts.subscriber.clone(), "in_denom")
            .unwrap();

        let _res = app
            .execute_contract(
                test_accounts.subscriber.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &StreamSwapExecuteMsg::Withdraw {
                    cap: Some(Uint128::new(500)),
                    operator_target: None,
                },
                &[],
            )
            .unwrap();
        let subscriber_1_balance_after = app
            .wrap()
            .query_balance(test_accounts.subscriber.clone(), "in_denom")
            .unwrap();
        // Subscriber balance should be increased by 1_000 after withdraw
        assert_eq!(
            subscriber_1_balance_before
                .amount
                .checked_add(Uint128::new(500))
                .unwrap(),
            subscriber_1_balance_after.amount
        );
        // Query stream
        let stream: StreamResponse = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(stream_swap_contract_address.clone()),
                &StreamSwapQueryMsg::Stream {},
            )
            .unwrap();
        assert_eq!(stream.dist_index, Decimal256::zero());
        assert_eq!(stream.in_supply, Uint128::new(500));
        assert_eq!(stream.spent_in, Uint128::zero());

        // Withdraw rest of the funds
        let subscriber_1_balance_before = app
            .wrap()
            .query_balance(test_accounts.subscriber.clone(), "in_denom")
            .unwrap();
        let _res = app
            .execute_contract(
                test_accounts.subscriber.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &StreamSwapExecuteMsg::Withdraw {
                    cap: None,
                    operator_target: None,
                },
                &[],
            )
            .unwrap();

        let subscriber_1_balance_after = app
            .wrap()
            .query_balance(test_accounts.subscriber.clone(), "in_denom")
            .unwrap();
        // Subscriber balance should be increased by 500 after withdraw
        assert_eq!(
            subscriber_1_balance_after
                .amount
                .checked_sub(subscriber_1_balance_before.amount)
                .unwrap(),
            Uint128::new(500)
        );
        // Query stream
        let stream: StreamResponse = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(stream_swap_contract_address.clone()),
                &StreamSwapQueryMsg::Stream {},
            )
            .unwrap();
        assert_eq!(stream.in_supply, Uint128::zero());
        assert_eq!(stream.spent_in, Uint128::new(0));

        // Set block time to end time
        app.set_block(BlockInfo {
            height: 200,
            time: end_time.plus_seconds(1),
            chain_id: "test".to_string(),
        });
        // Exit stream wont work because the subscriber has withdrawn all the funds
        let err = app
            .execute_contract(
                test_accounts.subscriber.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &StreamSwapExecuteMsg::ExitStream {
                    operator_target: None,
                },
                &[],
            )
            .unwrap_err();
    }

    #[test]
    fn test_withdraw_all_before_exit_case() {
        let setup_res = setup();
        let test_accounts = setup_res.test_accounts;
        let mut app = setup_res.app;

        // Instantiate stream swap
        let stream_swap_code_id = setup_res.stream_swap_code_id;
        let stream_swap_factory_code_id = setup_res.stream_swap_factory_code_id;
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
        let start_time = app.block_info().time.plus_seconds(1000).into();
        let end_time = app.block_info().time.plus_seconds(5000).into();

        let create_stream_msg = get_create_stream_msg(
            &"Stream Swap test".to_string(),
            Some("https://sample.url".to_string()),
            &test_accounts.creator.to_string(),
            coin(1_000_000_000_000, "out_denom"),
            "in_denom",
            start_time,
            end_time,
            None,
        );

        let res = app
            .execute_contract(
                test_accounts.creator.clone(),
                factory_address,
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(1_000_000_000_000, "out_denom")],
            )
            .unwrap();
        let stream_swap_contract_address = get_contract_address_from_res(res);

        // First subscription
        app.set_block(BlockInfo {
            height: 1000,
            time: start_time,
            chain_id: "test".to_string(),
        });

        let subscribe_msg = StreamSwapExecuteMsg::Subscribe {
            operator_target: None,
            operator: None,
        };
        let _res = app
            .execute_contract(
                test_accounts.subscriber.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &subscribe_msg,
                &[coin(2_000_000_000_000, "in_denom")],
            )
            .unwrap();
        app.set_block(BlockInfo {
            height: 2000,
            time: start_time.plus_seconds(1),
            chain_id: "test".to_string(),
        });

        // Second subscription
        let _res = app
            .execute_contract(
                test_accounts.subscriber_2.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &subscribe_msg,
                &[coin(1_000_000_000_000, "in_denom")],
            )
            .unwrap();

        app.set_block(BlockInfo {
            height: 3000,
            time: start_time.plus_seconds(2),
            chain_id: "test".to_string(),
        });

        // Third subscription
        let _res = app
            .execute_contract(
                test_accounts.admin.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &subscribe_msg,
                &[coin(10, "in_denom")],
            )
            .unwrap();

        // First withdraw
        app.set_block(BlockInfo {
            height: 2000,
            time: start_time.plus_seconds(10),
            chain_id: "test".to_string(),
        });

        let withdraw_msg = StreamSwapExecuteMsg::Withdraw {
            cap: None,
            operator_target: None,
        };
        let _res = app
            .execute_contract(
                test_accounts.subscriber.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &withdraw_msg,
                &[],
            )
            .unwrap();

        // Second withdraw
        let _res = app
            .execute_contract(
                test_accounts.subscriber_2.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &withdraw_msg,
                &[],
            )
            .unwrap();

        // Exit stream
        app.set_block(BlockInfo {
            height: 3000,
            time: end_time.plus_seconds(1),
            chain_id: "test".to_string(),
        });

        let _res = app
            .execute_contract(
                test_accounts.subscriber.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &StreamSwapExecuteMsg::ExitStream {
                    operator_target: None,
                },
                &[],
            )
            .unwrap();

        let _res = app
            .execute_contract(
                test_accounts.subscriber_2.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &StreamSwapExecuteMsg::ExitStream {
                    operator_target: None,
                },
                &[],
            )
            .unwrap();
    }
    #[test]
    fn test_withdraw() {
        let setup_res = setup();
        let test_accounts = setup_res.test_accounts;
        let mut app = setup_res.app;

        // Instantiate stream swap
        let stream_swap_code_id = setup_res.stream_swap_code_id;
        let stream_swap_factory_code_id = setup_res.stream_swap_factory_code_id;
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
        let start_time = app.block_info().time.plus_seconds(100).into();
        let end_time = app.block_info().time.plus_seconds(200).into();

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
                factory_address,
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(1_000_000, "out_denom")],
            )
            .unwrap();
        let stream_swap_contract_address = get_contract_address_from_res(res);

        app.set_block(BlockInfo {
            height: 100,
            time: start_time,
            chain_id: "test".to_string(),
        });
        // Subscribe to stream
        let _res = app
            .execute_contract(
                test_accounts.subscriber.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &StreamSwapExecuteMsg::Subscribe {
                    operator_target: None,
                    operator: None,
                },
                &[coin(1_000, "in_denom")],
            )
            .unwrap();

        // Withdraw with cap
        let subscriber_1_balance_before = app
            .wrap()
            .query_balance(test_accounts.subscriber.clone(), "in_denom")
            .unwrap();

        let _res = app
            .execute_contract(
                test_accounts.subscriber.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &StreamSwapExecuteMsg::Withdraw {
                    cap: Some(Uint128::new(500)),
                    operator_target: None,
                },
                &[],
            )
            .unwrap();
        let subscriber_1_balance_after = app
            .wrap()
            .query_balance(test_accounts.subscriber.clone(), "in_denom")
            .unwrap();
        // Subscriber balance should be increased by 500 after withdraw
        assert_eq!(
            subscriber_1_balance_after
                .amount
                .checked_sub(subscriber_1_balance_before.amount)
                .unwrap(),
            Uint128::new(500)
        );

        // Withdraw amount zero
        let err = app
            .execute_contract(
                test_accounts.subscriber.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &StreamSwapExecuteMsg::Withdraw {
                    cap: Some(Uint128::zero()),
                    operator_target: None,
                },
                &[],
            )
            .unwrap_err();
        let error = err.source().unwrap();
        let error = error.downcast_ref::<StreamSwapError>().unwrap();
        assert_eq!(error, &StreamSwapError::InvalidWithdrawAmount {});

        // Withdraw amount too high
        let err = app
            .execute_contract(
                test_accounts.subscriber.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &StreamSwapExecuteMsg::Withdraw {
                    cap: Some(Uint128::new(2_250_000_000_000)),
                    operator_target: None,
                },
                &[],
            )
            .unwrap_err();
        let error = err.source().unwrap();
        let error = error.downcast_ref::<StreamSwapError>().unwrap();
        assert_eq!(
            error,
            &StreamSwapError::WithdrawAmountExceedsBalance(Uint128::new(2_250_000_000_000))
        );

        // Withdraw with valid cap
        let subscriber_1_balance_before = app
            .wrap()
            .query_balance(test_accounts.subscriber.clone(), "in_denom")
            .unwrap();

        let _res = app
            .execute_contract(
                test_accounts.subscriber.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &StreamSwapExecuteMsg::Withdraw {
                    cap: Some(Uint128::new(500)),
                    operator_target: None,
                },
                &[],
            )
            .unwrap();
        let subscriber_1_balance_after = app
            .wrap()
            .query_balance(test_accounts.subscriber.clone(), "in_denom")
            .unwrap();
        // Subscriber balance should be increased by 500 after withdraw
        assert_eq!(
            subscriber_1_balance_after
                .amount
                .checked_sub(subscriber_1_balance_before.amount)
                .unwrap(),
            Uint128::new(500)
        );
    }
}

//     fn test_withdraw() {
//         let treasury = Addr::unchecked("treasury");
//         let start = 1_000_000;
//         let end = 5_000_000;
//         let out_supply = Uint128::new(1_000_000_000_000);
//         let out_denom = "out_denom";

//         // instantiate
//         let mut deps = mock_dependencies();
//         let mut env = mock_env();
//         env.block.height = 0;
//         let msg = crate::msg::InstantiateMsg {
//             min_stream_blocks: 1000,
//             min_blocks_until_start_block: 1000,
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
//         env.block.height = 0;
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

//         // first subscription
//         let mut env = mock_env();
//         env.block.height = start + 0;
//         let funds = Coin::new(2_000_000_000_000, "in");
//         let info = mock_info("creator1", &[funds.clone()]);
//         let msg = crate::msg::ExecuteMsg::Subscribe {
//             stream_id: 1,
//             operator_target: None,
//             operator: None,
//         };
//         let _res = execute(deps.as_mut(), env, info, msg).unwrap();

//         // withdraw with cap
//         let mut env = mock_env();
//         env.block.height = start + 5000;
//         let info = mock_info("creator1", &[]);
//         // withdraw amount zero
//         let cap = Uint128::zero();
//         let msg = crate::msg::ExecuteMsg::Withdraw {
//             stream_id: 1,
//             cap: Some(cap),
//             operator_target: None,
//         };
//         let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
//         assert_eq!(res, ContractError::InvalidWithdrawAmount {});
//         // withdraw amount too high
//         let cap = Uint128::new(2_250_000_000_000);
//         let msg = crate::msg::ExecuteMsg::Withdraw {
//             stream_id: 1,
//             cap: Some(cap),
//             operator_target: None,
//         };
//         let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
//         assert_eq!(
//             res,
//             ContractError::WithdrawAmountExceedsBalance(Uint128::new(2250000000000))
//         );
//         //withdraw with valid cap
//         let cap = Uint128::new(25_000_000);
//         let msg = crate::msg::ExecuteMsg::Withdraw {
//             stream_id: 1,
//             cap: Some(cap),
//             operator_target: None,
//         };
//         let _res = execute(deps.as_mut(), env, info, msg).unwrap();
//         let position =
//             query_position(deps.as_ref(), mock_env(), 1, "creator1".to_string()).unwrap();
//         assert_eq!(position.in_balance, Uint128::new(1_997_475_000_000));
//         assert_eq!(position.spent, Uint128::new(2_500_000_000));
//         assert_eq!(position.purchased, Uint128::new(1_250_000_000));
//         // first fund amount should be equal to in_balance + spent + cap
//         assert_eq!(position.in_balance + position.spent + cap, funds.amount);

//         let mut env = mock_env();
//         env.block.height = start + 1_000_000;
//         let info = mock_info("creator1", &[]);
//         let msg = crate::msg::ExecuteMsg::Withdraw {
//             stream_id: 1,
//             cap: None,
//             operator_target: None,
//         };
//         let res = execute(deps.as_mut(), env, info, msg).unwrap();
//         let position =
//             query_position(deps.as_ref(), mock_env(), 1, "creator1".to_string()).unwrap();
//         assert_eq!(position.in_balance, Uint128::zero());
//         assert_eq!(position.spent, Uint128::new(499_993_773_466));
//         assert_eq!(position.purchased, Uint128::new(249_999_999_998));
//         assert_eq!(position.shares, Uint128::zero());
//         let msg = res.messages.get(0).unwrap();
//         assert_eq!(
//             msg.msg,
//             CosmosMsg::Bank(BankMsg::Send {
//                 to_address: "creator1".to_string(),
//                 amount: vec![Coin::new(1_499_981_226_534, "in")]
//             })
//         );

//         // can't withdraw after stream ends
//         let mut env = mock_env();
//         env.block.height = end + 1;
//         let info = mock_info("creator1", &[]);
//         let msg = crate::msg::ExecuteMsg::Withdraw {
//             stream_id: 1,
//             cap: None,
//             operator_target: None,
//         };
//         let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
//         assert_eq!(res, ContractError::StreamEnded {});
//     }
