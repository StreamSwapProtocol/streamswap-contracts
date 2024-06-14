use cosmwasm_std::{Coin, Decimal, Timestamp, Uint128};
use streamswap_factory::msg::{
    CreatePool, CreateStreamMsg, ExecuteMsg as FactoryExecuteMsg,
    InstantiateMsg as FactoryInstantiateMsg,
};

use super::setup::TestAccounts;

pub fn get_factory_inst_msg(
    stream_swap_code_id: u64,
    test_accounts: &TestAccounts,
) -> FactoryInstantiateMsg {
    FactoryInstantiateMsg {
        stream_swap_code_id,
        protocol_admin: Some(test_accounts.admin.to_string()),
        fee_collector: Some(test_accounts.admin.to_string()),
        stream_creation_fee: Coin {
            denom: "fee_denom".to_string(),
            amount: 100u128.into(),
        },
        exit_fee_percent: Decimal::percent(1),
        accepted_in_denoms: vec!["in_denom".to_string()],
        min_stream_seconds: 100,
        min_seconds_until_start_time: 100,
        pool_creation_denom: "uosmo".to_string(),
    }
}

pub fn get_create_stream_msg(
    name: &str,
    url: Option<String>,
    treasury: &str,
    out_asset: Coin,
    in_denom: &str,
    start_time: Timestamp,
    end_time: Timestamp,
    threshold: Option<Uint128>,
    create_pool: Option<CreatePool>,
) -> FactoryExecuteMsg {
    FactoryExecuteMsg::CreateStream {
        msg: CreateStreamMsg {
            treasury: treasury.to_string(),
            stream_admin: treasury.to_string(),
            name: name.to_string(),
            url,
            out_asset,
            in_denom: in_denom.to_string(),
            start_time,
            end_time,
            threshold,
            create_pool,
        },
    }
}
