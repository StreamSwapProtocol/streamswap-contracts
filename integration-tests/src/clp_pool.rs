#[cfg(test)]
mod clp_pool {
    use cosmwasm_std::{coin, Coin, Decimal, Uint128};
    use cw_orch::prelude::*;
    use cw_orch_osmosis_test_tube::OsmosisTestTube;
    use osmosis_std::types::osmosis::concentratedliquidity::poolmodel::concentrated::v1beta1::MsgCreateConcentratedPool;
    use cw_orch_osmosis_test_tube::osmosis_test_tube::{Account, Module, OsmosisTestApp, Wasm};
    use streamswap_interface::factory::StreamSwapFactoryContract;
    use streamswap_interface::stream::StreamSwapStreamContract;
    use streamswap_interface::vesting::VestingContract;
    use streamswap_test_helpers::utils::get_wasm_attribute_with_key;
    use streamswap_types::factory::{CreatePool, InstantiateMsg as FactoryInstantiateMsg};
    use streamswap_test_helpers::mock_messages::get_create_stream_msg;

    #[test]
    fn test_pool_creation() {
        let coins = vec![
            coin(1_000_000_000_000, "uosmo"),
            coin(1_000_000_000_000, "fee_denom"),
            coin(1_000_000_000_000, "in_denom"),
            coin(1_000_000_000_000, "out_denom")
        ];
        let mut chain = OsmosisTestTube::new(coins);
        let signer = chain.init_account(
            vec![
            coin(1_000_000_000_000, "uosmo"),
            coin(1_000_000_000_000, "fee_denom"),
            coin(1_000_000_000_000, "in_denom"),
            coin(1_000_000_000_000, "out_denom")
        ]).unwrap();

        let out_denom = "out_denom".to_string();
        let in_denom= "in_denom".to_string();

        let factory = StreamSwapFactoryContract::new(chain.clone());
        factory.upload().unwrap();

        let stream = StreamSwapStreamContract::new(chain.clone());
        stream.upload().unwrap();
        let stream_swap_code_id = stream.code_id().unwrap();

        let vesting = VestingContract::new(chain.clone());
        vesting.upload().unwrap();
        let vesting_code_id = vesting.code_id().unwrap();

        let admin = chain
            .init_account(vec![
                coin(1_000_000_000_000, out_denom.clone()),
                coin(100, "fee_denom")
            ])
            .unwrap();

        let treasury = chain
            .init_account(vec![coin(1_000_000_000_000, out_denom.clone())])
            .unwrap();

        let creator1 = chain
            .init_account(vec![
                coin(1_000_000_000_000, out_denom.clone()),
                coin(100, "fee_denom"),
                coin(1_000_000_000_000, "uosmo".to_string()),
            ])
            .unwrap();

            // add uosmo coin to subscriber

        let subscriber = chain
            .init_account(vec![
                coin(1_000_000_000_000, in_denom.clone()),
                coin(1_000_000_000_000, "uosmo".to_string()),
                ])
            .unwrap();

        // create pool
        let pool_id = chain.create_pool(vec![
            coin(1_000, out_denom.clone()),
            coin(1_000, in_denom.clone()),
        ]).unwrap();

        println!("Pool ID: {:?}", pool_id);
        // instantiate factory
        let msg = FactoryInstantiateMsg {
            stream_swap_code_id,
            vesting_code_id,
            protocol_admin: Some(admin.address()),
            fee_collector: Some(admin.address()),
            stream_creation_fee: Coin {
                denom: "fee_denom".to_string(),
                amount: 100u128.into(),
            },
            exit_fee_percent: Decimal::percent(1),
            accepted_in_denoms: vec!["in_denom".to_string()],
            min_stream_seconds: 100,
            min_seconds_until_start_time: 100,
        };

        factory.instantiate(&msg, None, None).unwrap();

        let start_time = chain.block_info().unwrap().time.plus_seconds(1000);
        let end_time = chain.block_info().unwrap().time.plus_seconds(3000);
        let create_stream_msg = get_create_stream_msg(
            &"Stream Swap tests".to_string(),
            None,
            creator1.address().as_str(),
            coin(1_000_000, "out_denom"),
            "in_denom",
            start_time,
            end_time,
            None,
            Some(CreatePool {
                out_amount_clp: Uint128::new(1_000_000),
                msg_create_pool: MsgCreateConcentratedPool {
                    sender: treasury.address(),
                    denom0: "out_denom".to_string(),
                    denom1: "in_denom".to_string(),
                    tick_spacing: 1,
                    spread_factor: "".to_string(),
                }
            }),
            None,
        );
        
        let res = factory.execute(&create_stream_msg, Some(&[coin(100, "fee_denom"), coin(1_000_000, "out_denom")])).unwrap();  
        // print events
        let addr = get_wasm_attribute_with_key(res.events, "stream_contract_address".to_string());
        let stream_addr = "osmo1nc5tatafv6eyq7llkr2gv50ff9e22mnf70qgjlv737ktmt4eswrqvlx82r".to_string();
        stream.set_address(&Addr::unchecked(stream_addr.clone()));
        // TODO fix later address issue
        // stream 1 addr osmo1nc5tatafv6eyq7llkr2gv50ff9e22mnf70qgjlv737ktmt4eswrqvlx82r

        // proceed with stream subscription
        let subsribe_msg = streamswap_types::stream::ExecuteMsg::Subscribe {
            operator_target: None,
            operator: None,
        };
        stream.call_as(&subscriber).execute(&subsribe_msg, Some(&[coin(200, in_denom)])).unwrap();

        // increase time
        chain.app.borrow().increase_time(end_time.seconds());
        // finalize stream
        let finalized_msg = streamswap_types::stream::ExecuteMsg::FinalizeStream { new_treasury: None };
        let finalize_res = stream.call_as(&creator1).execute(&finalized_msg, None).unwrap();

        // print events
        println!("{:?}", finalize_res.events);

        //let concentrated_liquidity = ConcentratedLiquidity::new(&*chain.app.borrow()).query_pools(&PoolsRequest { pagination: None }).unwrap();
        //println!("{:?}", concentrated_liquidity);

    }

    /*
        let mut chain = OsmosisTestApp::new();
        let signer = chain.init_account(
            &[
            coin(1_000_000_000_000, "uosmo"),
            coin(1_000_000_000_000, "fee_denom"),
            coin(1_000_000_000_000, "in_denom"),
            coin(1_000_000_000_000, "out_denom")
        ]).unwrap();
     */

    #[test]
    fn test_pool_creation_test_tube() {
        let out_denom = "out_denom".to_string();
        let in_denom= "in_denom".to_string();

        let mut app= OsmosisTestApp::new();
        let accs= app.init_accounts(
            &[
                coin(1_000_000_000_000, "uosmo"),
                coin(1_000_000_000_000, "fee_denom"),
                coin(1_000_000_000_000, "in_denom"),
                coin(1_000_000_000_000, "out_denom")
            ],2).unwrap();
        let admin = &accs[0];
        let creator= &accs[1];

        let wasm = Wasm::new(&app);

        // Load compiled wasm bytecode
        let wasm_byte_code = std::fs::read("../artifacts/streamswap_factory.wasm").unwrap();
        let code_id = wasm
            .store_code(&wasm_byte_code, None, &admin)
            .unwrap()
            .data
            .code_id;
    }
}

