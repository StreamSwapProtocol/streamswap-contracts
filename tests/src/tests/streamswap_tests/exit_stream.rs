#[cfg(test)]
mod exit_stream {
    use crate::helpers::mock_messages::CreateStreamMsgBuilder;
    use crate::helpers::suite::SuiteBuilder;
    use crate::helpers::{
        mock_messages::get_controller_inst_msg,
        suite::Suite,
        utils::{get_contract_address_from_res, get_funds_from_res},
    };
    use cosmwasm_std::{coin, Addr, BlockInfo, Uint128, Uint256};
    use cw_multi_test::Executor;
    use streamswap_stream::ContractError;
    use streamswap_types::stream::ExecuteMsg as StreamSwapExecuteMsg;

    #[test]
    fn happy_path() {
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

        let create_stream_msg = CreateStreamMsgBuilder::new(
            "stream",
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            start_time,
            end_time,
        )
        .build();

        let _res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(100, "out_denom")],
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
                &[coin(10, "in_denom")],
            )
            .unwrap();

        app.set_block(BlockInfo {
            time: end_time.plus_seconds(20),
            height: 3,
            chain_id: "test".to_string(),
        });

        let exit_msg = StreamSwapExecuteMsg::ExitStream { salt: None };

        let res = app
            .execute_contract(
                test_accounts.subscriber_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &exit_msg,
                &[],
            )
            .unwrap();
    }

    #[test]
    fn exit_stream_threshold_not_reached() {
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
        let threshold = Uint256::from(1000u128);

        let create_stream_msg = CreateStreamMsgBuilder::new(
            "stream",
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            start_time,
            end_time,
        )
        .threshold(threshold)
        .build();

        let _res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(100, "out_denom")],
            )
            .unwrap();

        let stream_swap_contract_address = get_contract_address_from_res(_res);

        app.set_block(BlockInfo {
            time: start_time.plus_seconds(20),
            height: 2,
            chain_id: "test".to_string(),
        });

        // Subscriber 1 subscribes to the stream
        let subscribe_msg = StreamSwapExecuteMsg::Subscribe {};

        let _res = app
            .execute_contract(
                test_accounts.subscriber_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &subscribe_msg,
                &[coin(10, "in_denom")],
            )
            .unwrap();

        // Subscriber 2 subscribes to the stream
        let _res = app
            .execute_contract(
                test_accounts.subscriber_2.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &subscribe_msg,
                &[coin(10, "in_denom")],
            )
            .unwrap();

        app.set_block(BlockInfo {
            time: end_time.plus_seconds(20),
            height: 3,
            chain_id: "test".to_string(),
        });

        // Subscriber 1 exits the stream before its finalized but the threshold is not reached
        let exit_msg = StreamSwapExecuteMsg::ExitStream { salt: None };

        let res = app
            .execute_contract(
                test_accounts.subscriber_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &exit_msg,
                &[],
            )
            .unwrap();

        let funds = get_funds_from_res(res);

        assert_eq!(funds[0].1.amount, Uint128::from(10u128));
        assert_eq!(funds[0].1.denom, "in_denom");
        assert_eq!(funds[0].0, test_accounts.subscriber_1);

        // Creator finalizes the stream after subscriber 1 exits
        let finalize_msg = StreamSwapExecuteMsg::FinalizeStream {
            new_treasury: None,
            create_pool: None,
            salt: None,
        };

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &finalize_msg,
                &[],
            )
            .unwrap();
        // Creator gets full refund
        let funds = get_funds_from_res(res);
        let expected = [(
            test_accounts.creator_1.clone().into_string(),
            coin(100, "out_denom"),
        )];

        assert_eq!(funds, expected);

        // Subscriber 2 exits the stream after its finalized
        let res = app
            .execute_contract(
                test_accounts.subscriber_2.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &exit_msg,
                &[],
            )
            .unwrap();

        let funds = get_funds_from_res(res);

        assert_eq!(funds[0].1.amount, Uint128::from(10u128));
        assert_eq!(funds[0].1.denom, "in_denom");

        // Check balance of the stream contract
        // The stream contract should have no balance
        let balance = app
            .wrap()
            .query_all_balances(stream_swap_contract_address)
            .unwrap();

        assert_eq!(balance.len(), 0);
    }

    #[test]
    fn exit_stream_cancelled() {
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

        let create_stream_msg = CreateStreamMsgBuilder::new(
            "stream",
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            start_time,
            end_time,
        )
        .build();

        let _res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(100, "out_denom")],
            )
            .unwrap();

        let stream_swap_contract_address = get_contract_address_from_res(_res);

        app.set_block(BlockInfo {
            time: start_time.plus_seconds(20),
            height: 2,
            chain_id: "test".to_string(),
        });

        // Subscriber 1 subscribes to the stream
        let subscribe_msg = StreamSwapExecuteMsg::Subscribe {};

        let _res = app
            .execute_contract(
                test_accounts.subscriber_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &subscribe_msg,
                &[coin(10, "in_denom")],
            )
            .unwrap();

        // Subscriber 2 subscribes to the stream
        let _res = app
            .execute_contract(
                test_accounts.subscriber_2.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &subscribe_msg,
                &[coin(10, "in_denom")],
            )
            .unwrap();

        // Protocol admin cancels the stream
        let cancel_msg = StreamSwapExecuteMsg::CancelStream {};

        let res = app
            .execute_contract(
                test_accounts.admin.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &cancel_msg,
                &[],
            )
            .unwrap();
        // Creator gets full refund
        let funds = get_funds_from_res(res);
        let expected = [(
            test_accounts.creator_1.clone().into_string(),
            coin(100, "out_denom"),
        )];

        assert_eq!(funds, expected);

        // Subscriber 1 exits the stream after its cancelled
        let exit_msg = StreamSwapExecuteMsg::ExitStream { salt: None };

        let res = app
            .execute_contract(
                test_accounts.subscriber_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &exit_msg,
                &[],
            )
            .unwrap();

        let funds = get_funds_from_res(res);

        assert_eq!(funds[0].1.amount, Uint128::from(10u128));
        assert_eq!(funds[0].1.denom, "in_denom");
        assert_eq!(funds[0].0, test_accounts.subscriber_1);

        // Check balance of the stream contract
        // The stream contract should still have the remaining balance from subscriber 2
        let balance = app
            .wrap()
            .query_all_balances(stream_swap_contract_address.clone())
            .unwrap();

        assert_eq!(balance.len(), 1);

        // Subscriber 2 exits the stream after its cancelled
        let res = app
            .execute_contract(
                test_accounts.subscriber_2.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &exit_msg,
                &[],
            )
            .unwrap();

        let funds = get_funds_from_res(res);

        assert_eq!(funds[0].1.amount, Uint128::from(10u128));
        assert_eq!(funds[0].1.denom, "in_denom");

        // Check balance of the stream contract
        // The stream contract should have no balance
        let balance = app
            .wrap()
            .query_all_balances(stream_swap_contract_address)
            .unwrap();

        assert_eq!(balance.len(), 0);
    }

    #[test]
    fn exit_without_a_position() {
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

        // Attempting to exit without a position
        let exit_msg = StreamSwapExecuteMsg::ExitStream { salt: None };

        let err = app
            .execute_contract(
                test_accounts.subscriber_2.clone(), // No position for this subscriber
                Addr::unchecked(stream_swap_contract_address.clone()),
                &exit_msg,
                &[],
            )
            .unwrap_err();
    }

    #[test]
    fn attempt_exit_after_already_exited() {
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

        // Instantiate the controller
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

        // Create a stream
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

        // Simulate block time to allow subscription
        app.set_block(BlockInfo {
            height: 1_100,
            time: start_time,
            chain_id: "test".to_string(),
        });

        // Subscriber subscribes to the stream
        let subscribe_msg = StreamSwapExecuteMsg::Subscribe {};
        app.execute_contract(
            test_accounts.subscriber_1.clone(),
            Addr::unchecked(stream_swap_contract_address.clone()),
            &subscribe_msg,
            &[coin(150, "in_denom")],
        )
        .unwrap();

        // Simulate block time for stream execution
        app.set_block(BlockInfo {
            height: 1_200,
            time: end_time.plus_seconds(1),
            chain_id: "test".to_string(),
        });

        // Subscriber exits the stream
        let exit_msg = StreamSwapExecuteMsg::ExitStream { salt: None };
        app.execute_contract(
            test_accounts.subscriber_1.clone(),
            Addr::unchecked(stream_swap_contract_address.clone()),
            &exit_msg,
            &[],
        )
        .unwrap();

        // Attempt to exit again
        let err = app
            .execute_contract(
                test_accounts.subscriber_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &exit_msg,
                &[],
            )
            .unwrap_err();

        assert_eq!(
            err.downcast::<ContractError>().unwrap(),
            ContractError::SubscriberAlreadyExited {}
        );
    }
}
