#[cfg(test)]
mod create_stream_tests {
    use crate::helpers::suite::SuiteBuilder;
    use crate::helpers::utils::get_wasm_attribute_with_key;
    use crate::helpers::{
        mock_messages::{get_create_stream_msg, get_factory_inst_msg},
        suite::Suite,
    };
    use cosmwasm_std::{coin, Api, Binary, Uint256};
    use cw_multi_test::Executor;
    use streamswap_factory::error::ContractError as FactoryError;
    use streamswap_stream::ContractError as StreamSwapError;
    use streamswap_types::factory::Params as FactoryParams;
    use streamswap_types::factory::QueryMsg;
    use streamswap_types::stream::ThresholdError;
    use streamswap_utils::payment_checker::CustomPaymentError;

    #[test]
    fn create_stream_failed_name_url_checks() {
        let Suite {
            mut app,
            test_accounts,
            stream_swap_code_id,
            stream_swap_factory_code_id,
            vesting_code_id,
        } = SuiteBuilder::default().build();
        let start_time = app.block_info().time.plus_seconds(100);
        let end_time = app.block_info().time.plus_seconds(200);
        let bootstrapping_start_time = app.block_info().time.plus_seconds(50);

        let msg = get_factory_inst_msg(stream_swap_code_id, vesting_code_id, &test_accounts);
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
        // Failed name checks
        // Name too short
        let create_stream_msg = get_create_stream_msg(
            "s",
            None,
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            start_time,
            end_time,
            None,
            None,
            None,
        );

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                factory_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(100, "out_denom")],
            )
            .unwrap_err();

        let err = res.source().unwrap().source().unwrap();
        let error = err.downcast_ref::<StreamSwapError>().unwrap();
        assert_eq!(*error, StreamSwapError::StreamNameTooShort {});
        // Name too long
        let long_name = "a".repeat(65);
        let create_stream_msg = get_create_stream_msg(
            &long_name,
            None,
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            app.block_info().time.plus_seconds(100),
            app.block_info().time.plus_seconds(200),
            None,
            None,
            None,
        );

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                factory_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(100, "out_denom")],
            )
            .unwrap_err();

        let err = res.source().unwrap().source().unwrap();
        let error = err.downcast_ref::<StreamSwapError>().unwrap();
        assert_eq!(*error, StreamSwapError::StreamNameTooLong {});

        // Invalid name
        let create_stream_msg = get_create_stream_msg(
            "abc~ß",
            None,
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            app.block_info().time.plus_seconds(100),
            app.block_info().time.plus_seconds(200),
            None,
            None,
            None,
        );

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                factory_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(100, "out_denom")],
            )
            .unwrap_err();

        let err = res.source().unwrap().source().unwrap();
        let error = err.downcast_ref::<StreamSwapError>().unwrap();
        assert_eq!(*error, StreamSwapError::InvalidStreamName {});

        // Failed url checks
        // URL too short
        let create_stream_msg = get_create_stream_msg(
            "stream",
            Some("a".to_string()),
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            app.block_info().time.plus_seconds(100),
            app.block_info().time.plus_seconds(200),
            None,
            None,
            None,
        );

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                factory_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(100, "out_denom")],
            )
            .unwrap_err();

        let err = res.source().unwrap().source().unwrap();
        let error = err.downcast_ref::<StreamSwapError>().unwrap();

        assert_eq!(*error, StreamSwapError::StreamUrlTooShort {});

        // URL too long
        let long_url = "a".repeat(256);
        let create_stream_msg = get_create_stream_msg(
            "stream",
            Some(long_url),
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            app.block_info().time.plus_seconds(100),
            app.block_info().time.plus_seconds(200),
            Some(Uint256::from(100u128)),
            None,
            None,
        );

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                factory_address.clone(),
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
            stream_swap_factory_code_id,
            vesting_code_id,
        } = SuiteBuilder::default().build();

        let start_time = app.block_info().time.plus_seconds(100);
        let end_time = app.block_info().time.plus_seconds(200);
        let bootstrapping_start_time = app.block_info().time.plus_seconds(50);

        let msg = get_factory_inst_msg(stream_swap_code_id, vesting_code_id, &test_accounts);
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

        // Non permissioned in denom
        let create_stream_msg = get_create_stream_msg(
            "stream",
            None,
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "invalid_in_denom",
            bootstrapping_start_time,
            start_time,
            end_time,
            None,
            None,
            None,
        );
        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                factory_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(100, "out_denom")],
            )
            .unwrap_err();
        let err = res.source().unwrap();
        let error = err.downcast_ref::<FactoryError>().unwrap();
        assert_eq!(*error, FactoryError::InDenomIsNotAccepted {});

        // Same in and out denom
        let create_stream_msg = get_create_stream_msg(
            "stream",
            None,
            test_accounts.creator_1.as_ref(),
            coin(100, "in_denom"),
            "in_denom",
            bootstrapping_start_time,
            app.block_info().time.plus_seconds(100),
            app.block_info().time.plus_seconds(200),
            None,
            None,
            None,
        );
        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                factory_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(100, "in_denom")],
            )
            .unwrap_err();

        let err = res.source().unwrap().source().unwrap();
        let error = err.downcast_ref::<StreamSwapError>().unwrap();
        assert_eq!(*error, StreamSwapError::SameDenomOnEachSide {});

        // Zero out supply
        let create_stream_msg = get_create_stream_msg(
            "stream",
            None,
            test_accounts.creator_1.as_ref(),
            coin(0, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            app.block_info().time.plus_seconds(100),
            app.block_info().time.plus_seconds(200),
            Some(Uint256::from(100u128)),
            None,
            None,
        );

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                factory_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom")],
            )
            .unwrap_err();
        let err = res.source().unwrap();
        let error = err.downcast_ref::<FactoryError>().unwrap();
        assert_eq!(*error, FactoryError::ZeroOutSupply {});

        // No funds sent
        let create_stream_msg = get_create_stream_msg(
            "stream",
            None,
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            app.block_info().time.plus_seconds(100),
            app.block_info().time.plus_seconds(200),
            None,
            None,
            None,
        );

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                factory_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom")],
            )
            .unwrap_err();

        let err = res.source().unwrap();
        let error = err.downcast_ref::<FactoryError>().unwrap();
        assert_eq!(
            *error,
            FactoryError::CustomPayment(CustomPaymentError::InsufficientFunds {
                expected: [coin(100, "fee_denom"), coin(100, "out_denom")].to_vec(),
                actual: [coin(100, "fee_denom")].to_vec()
            })
        );

        // Insufficient fee
        let create_stream_msg = get_create_stream_msg(
            "stream",
            None,
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            app.block_info().time.plus_seconds(100),
            app.block_info().time.plus_seconds(200),
            None,
            None,
            None,
        );

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                factory_address.clone(),
                &create_stream_msg,
                &[coin(99, "fee_denom"), coin(100, "out_denom")],
            )
            .unwrap_err();

        let err = res.source().unwrap();
        let error = err.downcast_ref::<FactoryError>().unwrap();
        assert_eq!(
            *error,
            FactoryError::CustomPayment(CustomPaymentError::InsufficientFunds {
                expected: [coin(100, "fee_denom"), coin(100, "out_denom")].to_vec(),
                actual: [coin(99, "fee_denom"), coin(100, "out_denom")].to_vec()
            })
        );

        // Extra funds sent
        let create_stream_msg = get_create_stream_msg(
            "stream",
            None,
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            app.block_info().time.plus_seconds(100),
            app.block_info().time.plus_seconds(200),
            None,
            None,
            None,
        );

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                factory_address.clone(),
                &create_stream_msg,
                &[
                    coin(100, "fee_denom"),
                    coin(100, "out_denom"),
                    coin(100, "wrong_denom"),
                ],
            )
            .unwrap_err();

        let err = res.source().unwrap();
        let error = err.downcast_ref::<FactoryError>().unwrap();
        assert_eq!(
            *error,
            FactoryError::CustomPayment(CustomPaymentError::InsufficientFunds {
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
        let create_stream_msg = get_create_stream_msg(
            "stream",
            None,
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            app.block_info().time.plus_seconds(100),
            app.block_info().time.plus_seconds(200),
            Some(Uint256::from(0u128)),
            None,
            None,
        );

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                factory_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(100, "out_denom")],
            )
            .unwrap_err();

        let err = res.source().unwrap().source().unwrap();
        let error = err.downcast_ref::<StreamSwapError>().unwrap();
        assert_eq!(
            *error,
            StreamSwapError::ThresholdError(ThresholdError::ThresholdZero {})
        );
    }

    #[test]
    fn create_stream_failed_duration_checks() {
        let Suite {
            mut app,
            test_accounts,
            stream_swap_code_id,
            stream_swap_factory_code_id,
            vesting_code_id,
        } = SuiteBuilder::default().build();

        let msg = get_factory_inst_msg(stream_swap_code_id, vesting_code_id, &test_accounts);
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

        let factory_params: FactoryParams = app
            .wrap()
            .query_wasm_smart(factory_address.clone(), &QueryMsg::Params {})
            .unwrap();

        // Waiting duration too short
        let bootstart_time = app
            .block_info()
            .time
            .plus_seconds(factory_params.min_waiting_duration - 1);

        let start_time = app.block_info().time.plus_seconds(100);
        let end_time = app.block_info().time.plus_seconds(200);

        let create_stream_msg = get_create_stream_msg(
            "stream",
            None,
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstart_time,
            start_time,
            end_time,
            None,
            None,
            None,
        );
        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                factory_address.clone(),
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

        let create_stream_msg = get_create_stream_msg(
            "stream",
            None,
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstart_time,
            start_time,
            end_time,
            None,
            None,
            None,
        );

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                factory_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(100, "out_denom")],
            )
            .unwrap_err();

        let err = res.source().unwrap().source().unwrap();
        let error = err.downcast_ref::<StreamSwapError>().unwrap();
        assert_eq!(*error, StreamSwapError::StreamDurationTooShort {});
    }

    #[test]
    fn create_stream_happy_path() {
        let Suite {
            mut app,
            test_accounts,
            stream_swap_code_id,
            stream_swap_factory_code_id,
            vesting_code_id,
        } = SuiteBuilder::default().build();

        let start_time = app.block_info().time.plus_seconds(100);
        let end_time = app.block_info().time.plus_seconds(200);
        let bootstrapping_start_time = app.block_info().time.plus_seconds(50);
        let msg = get_factory_inst_msg(stream_swap_code_id, vesting_code_id, &test_accounts);
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
            "stream",
            None,
            test_accounts.creator_1.as_ref(),
            coin(100, "out_denom"),
            "in_denom",
            bootstrapping_start_time,
            start_time,
            end_time,
            Some(Uint256::from(100u128)),
            None,
            None,
        );

        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                factory_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(100, "out_denom")],
            )
            .unwrap();
        // test contract address created deterministically
        let checksum = app
            .wrap()
            .query_wasm_code_info(stream_swap_code_id)
            .unwrap()
            .checksum;
        let canonical_contract_addr = cosmwasm_std::instantiate2_address(
            checksum.as_slice(),
            &app.api()
                .addr_canonicalize(test_accounts.creator_1.clone().as_str())
                .unwrap(),
            Binary::from_base64("salt").unwrap().as_slice(),
        )
        .unwrap();
        let contract_addr = app.api().addr_humanize(&canonical_contract_addr).unwrap();
        let res_contract_addr =
            get_wasm_attribute_with_key(res, "stream_contract_address".to_string());
        assert_eq!(contract_addr, res_contract_addr);

        // Query stream with id
        let query_msg = QueryMsg::LastStreamId {};
        let res: u32 = app
            .wrap()
            .query_wasm_smart(factory_address, &query_msg)
            .unwrap();
        assert_eq!(res, 1);
    }
}
