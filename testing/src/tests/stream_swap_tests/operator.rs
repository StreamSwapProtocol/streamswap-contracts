#[cfg(test)]
mod operator_tests {

    use crate::helpers::utils::get_contract_address_from_res;
    use crate::helpers::{
        mock_messages::{get_create_stream_msg, get_factory_inst_msg},
        setup::{setup, SetupResponse},
    };
    use cosmwasm_std::{coin, Addr, BlockInfo, Uint128};
    use cw_multi_test::Executor;
    use cw_streamswap::{
        msg::{
            ExecuteMsg as StreamSwapExecuteMsg, PositionResponse, QueryMsg as StreamSwapQueryMsg,
            StreamResponse,
        },
        ContractError as StreamSwapError,
    };
    use cw_streamswap_factory::msg::QueryMsg as FactoryQueryMsg;
    #[test]
    fn test_operator_first_subscribe() {
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
            Some("https://sample.url".to_string()),
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
                &[coin(100, "fee_denom"), coin(1_000_000, "out_denom")],
            )
            .unwrap();
        let stream_swap_contract_address: String = get_contract_address_from_res(res);

        let test_msg = StreamSwapExecuteMsg::Subscribe {
            operator_target: Some("subscriber_1".to_string()),
            operator: None,
        };
        app.set_block(BlockInfo {
            height: 1_100,
            time: start_time.minus_seconds(100),
            chain_id: "test".to_string(),
        });
        // Target a subscription that does not exist
        let res = app
            .execute_contract(
                test_accounts.wrong_user.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &test_msg,
                &[coin(100, "in_denom")],
            )
            .unwrap_err();
        let err = res.source().unwrap();
        let error = err.downcast_ref::<StreamSwapError>().unwrap();
        assert_eq!(*error, StreamSwapError::Unauthorized {});

        // Random cannot make the first subscription on behalf of user even if defined as operator in message
        let msg = StreamSwapExecuteMsg::Subscribe {
            operator_target: Some("subscriber_1".to_string()),
            operator: Some("wrong_user".to_string()),
        };
        let res = app
            .execute_contract(
                test_accounts.wrong_user.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &msg,
                &[coin(100, "in_denom")],
            )
            .unwrap_err();
        let err = res.source().unwrap();
        let error = err.downcast_ref::<StreamSwapError>().unwrap();
        assert_eq!(*error, StreamSwapError::Unauthorized {});
    }

    // test_operator_after_subscribe

    #[test]
    fn test_operator_after_subscribe() {
        let SetupResponse {
            mut app,
            test_accounts,
            stream_swap_code_id,
            stream_swap_factory_code_id,
        } = setup();
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
            "stream",
            None,
            &test_accounts.creator.to_string(),
            coin(100, "out_denom"),
            "in_denom",
            app.block_info().time.plus_seconds(100).into(),
            app.block_info().time.plus_seconds(200).into(),
            Some(Uint128::from(100u128)),
        );

        let res = app
            .execute_contract(
                test_accounts.creator.clone(),
                factory_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(1_000_000, "out_denom")],
            )
            .unwrap();

        let stream_swap_contract_address: String = get_contract_address_from_res(res);
        // Set operator as subscriber_2
        let msg = StreamSwapExecuteMsg::Subscribe {
            operator_target: None,
            operator: Some("subscriber_2".to_string()),
        };
        let _res = app
            .execute_contract(
                test_accounts.subscriber.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &msg,
                &[coin(100, "in_denom")],
            )
            .unwrap();
        // random targeting subscriber_1 it should fail
        let msg = StreamSwapExecuteMsg::Subscribe {
            operator_target: Some("subscriber_1".to_string()),
            operator: None,
        };
        let res = app
            .execute_contract(
                test_accounts.wrong_user.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &msg,
                &[coin(100, "in_denom")],
            )
            .unwrap_err();
        let err = res.source().unwrap();
        let error = err.downcast_ref::<StreamSwapError>().unwrap();
        assert_eq!(*error, StreamSwapError::Unauthorized {});
    }

    // test_update_operator

    #[test]
    fn test_update_operator() {
        let SetupResponse {
            mut app,
            test_accounts,
            stream_swap_code_id,
            stream_swap_factory_code_id,
        } = setup();

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
            "stream",
            None,
            &test_accounts.creator.to_string(),
            coin(100, "out_denom"),
            "in_denom",
            app.block_info().time.plus_seconds(100).into(),
            app.block_info().time.plus_seconds(200).into(),
            Some(Uint128::from(100u128)),
        );

        let res = app
            .execute_contract(
                test_accounts.creator.clone(),
                factory_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(1_000_000, "out_denom")],
            )
            .unwrap();
        let stream_swap_contract_address: String = get_contract_address_from_res(res);

        // Set operator as subscriber_2
        let msg = StreamSwapExecuteMsg::Subscribe {
            operator_target: None,
            operator: Some("subscriber_2".to_string()),
        };
        let _res = app
            .execute_contract(
                test_accounts.subscriber.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &msg,
                &[coin(100, "in_denom")],
            )
            .unwrap();

        // test update operator to subscriber_1 by random
        let msg = StreamSwapExecuteMsg::UpdateOperator {
            new_operator: Some("subscriber_1".to_string()),
        };
        let _res = app
            .execute_contract(
                test_accounts.wrong_user.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &msg,
                &[coin(100, "in_denom")],
            )
            .unwrap_err();

        // test update operator to subscibe 1
        let msg = StreamSwapExecuteMsg::UpdateOperator {
            new_operator: Some("subscriber_1".to_string()),
        };
        let _res = app
            .execute_contract(
                test_accounts.subscriber.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &msg,
                &[coin(100, "in_denom")],
            )
            .unwrap();
        // withdraw
        let subscriber_1_balance_before = app
            .wrap()
            .query_balance(test_accounts.subscriber.clone(), "in_denom")
            .unwrap();

        let _res = app
            .execute_contract(
                test_accounts.subscriber.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &StreamSwapExecuteMsg::Withdraw {
                    cap: Some(Uint128::new(100)),
                    operator_target: None,
                },
                &[],
            )
            .unwrap();
        let subscriber_1_balance_after = app
            .wrap()
            .query_balance(test_accounts.subscriber.clone(), "in_denom")
            .unwrap();
        assert_eq!(
            subscriber_1_balance_after
                .amount
                .checked_sub(subscriber_1_balance_before.amount)
                .unwrap(),
            Uint128::new(100)
        );
    }
}
