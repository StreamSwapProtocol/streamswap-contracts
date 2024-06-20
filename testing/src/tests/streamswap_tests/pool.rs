#[cfg(test)]
mod pool_tests {
    use crate::helpers::mock_messages::{get_create_stream_msg, get_factory_inst_msg};
    use crate::helpers::setup::{setup, SetupResponse};
    use crate::helpers::utils::get_contract_address_from_res;
    use cosmwasm_std::{coin, Addr, Coin};
    use cw_multi_test::Executor;
    use osmosis_std::types::osmosis::concentratedliquidity::poolmodel::concentrated::v1beta1::MsgCreateConcentratedPool;
    use streamswap_factory::msg::CreatePool;
    use streamswap_stream::msg::ExecuteMsg;

    #[test]
    fn test_pool_creation() {
        let SetupResponse {
            mut app,
            test_accounts,
            stream_swap_code_id,
            stream_swap_factory_code_id,
        } = setup();

        let start_time = app.block_info().time.plus_seconds(1_000_000).into();
        let end_time = app.block_info().time.plus_seconds(5_000_000).into();

        let in_denom = "in_denom";
        let out_supply = 1_000_000_000_000u128;
        let out_denom = "out_denom";
        // %20 of out_supply will go to pool
        let out_clp_amount = 200_000_000_000u128;
        // this is mocked by querier at test_helpers.rs
        let pool_creation_fee = 1000000;
        let pool_creation_denom = "uosmo";
        let stream_creation_denom = "uosmo";
        let stream_creation_fee = 100;
        let subs1_token = Coin::new(1_000_000_000, in_denom);
        let subs2_token = Coin::new(3_000_000_000, in_denom);

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
            None,
            &test_accounts.creator_1.to_string(),
            coin(1_000_000, "out_denom"),
            "in_denom",
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

        // First Subscription
        let subscribe_msg = ExecuteMsg::Subscribe {
            operator_target: None,
            operator: None,
        };
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
        // check outgoing messages
        app.update_block(|b| {
            b.time = end_time.plus_seconds(100_000);
        });
        app.execute_contract(
            test_accounts.creator_1.clone(),
            Addr::unchecked(stream_swap_contract_address.clone()),
            &ExecuteMsg::FinalizeStream { new_treasury: None },
            &[],
        )
        .unwrap();
    }
}
/*
   #[cfg(test)]
   mod pool {
       use super::*;

       use crate::msg::ExecuteMsg;
       use crate::state::CreatePool;
       use crate::test_helpers::{contract_streamswap, MyStargateKeeper};
       use cosmwasm_std::BlockInfo;
       use cw_multi_test::{AppBuilder, Executor};
       use osmosis_std::types::osmosis::concentratedliquidity::poolmodel::concentrated::v1beta1::MsgCreateConcentratedPool;

       #[test]
       fn test_pool_creation() {
           let admin = Addr::unchecked("admin");
           let treasury = Addr::unchecked("treasury");
           let start = Timestamp::from_seconds(1_000_000);
           let end = Timestamp::from_seconds(5_000_000);
           let in_denom = "in_denom";
           let out_supply = 1_000_000_000_000;
           let out_denom = "out_denom";
           // %20 of out_supply will go to pool
           let out_clp_amount = 200_000_000_000;
           // this is mocked by querier at test_helpers.rs
           let pool_creation_fee = 1000000;
           let pool_creation_denom = "uosmo";
           let stream_creation_denom = "uosmo";
           let stream_creation_fee = 100;

           let subs1_addr = Addr::unchecked("subs1");
           let subs1_token = Coin::new(1_000_000_000, in_denom);

           let subs2_addr = Addr::unchecked("subs2");
           let subs2_token = Coin::new(3_000_000_000, in_denom);

           let mut app = AppBuilder::default()
               .with_stargate(MyStargateKeeper {})
               .build(|router, _, storage| {
                   // initialization moved to App construction
                   router
                       .bank
                       .init_balance(
                           storage,
                           &treasury,
                           vec![
                               Coin::new(out_supply + out_clp_amount, out_denom),
                               Coin::new(
                                   pool_creation_fee + stream_creation_fee,
                                   pool_creation_denom,
                               ),
                               // Coin::new(stream_creation_fee, stream_creation_denom),
                           ],
                       )
                       .unwrap();
                   router
                       .bank
                       .init_balance(storage, &subs1_addr, vec![subs1_token.clone()])
                       .unwrap();
                   router
                       .bank
                       .init_balance(storage, &subs2_addr, vec![subs2_token.clone()])
                       .unwrap();
               });

           let code_id = app.store_code(contract_streamswap());
           let msg = crate::msg::InstantiateMsg {
               min_stream_seconds: Uint64::new(1000),
               min_seconds_until_start_time: Uint64::new(1000),
               stream_creation_denom: stream_creation_denom.to_string(),
               stream_creation_fee: stream_creation_fee.into(),
               exit_fee_percent: Decimal::percent(1),
               fee_collector: "collector".to_string(),
               protocol_admin: "protocol_admin".to_string(),
               accepted_in_denom: in_denom.to_string(),
               pool_creation_denom: pool_creation_denom.to_string(),
           };

           // instantiate
           let mut block = BlockInfo {
               height: 100,
               time: Timestamp::from_seconds(100),
               chain_id: "test".to_string(),
           };
           app.set_block(block.clone());
           let contract_addr = app
               .instantiate_contract(
                   code_id,
                   admin.clone(),
                   &msg,
                   &[],
                   "streamswap",
                   Some(admin.to_string()),
               )
               .unwrap();

           // create stream
           block.time = Timestamp::from_seconds(1);
           app.set_block(block);
           let create_stream_msg = ExecuteMsg::CreateStream {
               treasury: treasury.to_string(),
               name: "test".to_string(),
               url: Some("https://sample.url".to_string()),
               in_denom: in_denom.to_string(),
               out_denom: out_denom.to_string(),
               out_supply: out_supply.into(),
               start_time: start,
               end_time: end,
               // %20 will go to pool
               // sender is contract
               threshold: None,
               create_pool: Some(CreatePool {
                   out_amount_clp: out_clp_amount.into(),
                   msg_create_pool: MsgCreateConcentratedPool {
                       sender: treasury.to_string(),
                       denom0: in_denom.to_string(),
                       denom1: out_denom.to_string(),
                       tick_spacing: 100,
                       spread_factor: "10".to_string(),
                   },
               }),
           };
           app.execute_contract(
               treasury.clone(),
               contract_addr.clone(),
               &create_stream_msg,
               &[
                   Coin::new(out_supply + out_clp_amount, out_denom),
                   Coin::new(
                       stream_creation_fee + pool_creation_fee,
                       stream_creation_denom,
                   ),
               ],
           )
           .unwrap();

           // first subscription
           let mut env = mock_env();
           env.block.time = start.plus_seconds(100);
           app.update_block(|b| {
               b.time = start.plus_seconds(100);
           });
           app.execute_contract(
               subs1_addr,
               contract_addr.clone(),
               &ExecuteMsg::Subscribe {
                   stream_id: 1,
                   operator_target: None,
                   operator: None,
               },
               &[subs1_token],
           )
           .unwrap();

           // second subscription
           let mut env = mock_env();
           env.block.time = start.plus_seconds(100_000);
           app.update_block(|b| {
               b.time = start.plus_seconds(100_000);
           });
           app.execute_contract(
               subs2_addr,
               contract_addr.clone(),
               &ExecuteMsg::Subscribe {
                   stream_id: 1,
                   operator_target: None,
                   operator: None,
               },
               &[subs2_token],
           )
           .unwrap();

           // finalize stream
           // check outgoing messages
           app.update_block(|b| {
               b.time = end.plus_seconds(100_000);
           });
           app.execute_contract(
               treasury,
               contract_addr.clone(),
               &ExecuteMsg::FinalizeStream {
                   stream_id: 1,
                   new_treasury: None,
               },
               &[],
           )
           .unwrap();
       }
   }
*/
