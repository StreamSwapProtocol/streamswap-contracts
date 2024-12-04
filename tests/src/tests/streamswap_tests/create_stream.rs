#[cfg(test)]
mod create_stream_tests {
    use crate::helpers::mock_messages::CreateStreamMsgBuilder;
    use crate::helpers::suite::SuiteBuilder;
    use crate::helpers::utils::get_wasm_attribute_with_key;
    use crate::helpers::{mock_messages::get_controller_inst_msg, suite::Suite};
    use cosmwasm_std::{coin, Uint256};
    use cw_multi_test::Executor;
    use streamswap_controller::error::ContractError as ControllerError;
    use streamswap_controller::error::ContractError::InvalidToSVersion;
    use streamswap_stream::ContractError as StreamSwapError;
    use streamswap_types::controller::Params as ControllerParams;
    use streamswap_types::controller::QueryMsg;
    use streamswap_utils::payment_checker::CustomPaymentError;

    #[test]
    fn create_stream_failed_name_url_checks() {
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
        // Failed name checks
        // Name too short
        let create_stream_msg = CreateStreamMsgBuilder::new(
            "s",
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
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
                &[coin(100, "fee_denom"), coin(100, "out_denom")],
            )
            .unwrap_err();

        let err = res.source().unwrap().source().unwrap();
        let error = err.downcast_ref::<StreamSwapError>().unwrap();
        assert_eq!(*error, StreamSwapError::StreamNameTooShort {});
        // Name too long
        let long_name = "a".repeat(65);
        let create_stream_msg = CreateStreamMsgBuilder::new(
            &long_name,
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            app.block_info().time.plus_seconds(100),
            app.block_info().time.plus_seconds(200),
        )
        .build();

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(100, "out_denom")],
            )
            .unwrap_err();

        let err = res.source().unwrap().source().unwrap();
        let error = err.downcast_ref::<StreamSwapError>().unwrap();
        assert_eq!(*error, StreamSwapError::StreamNameTooLong {});

        // Invalid name
        let create_stream_msg = CreateStreamMsgBuilder::new(
            "abc~ß",
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            app.block_info().time.plus_seconds(100),
            app.block_info().time.plus_seconds(200),
        )
        .build();

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(100, "out_denom")],
            )
            .unwrap_err();

        let err = res.source().unwrap().source().unwrap();
        let error = err.downcast_ref::<StreamSwapError>().unwrap();
        assert_eq!(*error, StreamSwapError::InvalidStreamName {});

        // Failed url checks
        // URL too short
        let create_stream_msg = CreateStreamMsgBuilder::new(
            "stream",
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            app.block_info().time.plus_seconds(100),
            app.block_info().time.plus_seconds(200),
        )
        .url("a".to_string())
        .build();

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(100, "out_denom")],
            )
            .unwrap_err();

        let err = res.source().unwrap().source().unwrap();
        let error = err.downcast_ref::<StreamSwapError>().unwrap();

        assert_eq!(*error, StreamSwapError::StreamUrlTooShort {});

        // URL too long
        let long_url = "a".repeat(256);
        let create_stream_msg = CreateStreamMsgBuilder::new(
            "stream",
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            app.block_info().time.plus_seconds(100),
            app.block_info().time.plus_seconds(200),
        )
        .url(long_url)
        .threshold(Uint256::from(100u128))
        .build();

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(100, "out_denom")],
            )
            .unwrap_err();

        let err = res.source().unwrap().source().unwrap();
        let error = err.downcast_ref::<StreamSwapError>().unwrap();
        assert_eq!(*error, StreamSwapError::StreamUrlTooLong {});
    }

    #[test]
    fn create_stream_failed_fund_checks() {
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

        // Non permissioned in denom
        let create_stream_msg = CreateStreamMsgBuilder::new(
            "stream",
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "invalid_in_denom",
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
                &[coin(100, "fee_denom"), coin(100, "out_denom")],
            )
            .unwrap_err();
        let err = res.source().unwrap();
        let error = err.downcast_ref::<ControllerError>().unwrap();
        assert_eq!(*error, ControllerError::InDenomIsNotAccepted {});

        // Same in and out denom
        let create_stream_msg = CreateStreamMsgBuilder::new(
            "stream",
            test_accounts.creator_1.as_ref(),
            coin(100, "in_denom"),
            "in_denom",
            bootstrapping_start_time,
            app.block_info().time.plus_seconds(100),
            app.block_info().time.plus_seconds(200),
        )
        .build();
        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(100, "in_denom")],
            )
            .unwrap_err();

        let err = res.source().unwrap().source().unwrap();
        let error = err.downcast_ref::<StreamSwapError>().unwrap();
        assert_eq!(*error, StreamSwapError::SameDenomOnEachSide {});

        // Zero out supply
        let create_stream_msg = CreateStreamMsgBuilder::new(
            "stream",
            test_accounts.creator_1.as_ref(),
            coin(0, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            app.block_info().time.plus_seconds(100),
            app.block_info().time.plus_seconds(200),
        )
        .threshold(Uint256::from(100u128))
        .build();

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom")],
            )
            .unwrap_err();
        let err = res.source().unwrap();
        let error = err.downcast_ref::<ControllerError>().unwrap();
        assert_eq!(*error, ControllerError::ZeroOutSupply {});

        // No funds sent
        let create_stream_msg = CreateStreamMsgBuilder::new(
            "stream",
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            app.block_info().time.plus_seconds(100),
            app.block_info().time.plus_seconds(200),
        )
        .build();

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom")],
            )
            .unwrap_err();

        let err = res.source().unwrap();
        let error = err.downcast_ref::<ControllerError>().unwrap();
        assert_eq!(
            *error,
            ControllerError::CustomPayment(CustomPaymentError::InsufficientFunds {
                expected: [coin(100, "fee_denom"), coin(100, "out_denom")].to_vec(),
                actual: [coin(100, "fee_denom")].to_vec()
            })
        );

        // Insufficient fee
        let create_stream_msg = CreateStreamMsgBuilder::new(
            "stream",
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            app.block_info().time.plus_seconds(100),
            app.block_info().time.plus_seconds(200),
        )
        .build();

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[coin(99, "fee_denom"), coin(100, "out_denom")],
            )
            .unwrap_err();

        let err = res.source().unwrap();
        let error = err.downcast_ref::<ControllerError>().unwrap();
        assert_eq!(
            *error,
            ControllerError::CustomPayment(CustomPaymentError::InsufficientFunds {
                expected: [coin(100, "fee_denom"), coin(100, "out_denom")].to_vec(),
                actual: [coin(99, "fee_denom"), coin(100, "out_denom")].to_vec()
            })
        );

        // Extra funds sent
        let create_stream_msg = CreateStreamMsgBuilder::new(
            "stream",
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            app.block_info().time.plus_seconds(100),
            app.block_info().time.plus_seconds(200),
        )
        .build();

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[
                    coin(100, "fee_denom"),
                    coin(100, "out_denom"),
                    coin(100, "wrong_denom"),
                ],
            )
            .unwrap_err();

        let err = res.source().unwrap();
        let error = err.downcast_ref::<ControllerError>().unwrap();
        assert_eq!(
            *error,
            ControllerError::CustomPayment(CustomPaymentError::InsufficientFunds {
                expected: [coin(100, "fee_denom"), coin(100, "out_denom")].to_vec(),
                actual: [
                    coin(100, "fee_denom"),
                    coin(100, "out_denom"),
                    coin(100, "wrong_denom")
                ]
                .to_vec()
            })
        );

        // Threshold zero
        let create_stream_msg = CreateStreamMsgBuilder::new(
            "stream",
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            app.block_info().time.plus_seconds(100),
            app.block_info().time.plus_seconds(200),
        )
        .threshold(Uint256::from(0u128))
        .build();

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(100, "out_denom")],
            )
            .unwrap_err();

        let err = res.source().unwrap().source().unwrap();
        let error = err.downcast_ref::<StreamSwapError>().unwrap();
        assert_eq!(*error, StreamSwapError::InvalidThreshold {});
    }

    #[test]
    fn create_stream_failed_duration_checks() {
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

        let controller_params: ControllerParams = app
            .wrap()
            .query_wasm_smart(controller_address.clone(), &QueryMsg::Params {})
            .unwrap();

        // Waiting duration too short
        let bootstart_time = app
            .block_info()
            .time
            .plus_seconds(controller_params.min_waiting_duration - 1);

        let start_time = app.block_info().time.plus_seconds(100);
        let end_time = app.block_info().time.plus_seconds(200);

        let create_stream_msg = CreateStreamMsgBuilder::new(
            "stream",
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstart_time,
            start_time,
            end_time,
        )
        .build();
        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(100, "out_denom")],
            )
            .unwrap_err();
        let err = res.source().unwrap().source().unwrap();
        let error = err.downcast_ref::<StreamSwapError>().unwrap();
        assert_eq!(*error, StreamSwapError::StreamWaitingDurationTooShort {});

        // Stream duration too short
        let bootstart_time = app.block_info().time.plus_seconds(50);
        let start_time = app.block_info().time.plus_seconds(100);
        let end_time = app.block_info().time.plus_seconds(101);

        let create_stream_msg = CreateStreamMsgBuilder::new(
            "stream",
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstart_time,
            start_time,
            end_time,
        )
        .build();

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(100, "out_denom")],
            )
            .unwrap_err();

        let err = res.source().unwrap().source().unwrap();
        let error = err.downcast_ref::<StreamSwapError>().unwrap();
        assert_eq!(*error, StreamSwapError::StreamDurationTooShort {});
    }

    #[test]
    fn create_stream_failed_tos_version() {
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
        .tos_version("invalid".to_string())
        .build();

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(100, "out_denom")],
            )
            .unwrap_err();
        let err = res.source().unwrap();
        let error = err.downcast_ref::<ControllerError>().unwrap();
        assert_eq!(*error, InvalidToSVersion {});
    }

    #[test]
    fn create_stream_happy_path() {
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
            "stream",
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            start_time,
            end_time,
        )
        .threshold(Uint256::from(100u128))
        .build();

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(100, "out_denom")],
            )
            .unwrap();
        // Test contract address created deterministically
        // We predict the address upon creation of the stream and return address via wasm attribute "stream_contract_addr"
        // Wasm releated attributes are stored in the "wasm" event type such as "execute", "instantiate" etc.
        // Inside these attributes the contract to be created is stored in the "_contract_address" key
        // We can use this to verify the contract address created
        let res_contract_addr =
            get_wasm_attribute_with_key(res.clone(), "stream_contract_addr".to_string());
        // Iterate every event and print
        let instantiate_event = res.events.iter().find(|e| e.ty == "instantiate").unwrap();
        let instantiate_contract_addr = instantiate_event
            .attributes
            .iter()
            .find(|a| a.key == "_contract_address")
            .unwrap()
            .value
            .clone();

        assert_eq!(res_contract_addr, instantiate_contract_addr);
        let query_msg = QueryMsg::LastStreamId {};
        let res: u32 = app
            .wrap()
            .query_wasm_smart(controller_address, &query_msg)
            .unwrap();
        assert_eq!(res, 1);
    }
}
