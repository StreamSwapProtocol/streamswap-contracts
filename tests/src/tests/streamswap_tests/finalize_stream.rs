#[cfg(test)]
mod finalize_stream_tests {
    use std::str::FromStr;

    use crate::helpers::mock_messages::CreateStreamMsgBuilder;
    use crate::helpers::suite::SuiteBuilder;
    use crate::helpers::utils::{get_contract_address_from_res, get_funds_from_res};
    use crate::helpers::{mock_messages::get_controller_inst_msg, suite::Suite};
    use cosmwasm_std::{coin, Addr, BlockInfo, Coin, Uint128, Uint256};
    use cw_multi_test::Executor;
    use streamswap_types::stream::{
        ExecuteMsg as StreamSwapExecuteMsg, FinalizedStatus, QueryMsg as StreamSwapQueryMsg,
        Status, StreamResponse,
    };

    #[test]
    fn recurring_finalize_stream_calls() {
        let Suite {
            mut app,
            test_accounts,
            stream_swap_code_id,
            stream_swap_controller_code_id,
            vesting_code_id,
        } = SuiteBuilder::default().build();
        let start_time = app.block_info().time.plus_seconds(100);
        let end_time = app.block_info().time.plus_seconds(200);
        let bootstrapping_start_time = app.block_info().time.plus_seconds(50);
        let msg = get_controller_inst_msg(stream_swap_code_id, vesting_code_id, &test_accounts);
        let controller_address = app
            .instantiate_contract(
                stream_swap_controller_code_id,
                test_accounts.admin.clone(),
                &msg,
                &[],
                "Controller".to_string(),
                None,
            )
            .unwrap();
        let create_stream_msg = CreateStreamMsgBuilder::new(
            "Stream Swap tests",
            test_accounts.creator_1.as_ref(),
            coin(1_000_000, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            start_time,
            end_time,
        )
        .build();
        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(1_000_000, "out_denom")],
            )
            .unwrap();
        let stream_swap_contract_address: String = get_contract_address_from_res(res);
        let subscribe_msg = StreamSwapExecuteMsg::Subscribe {};
        app.set_block(BlockInfo {
            height: 1_100,
            time: start_time,
            chain_id: "test".to_string(),
        });
        // First subscription
        let _res = app
            .execute_contract(
                test_accounts.subscriber_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &subscribe_msg,
                &[coin(200, "in_denom")],
            )
            .unwrap();
        app.set_block(BlockInfo {
            height: 1_100,
            time: end_time.plus_seconds(1),
            chain_id: "test".to_string(),
        });
        let finalized_msg = StreamSwapExecuteMsg::FinalizeStream {
            new_treasury: None,
            create_pool: None,
            salt: None,
        };
        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &finalized_msg,
                &[],
            )
            .unwrap();
        let stream_swap_funds = get_funds_from_res(res);
        assert_eq!(
            stream_swap_funds,
            vec![
                (
                    String::from(test_accounts.creator_1.clone()),
                    Coin {
                        denom: "in_denom".to_string(),
                        amount: Uint128::new(198)
                    }
                ),
                (
                    String::from(test_accounts.admin.clone(),),
                    Coin {
                        denom: "in_denom".to_string(),
                        amount: Uint128::new(2)
                    }
                ),
            ]
        );
        // Query the stream status(Check stream status)
        let stream: StreamResponse = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(stream_swap_contract_address.clone()),
                &StreamSwapQueryMsg::Stream {},
            )
            .unwrap();

        assert_eq!(
            stream.status,
            Status::Finalized(FinalizedStatus::ThresholdReached)
        );
        // Creator_1 can finalize the stream only once
        let _res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &finalized_msg,
                &[coin(150, "in_denom")],
            )
            .unwrap_err();
    }

    #[test]
    fn finalize_authorizations() {
        let Suite {
            mut app,
            test_accounts,
            stream_swap_code_id,
            stream_swap_controller_code_id,
            vesting_code_id,
        } = SuiteBuilder::default().build();
        let start_time = app.block_info().time.plus_seconds(100);
        let end_time = app.block_info().time.plus_seconds(200);
        let bootstrapping_start_time = app.block_info().time.plus_seconds(50);
        let msg = get_controller_inst_msg(stream_swap_code_id, vesting_code_id, &test_accounts);
        let controller_address = app
            .instantiate_contract(
                stream_swap_controller_code_id,
                test_accounts.admin.clone(),
                &msg,
                &[],
                "Controller".to_string(),
                None,
            )
            .unwrap();
        let create_stream_msg = CreateStreamMsgBuilder::new(
            "Stream Swap tests",
            test_accounts.creator_1.as_ref(),
            coin(1_000_000, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            start_time,
            end_time,
        )
        .build();
        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(1_000_000, "out_denom")],
            )
            .unwrap();
        let stream_swap_contract_address: String = get_contract_address_from_res(res);
        let subscribe_msg = StreamSwapExecuteMsg::Subscribe {};
        app.set_block(BlockInfo {
            height: 1_100,
            time: start_time,
            chain_id: "test".to_string(),
        });
        // First subscription
        let _res = app
            .execute_contract(
                test_accounts.subscriber_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &subscribe_msg,
                &[coin(200, "in_denom")],
            )
            .unwrap();
        // Finalizing with wrong user
        let finalized_msg = StreamSwapExecuteMsg::FinalizeStream {
            new_treasury: None,
            create_pool: None,
            salt: None,
        };
        app.set_block(BlockInfo {
            height: 1_100,
            time: end_time.plus_seconds(100),
            chain_id: "test".to_string(),
        });
        let _res = app
            .execute_contract(
                test_accounts.wrong_user.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &finalized_msg,
                &[],
            )
            .unwrap_err();
        // Finalize with correct user that is creator
        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &finalized_msg,
                &[],
            )
            .unwrap();
        let stream_swap_funds = get_funds_from_res(res);
        assert_eq!(
            stream_swap_funds,
            vec![
                (
                    String::from(test_accounts.creator_1.clone()),
                    Coin {
                        denom: "in_denom".to_string(),
                        amount: Uint128::new(198)
                    }
                ),
                (
                    String::from(test_accounts.admin.clone(),),
                    Coin {
                        denom: "in_denom".to_string(),
                        amount: Uint128::new(2)
                    }
                ),
            ]
        );
        // Query the stream status
        let stream: StreamResponse = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(stream_swap_contract_address.clone()),
                &StreamSwapQueryMsg::Stream {},
            )
            .unwrap();

        assert_eq!(
            stream.status,
            Status::Finalized(FinalizedStatus::ThresholdReached)
        );
    }

    #[test]
    fn finalize_with_new_treasury() {
        let Suite {
            mut app,
            test_accounts,
            stream_swap_code_id,
            stream_swap_controller_code_id,
            vesting_code_id,
        } = SuiteBuilder::default().build();
        let start_time = app.block_info().time.plus_seconds(100);
        let end_time = app.block_info().time.plus_seconds(200);
        let bootstrapping_start_time = app.block_info().time.plus_seconds(50);
        let msg = get_controller_inst_msg(stream_swap_code_id, vesting_code_id, &test_accounts);
        let controller_address = app
            .instantiate_contract(
                stream_swap_controller_code_id,
                test_accounts.admin.clone(),
                &msg,
                &[],
                "Controller".to_string(),
                None,
            )
            .unwrap();
        let create_stream_msg = CreateStreamMsgBuilder::new(
            "Stream Swap tests",
            test_accounts.creator_1.as_ref(),
            coin(1_000_000, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            start_time,
            end_time,
        )
        .build();
        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(1_000_000, "out_denom")],
            )
            .unwrap();
        let stream_swap_contract_address: String = get_contract_address_from_res(res);
        let subscribe_msg = StreamSwapExecuteMsg::Subscribe {};
        app.set_block(BlockInfo {
            height: 1_100,
            time: start_time,
            chain_id: "test".to_string(),
        });
        // First subscription
        let _res = app
            .execute_contract(
                test_accounts.subscriber_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &subscribe_msg,
                &[coin(200, "in_denom")],
            )
            .unwrap();
        //Update environment time to end_time
        app.set_block(BlockInfo {
            height: 1_100,
            time: end_time,
            chain_id: "test".to_string(),
        });
        // Finalizing with wrong user with new treasury
        let finalized_msg = StreamSwapExecuteMsg::FinalizeStream {
            new_treasury: Some(test_accounts.creator_1.to_string()),
            create_pool: None,
            salt: None,
        };
        app.set_block(BlockInfo {
            height: 1_100,
            time: end_time.plus_seconds(100),
            chain_id: "test".to_string(),
        });
        let _res = app
            .execute_contract(
                test_accounts.wrong_user.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &finalized_msg,
                &[],
            )
            .unwrap_err();
        // Finalize with correct user with new treasury as creator_2
        let finalized_msg = StreamSwapExecuteMsg::FinalizeStream {
            new_treasury: Some(test_accounts.creator_1.to_string()),
            create_pool: None,
            salt: None,
        };
        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &finalized_msg,
                &[],
            )
            .unwrap();
        // Check if the funds are transferred to the new treasury
        let stream_swap_funds = get_funds_from_res(res);
        assert_eq!(
            stream_swap_funds,
            vec![
                (
                    String::from(test_accounts.creator_1.clone()),
                    Coin {
                        denom: "in_denom".to_string(),
                        amount: Uint128::new(198)
                    }
                ),
                (
                    String::from(test_accounts.admin.clone()),
                    Coin {
                        denom: "in_denom".to_string(),
                        amount: Uint128::new(2)
                    }
                ),
            ]
        );
        // Query the stream status(Check stream status)
        let stream: StreamResponse = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(stream_swap_contract_address.clone()),
                &StreamSwapQueryMsg::Stream {},
            )
            .unwrap();

        assert_eq!(
            stream.status,
            Status::Finalized(FinalizedStatus::ThresholdReached)
        );
    }

    #[test]
    fn finalize_stream_threshold_not_reached() {
        let Suite {
            mut app,
            test_accounts,
            stream_swap_code_id,
            stream_swap_controller_code_id,
            vesting_code_id,
        } = SuiteBuilder::default().build();
        let start_time = app.block_info().time.plus_seconds(100);
        let end_time = app.block_info().time.plus_seconds(200);
        let bootstrapping_start_time = app.block_info().time.plus_seconds(50);
        let threshold = Uint256::from(1_000u128);
        let msg = get_controller_inst_msg(stream_swap_code_id, vesting_code_id, &test_accounts);
        let controller_address = app
            .instantiate_contract(
                stream_swap_controller_code_id,
                test_accounts.admin.clone(),
                &msg,
                &[],
                "Controller".to_string(),
                None,
            )
            .unwrap();
        let create_stream_msg = CreateStreamMsgBuilder::new(
            "Stream Swap tests",
            test_accounts.creator_1.as_ref(),
            coin(1_000_000, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            start_time,
            end_time,
        )
        .threshold(threshold)
        .build();
        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(1_000_000, "out_denom")],
            )
            .unwrap();
        let stream_swap_contract_address: String = get_contract_address_from_res(res);
        let subscribe_msg = StreamSwapExecuteMsg::Subscribe {};
        app.set_block(BlockInfo {
            height: 1_100,
            time: start_time,
            chain_id: "test".to_string(),
        });
        // First subscription
        let _res = app
            .execute_contract(
                test_accounts.subscriber_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &subscribe_msg,
                &[coin(200, "in_denom")],
            )
            .unwrap();
        //Update environment time to end_time
        app.set_block(BlockInfo {
            height: 1_100,
            time: end_time,
            chain_id: "test".to_string(),
        });

        // Finalize stream with threshold not reached
        let finalized_msg = StreamSwapExecuteMsg::FinalizeStream {
            new_treasury: None,
            create_pool: None,
            salt: None,
        };
        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &finalized_msg,
                &[],
            )
            .unwrap();
        let stream_swap_funds = get_funds_from_res(res);
        assert_eq!(
            stream_swap_funds,
            vec![(
                String::from(test_accounts.creator_1.clone()),
                Coin {
                    denom: "out_denom".to_string(),
                    amount: Uint128::new(1_000_000)
                }
            ),]
        );
        assert_eq!(stream_swap_funds.len(), 1);
    }

    #[test]
    fn finalize_stream_threshold_not_reached_pool_refund() {
        let Suite {
            mut app,
            test_accounts,
            stream_swap_code_id,
            stream_swap_controller_code_id,
            vesting_code_id,
        } = SuiteBuilder::default().build();
        let start_time = app.block_info().time.plus_seconds(100);
        let end_time = app.block_info().time.plus_seconds(200);
        let bootstrapping_start_time = app.block_info().time.plus_seconds(50);
        let threshold = Uint256::from(1_000u128);
        let pool_creation_fee: Coin = coin(1000000, "fee_denom");
        let pool_out_amount_clp: Uint256 = Uint256::from(500_000u128);
        let msg = get_controller_inst_msg(stream_swap_code_id, vesting_code_id, &test_accounts);
        let controller_address = app
            .instantiate_contract(
                stream_swap_controller_code_id,
                test_accounts.admin.clone(),
                &msg,
                &[],
                "Controller".to_string(),
                None,
            )
            .unwrap();
        let create_stream_msg = CreateStreamMsgBuilder::new(
            "Stream Swap tests",
            test_accounts.creator_1.as_ref(),
            coin(1_000_000, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            start_time,
            end_time,
        )
        .threshold(threshold)
        .pool_config(
            streamswap_types::controller::PoolConfig::ConcentratedLiquidity {
                out_amount_clp: pool_out_amount_clp,
            },
        )
        .build();

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[
                    coin(100, "fee_denom"),
                    coin(1_000_000, "out_denom"),
                    pool_creation_fee.clone(),
                    coin(
                        Uint128::from_str(pool_out_amount_clp.to_string().as_str())
                            .unwrap()
                            .u128(),
                        "out_denom",
                    ),
                ],
            )
            .unwrap();
        let stream_swap_contract_address: String = get_contract_address_from_res(res);
        let subscribe_msg = StreamSwapExecuteMsg::Subscribe {};
        app.set_block(BlockInfo {
            height: 1_100,
            time: start_time,
            chain_id: "test".to_string(),
        });
        // First subscription
        let _res = app
            .execute_contract(
                test_accounts.subscriber_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &subscribe_msg,
                &[coin(200, "in_denom")],
            )
            .unwrap();
        //Update environment time to end_time
        app.set_block(BlockInfo {
            height: 1_100,
            time: end_time,
            chain_id: "test".to_string(),
        });

        // Finalize stream with threshold not reached
        let finalized_msg = StreamSwapExecuteMsg::FinalizeStream {
            new_treasury: None,
            create_pool: None,
            salt: None,
        };
        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &finalized_msg,
                &[],
            )
            .unwrap();
        let stream_swap_funds = get_funds_from_res(res);

        assert_eq!(
            stream_swap_funds,
            vec![
                (
                    String::from(test_accounts.creator_1.clone()),
                    pool_creation_fee.clone()
                ),
                (
                    String::from(test_accounts.creator_1.clone()),
                    Coin {
                        denom: "out_denom".to_string(),
                        amount: Uint128::new(1_000_000 + 500_000)
                    }
                ),
            ]
        );
        // Contract only has the subscription amount left in the contract
        let contract_balance = app
            .wrap()
            .query_all_balances(Addr::unchecked(stream_swap_contract_address.clone()))
            .unwrap();
        assert_eq!(contract_balance.len(), 1);
        assert_eq!(
            contract_balance[0],
            Coin {
                denom: "in_denom".to_string(),
                amount: Uint128::new(200)
            }
        );

        // Query the stream status(Check stream status)
        let stream: StreamResponse = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(stream_swap_contract_address.clone()),
                &StreamSwapQueryMsg::Stream {},
            )
            .unwrap();

        assert_eq!(
            stream.status,
            Status::Finalized(FinalizedStatus::ThresholdNotReached)
        );

        // Subscriber 1 exits the stream
        let exit_stream_msg = StreamSwapExecuteMsg::ExitStream { salt: None };

        let res = app
            .execute_contract(
                test_accounts.subscriber_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &exit_stream_msg,
                &[],
            )
            .unwrap();
        let stream_swap_funds = get_funds_from_res(res);
        assert_eq!(
            stream_swap_funds,
            vec![(
                String::from(test_accounts.subscriber_1.clone()),
                Coin {
                    denom: "in_denom".to_string(),
                    amount: Uint128::new(200)
                }
            ),]
        );

        // Contract has no funds left
        let contract_balance = app
            .wrap()
            .query_all_balances(Addr::unchecked(stream_swap_contract_address.clone()))
            .unwrap();
        assert_eq!(contract_balance.len(), 0);
    }
}
