#[cfg(test)]
mod exit_stream {
    use crate::helpers::mock_messages::CreateStreamMsgBuilder;
    use crate::helpers::suite::SuiteBuilder;
    use crate::helpers::utils::get_wasm_attribute_with_key;
    use crate::helpers::{
        mock_messages::get_controller_inst_msg,
        suite::Suite,
        utils::{get_contract_address_from_res, get_funds_from_res},
    };
    use cosmwasm_std::{coin, Addr, Binary, BlockInfo, Uint256};
    use cw_multi_test::Executor;
    use streamswap_stream::ContractError;
    use streamswap_types::stream::ExecuteMsg as StreamSwapExecuteMsg;

    #[test]
    fn exit_stream() {
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
}
