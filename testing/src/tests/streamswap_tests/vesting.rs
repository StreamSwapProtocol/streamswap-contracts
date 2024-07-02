#[cfg(test)]
mod vesting_tests {
    use crate::helpers::mock_messages::{get_create_stream_msg, get_factory_inst_msg};
    use crate::helpers::setup::{setup, SetupResponse};
    use crate::helpers::utils::{get_contract_address_from_res, get_funds_from_res};
    use cosmwasm_std::{coin, Addr, Api, Binary, BlockInfo, Coin, Uint128};
    use cw_multi_test::Executor;
    use cw_vesting::msg::InstantiateMsg as VestingInstantiateMsg;
    use cw_vesting::vesting::Schedule;
    use cw_vesting::UncheckedDenom;
    use streamswap_stream::msg::StreamResponse;
    use streamswap_stream::state::Status;
    use streamswap_stream::{
        msg::ExecuteMsg as StreamSwapExecuteMsg, msg::QueryMsg as StreamSwapQueryMsg, ContractError,
    };

    #[test]
    fn test_vesting() {
        let SetupResponse {
            mut app,
            test_accounts,
            stream_swap_code_id,
            stream_swap_factory_code_id,
            vesting_code_id,
        } = setup();
        let start_time = app.block_info().time.plus_seconds(100).into();
        let end_time = app.block_info().time.plus_seconds(200).into();
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

        let vesting_msg = VestingInstantiateMsg {
            owner: None,
            recipient: test_accounts.subscriber_1.to_string(),
            title: "Streamswap vesting".to_string(),
            description: None,
            total: Uint128::new(0),
            denom: UncheckedDenom::Native("out_denom".to_string()),
            schedule: Schedule::SaturatingLinear,
            start_time: None,
            vesting_duration_seconds: 150,
            unbonding_duration_seconds: 0,
        };
        let create_stream_msg = get_create_stream_msg(
            &"Stream Swap tests".to_string(),
            None,
            &test_accounts.creator_1.to_string(),
            coin(1_000_000, "out_denom"),
            "in_denom",
            start_time,
            end_time,
            None,
            None,
            Some(vesting_msg),
        );
        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                factory_address.clone(),
                &create_stream_msg,
                &[coin(100, "fee_denom"), coin(1_000_000, "out_denom")],
            )
            .unwrap();
        let stream_swap_contract_address: String = get_contract_address_from_res(res);
        let subscribe_msg = StreamSwapExecuteMsg::Subscribe {
            operator_target: None,
            operator: None,
        };
        app.update_block(|b| b.time = start_time);
        // First subscription
        let _res = app
            .execute_contract(
                test_accounts.subscriber_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &subscribe_msg,
                &[coin(200, "in_denom")],
            )
            .unwrap();

        // update block time
        app.update_block(|b| b.time = end_time.plus_seconds(5));

        let finalized_msg = StreamSwapExecuteMsg::FinalizeStream { new_treasury: None };
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

        assert_eq!(stream.status, Status::Finalized);

        // no salt expect error
        let exit_msg = StreamSwapExecuteMsg::ExitStream {
            operator_target: None,
            salt: None,
        };
        let res = app
            .execute_contract(
                test_accounts.subscriber_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &exit_msg,
                &[],
            )
            .unwrap_err();

        // sub1 exists
        let exit_msg = StreamSwapExecuteMsg::ExitStream {
            operator_target: None,
            salt: Some(Binary::from_base64("salt").unwrap()),
        };
        let addr = app
            .api()
            .addr_validate(test_accounts.subscriber_1.to_string().as_str())
            .unwrap();
        let res = app
            .execute_contract(
                test_accounts.subscriber_1.clone(),
                Addr::unchecked(stream_swap_contract_address.clone()),
                &exit_msg,
                &[],
            )
            .unwrap();
    }
}
