#[cfg(test)]
mod pool {
    use crate::helpers::mock_messages::{get_controller_inst_msg, CreateStreamMsgBuilder};
    use crate::helpers::suite::{Suite, SuiteBuilder};
    use crate::helpers::utils::{
        get_contract_address_from_res, get_funds_from_res, get_wasm_attribute_with_key,
    };
    use cosmwasm_std::{coin, Addr, BlockInfo, Coin, Uint256};
    use cw_multi_test::Executor;
    use streamswap_types::controller::{CreatePool, PoolConfig};
    use streamswap_types::stream::ExecuteMsg as StreamSwapExecuteMsg;
    use streamswap_types::stream::QueryMsg as StreamSwapQueryMsg;
    use streamswap_types::stream::Status;
    use streamswap_types::stream::StreamResponse;

    #[test]
    fn pool_creation() {
        let Suite {
            mut app,
            test_accounts,
            stream_swap_code_id,
            stream_swap_controller_code_id,
            vesting_code_id,
        } = SuiteBuilder::default().build();

        let start_time = app.block_info().time.plus_seconds(1_000_000);
        let end_time = app.block_info().time.plus_seconds(5_000_000);
        let bootstrapping_start_time = app.block_info().time.plus_seconds(500_000);

        let in_denom = "in_denom";
        let out_supply = 1_000_000u128;
        let out_denom = "out_denom";
        let out_clp_amount = 200_000u128;
        let pool_creation_fee: Coin = coin(1000000, "fee_denom");

        let subs1_token = Coin::new(1_000_000_000, in_denom);
        let subs2_token = Coin::new(3_000_000_000, in_denom);
        let in_supply = 4_000_000_000u128;

        let stream_creation_fee = coin(100, "fee_denom");

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
            coin(out_supply, out_denom),
            in_denom,
            bootstrapping_start_time,
            start_time,
            end_time,
        )
        .pool_config(PoolConfig::ConcentratedLiquidity {
            out_amount_clp: out_clp_amount.into(),
        })
        .build();
        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[
                    pool_creation_fee,
                    coin(1_000_000, out_denom),
                    coin(out_clp_amount, out_denom),
                    stream_creation_fee,
                ],
            )
            .unwrap();
        let stream_swap_contract_address: String = get_contract_address_from_res(res);

        // First Subscription
        let subscribe_msg = StreamSwapExecuteMsg::Subscribe {};
        app.update_block(|b| b.time = start_time.plus_seconds(100));
        app.execute_contract(
            test_accounts.subscriber_1.clone(),
            Addr::unchecked(stream_swap_contract_address.clone()),
            &subscribe_msg,
            &[subs1_token],
        )
        .unwrap();

        // Second Subscription
        app.update_block(|b| b.time = start_time.plus_seconds(100_000));
        app.execute_contract(
            test_accounts.subscriber_2.clone(),
            Addr::unchecked(stream_swap_contract_address.clone()),
            &subscribe_msg,
            &[subs2_token],
        )
        .unwrap();

        // finalize stream
        app.update_block(|b| {
            b.time = end_time.plus_seconds(100_000);
        });
        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &StreamSwapExecuteMsg::FinalizeStream {
                    new_treasury: None,
                    create_pool: Some(CreatePool::ConcentratedLiquidity {
                        lower_tick: 500,
                        upper_tick: 100000,
                        tick_spacing: 100,
                        spread_factor: "0.01".to_string(),
                    }),
                    salt: None,
                },
                &[],
            )
            .unwrap();

        let res_swap_fee = get_wasm_attribute_with_key(res.clone(), "swap_fee".to_string());
        let res_creators_revenue =
            get_wasm_attribute_with_key(res.clone(), "creators_revenue".to_string());

        // exit rate is %1
        let swap_fee = in_supply / 100;
        assert_eq!(res_swap_fee, swap_fee.to_string());
        // last creator revenue = spent_in - swap_fee - in_clp;
        let creators_revenue = in_supply - swap_fee;

        let pool_out_ratio = out_clp_amount as f64 / out_supply as f64;

        let creators_revenue_after_pool_creation =
            creators_revenue - (pool_out_ratio * creators_revenue as f64) as u128;
        assert_eq!(
            res_creators_revenue,
            creators_revenue_after_pool_creation.to_string()
        );
    }
    #[test]
    fn cancel_stream_out_clp_returned() {
        let Suite {
            mut app,
            test_accounts,
            stream_swap_code_id,
            stream_swap_controller_code_id,
            vesting_code_id,
        } = SuiteBuilder::default().build();

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

        let start_time = app.block_info().time.plus_seconds(100);
        let end_time = app.block_info().time.plus_seconds(200);
        let bootstrapping_start_time = app.block_info().time.plus_seconds(50);
        let out_supply = 1_000_000u128;
        // pool amount is %20 of out_supply
        let out_clp_amount = 200_000u128;
        let out_denom = "out_denom";

        let out_coin = coin(out_supply, out_denom);
        let pool_out_coin = coin(out_clp_amount, out_denom);
        let pool_creation_fee = coin(1000000, "fee_denom");
        let stream_creation_fee = coin(100, "fee_denom");
        let in_denom = "in_denom";

        let create_stream_msg = CreateStreamMsgBuilder::new(
            "stream",
            test_accounts.creator_1.as_ref(),
            out_coin.clone(),
            in_denom,
            bootstrapping_start_time,
            start_time,
            end_time,
        )
        .threshold(Uint256::from(100u128))
        .pool_config(PoolConfig::ConcentratedLiquidity {
            out_amount_clp: out_clp_amount.into(),
        })
        .build();

        let _res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[
                    pool_creation_fee.clone(),
                    pool_out_coin.clone(),
                    out_coin,
                    stream_creation_fee,
                ],
            )
            .unwrap();

        let stream_swap_contract_address = get_contract_address_from_res(_res);

        app.set_block(BlockInfo {
            time: start_time.plus_seconds(20),
            height: 2,
            chain_id: "test".to_string(),
        });

        let subscribe_msg = StreamSwapExecuteMsg::Subscribe {};

        let _res = app
            .execute_contract(
                test_accounts.subscriber_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &subscribe_msg,
                &[coin(100, "in_denom")],
            )
            .unwrap();

        let cancel_stream_msg = StreamSwapExecuteMsg::CancelStream {};

        let res = app
            .execute_contract(
                test_accounts.admin.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &cancel_stream_msg,
                &[],
            )
            .unwrap();

        let stream_status = get_wasm_attribute_with_key(res.clone(), "status".to_string());
        let fund_transfer = get_funds_from_res(res.clone());

        assert_eq!(stream_status, "cancelled".to_string());
        // clp amount should be added
        let res_pool_amount = coin(out_clp_amount, out_denom);
        let res_refund_out_amount = coin(out_supply, out_denom);

        assert_eq!(
            fund_transfer,
            vec![
                (test_accounts.creator_1.to_string(), res_refund_out_amount),
                (test_accounts.creator_1.to_string(), pool_creation_fee),
                (test_accounts.creator_1.to_string(), res_pool_amount),
            ]
        );
    }
    // #[test]
    // fn cancel_stream_with_threshold_pool_clp_refund() {
    //     let Suite {
    //         mut app,
    //         test_accounts,
    //         stream_swap_code_id,
    //         stream_swap_controller_code_id,
    //         vesting_code_id,
    //     } = SuiteBuilder::default().build();

    //     let msg = get_controller_inst_msg(stream_swap_code_id, vesting_code_id, &test_accounts);
    //     let controller_address = app
    //         .instantiate_contract(
    //             stream_swap_controller_code_id,
    //             test_accounts.admin.clone(),
    //             &msg,
    //             &[],
    //             "Controller".to_string(),
    //             None,
    //         )
    //         .unwrap();

    //     let start_time = app.block_info().time.plus_seconds(100);
    //     let end_time = app.block_info().time.plus_seconds(200);
    //     let bootstrapping_start_time = app.block_info().time.plus_seconds(50);
    //     let out_supply = 1_000_000u128;
    //     // pool amount is %20 of out_supply
    //     let out_clp_amount = 200_000u128;
    //     let out_denom = "out_denom";

    //     let out_coin = coin(out_supply, out_denom);
    //     let pool_out_coin = coin(out_clp_amount, out_denom);
    //     let pool_creation_fee = coin(1000000, "fee_denom");
    //     let stream_creation_fee = coin(100, "fee_denom");
    //     let in_denom = "in_denom";

    //     let create_stream_msg = CreateStreamMsgBuilder::new(
    //         "stream",
    //         test_accounts.creator_1.as_ref(),
    //         out_coin.clone(),
    //         in_denom,
    //         bootstrapping_start_time,
    //         start_time,
    //         end_time,
    //     )
    //     .threshold(Uint256::from(100u128))
    //     .pool_config(PoolConfig::ConcentratedLiquidity {
    //         out_amount_clp: out_clp_amount.into(),
    //     })
    //     .build();

    //     let _res = app
    //         .execute_contract(
    //             test_accounts.creator_1.clone(),
    //             controller_address.clone(),
    //             &create_stream_msg,
    //             &[
    //                 pool_creation_fee.clone(),
    //                 pool_out_coin.clone(),
    //                 out_coin.clone(),
    //                 stream_creation_fee,
    //             ],
    //         )
    //         .unwrap();

    //     let stream_swap_contract_address = get_contract_address_from_res(_res);

    //     app.set_block(BlockInfo {
    //         time: start_time.plus_seconds(20),
    //         height: 2,
    //         chain_id: "test".to_string(),
    //     });

    //     let subscribe_msg = StreamSwapExecuteMsg::Subscribe {};

    //     let _res = app
    //         .execute_contract(
    //             test_accounts.subscriber_1.clone(),
    //             Addr::unchecked(stream_swap_contract_address.clone()),
    //             &subscribe_msg,
    //             &[coin(100 - 1, "in_denom")],
    //         )
    //         .unwrap();

    //     // Set time to end time
    //     app.set_block(BlockInfo {
    //         time: end_time.plus_seconds(20),
    //         height: 2,
    //         chain_id: "test".to_string(),
    //     });

    //     // Try finalizing stream should fail as threshold is not met
    //     let finalize_stream_msg = StreamSwapExecuteMsg::FinalizeStream {
    //         new_treasury: None,
    //         create_pool: None,
    //         salt: None,
    //     };
    //     let _err = app
    //         .execute_contract(
    //             test_accounts.creator_1.clone(),
    //             Addr::unchecked(stream_swap_contract_address.clone()),
    //             &finalize_stream_msg,
    //             &[],
    //         )
    //         .unwrap_err();
    //     // Threshold cancel stream
    //     let cancel_stream_msg = StreamSwapExecuteMsg::CancelStreamWithThreshold {};
    //     let res = app
    //         .execute_contract(
    //             test_accounts.creator_1.clone(),
    //             Addr::unchecked(stream_swap_contract_address.clone()),
    //             &cancel_stream_msg,
    //             &[],
    //         )
    //         .unwrap();
    //     let res_funds = get_funds_from_res(res.clone());
    //     assert_eq!(
    //         res_funds,
    //         vec![
    //             (test_accounts.creator_1.to_string(), out_coin),
    //             (test_accounts.creator_1.to_string(), pool_creation_fee),
    //             (test_accounts.creator_1.to_string(), pool_out_coin),
    //         ]
    //     );
    // }

    #[test]
    fn cancel_stream_stream_admin() {
        let Suite {
            mut app,
            test_accounts,
            stream_swap_code_id,
            stream_swap_controller_code_id,
            vesting_code_id,
        } = SuiteBuilder::default().build();

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

        let start_time = app.block_info().time.plus_seconds(100);
        let end_time = app.block_info().time.plus_seconds(200);
        let bootstrapping_start_time = app.block_info().time.plus_seconds(50);
        let out_supply = 1_000_000u128;
        // pool amount is %20 of out_supply
        let out_clp_amount = 200_000u128;
        let out_denom = "out_denom";

        let out_coin = coin(out_supply, out_denom);
        let pool_out_coin = coin(out_clp_amount, out_denom);
        let pool_creation_fee = coin(1000000, "fee_denom");
        let stream_creation_fee = coin(100, "fee_denom");
        let in_denom = "in_denom";

        let create_stream_msg = CreateStreamMsgBuilder::new(
            "stream",
            test_accounts.creator_1.as_ref(),
            out_coin.clone(),
            in_denom,
            bootstrapping_start_time,
            start_time,
            end_time,
        )
        .threshold(Uint256::from(100u128))
        .pool_config(PoolConfig::ConcentratedLiquidity {
            out_amount_clp: out_clp_amount.into(),
        })
        .build();

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[
                    pool_creation_fee.clone(),
                    pool_out_coin.clone(),
                    out_coin.clone(),
                    stream_creation_fee,
                ],
            )
            .unwrap();

        // Stream is started at waiting status
        let stream_swap_contract_address = get_contract_address_from_res(res);
        // Query stream status
        let query_res: StreamResponse = app
            .wrap()
            .query_wasm_smart(
                &stream_swap_contract_address,
                &StreamSwapQueryMsg::Stream {},
            )
            .unwrap();
        assert_eq!(query_res.status, Status::Waiting);

        // Execute cancel stream with stream admin
        let cancel_stream_msg = StreamSwapExecuteMsg::StreamAdminCancel {};
        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &cancel_stream_msg,
                &[],
            )
            .unwrap();
        let res_funds = get_funds_from_res(res.clone());
        assert_eq!(
            res_funds,
            vec![
                (test_accounts.creator_1.to_string(), out_coin),
                (test_accounts.creator_1.to_string(), pool_creation_fee),
                (test_accounts.creator_1.to_string(), pool_out_coin),
            ]
        );
    }
}
