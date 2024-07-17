#[cfg(test)]
mod clp_pool {
    use cosmwasm_std::{coin, Coin, Decimal};
    use cw_orch::prelude::*;
    use cw_orch_osmosis_test_tube::OsmosisTestTube;
    use osmosis_test_tube::Account;
    use streamswap_interface::factory::StreamSwapFactoryContract;
    use streamswap_interface::stream::StreamSwapStreamContract;
    use streamswap_interface::vesting::VestingContract;
    use streamswap_types::factory::InstantiateMsg;

    #[test]
    fn pool() {
        let mut chain = OsmosisTestTube::new(vec![coin(1_000_000_000_000, "uosmo")]);

        let factory = StreamSwapFactoryContract::new(chain.clone());
        factory.upload().unwrap();

        let stream = StreamSwapStreamContract::new(chain.clone());
        stream.upload().unwrap();
        let stream_swap_code_id = stream.code_id().unwrap();

        let vesting = VestingContract::new(chain.clone());
        vesting.upload().unwrap();
        let vesting_code_id = vesting.code_id().unwrap();

        let admin = chain
            .init_account(vec![coin(1_000_000_000_000, "sell")])
            .unwrap();
        // instantiate factory
        let msg = InstantiateMsg {
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

        let factory_addr = factory.instantiate(&msg, None, None).unwrap();
    }
}
