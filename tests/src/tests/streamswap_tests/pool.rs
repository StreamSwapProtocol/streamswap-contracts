#[cfg(test)]
mod pool {
    use crate::helpers::mock_messages::{get_controller_inst_msg, get_create_stream_msg};
    use crate::helpers::suite::{Suite, SuiteBuilder};
    use crate::helpers::utils::{get_contract_address_from_res, get_wasm_attribute_with_key};
    use cosmwasm_std::{coin, Addr, Coin};
    use cw_multi_test::Executor;
    use osmosis_std::types::osmosis::concentratedliquidity::poolmodel::concentrated::v1beta1::MsgCreateConcentratedPool;
    use std::ops::Div;
    use streamswap_types::controller::CreatePool;
    use streamswap_types::stream::ExecuteMsg;

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
        // %20 of out_supply will go to pool
        let out_clp_amount = 200_000u128;
        // this is mocked by querier at test_helpers.rs
        // pool_creation_fee = 1000000;
        // pool_creation_denom = "fee_denom";
        let stream_creation_denom = "fee_denom";
        let stream_creation_fee = 100;

        let subs1_token = Coin::new(1_000_000_000, in_denom);
        let subs2_token = Coin::new(3_000_000_000, in_denom);
        let in_supply = 4_000_000_000u128;

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
        let create_stream_msg = get_create_stream_msg(
            "Stream Swap tests",
            None,
            test_accounts.creator_1.as_ref(),
            coin(out_supply, out_denom),
            in_denom,
            bootstrapping_start_time,
            start_time,
            end_time,
            None,
            Some(CreatePool {
                out_amount_clp: out_clp_amount.into(),
                msg_create_pool: MsgCreateConcentratedPool {
                    sender: test_accounts.creator_1.to_string(),
                    denom0: in_denom.to_string(),
                    denom1: out_denom.to_string(),
                    tick_spacing: 100,
                    spread_factor: "10".to_string(),
                },
            }),
            None,
        );
        let res = app
            .execute_contract(
                test_accounts.creator_1.clone(),
                controller_address.clone(),
                &create_stream_msg,
                &[
                    coin(stream_creation_fee, stream_creation_denom),
                    coin(1_000_000, out_denom),
                ],
            )
            .unwrap();
        let stream_swap_contract_address: String = get_contract_address_from_res(res);

        // First Subscription
        let subscribe_msg = ExecuteMsg::Subscribe {};
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
                &ExecuteMsg::FinalizeStream { new_treasury: None },
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
        let expected_creators_revenue =
            (in_supply - swap_fee - (out_clp_amount / out_supply * in_supply)).to_string();
        assert_eq!(res_creators_revenue, expected_creators_revenue);
    }
}
