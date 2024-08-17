use super::suite::TestAccounts;
use cosmwasm_std::{Binary, Coin, Decimal256, Timestamp, Uint256};
use streamswap_types::controller::{CreateStreamMsg, PoolConfig, VestingConfig};
use streamswap_types::controller::{
    ExecuteMsg as ControllerExecuteMsg, InstantiateMsg as ControllerInstantiateMsg,
};

#[allow(dead_code)]
pub fn get_controller_inst_msg(
    stream_contract_code_id: u64,
    vesting_code_id: u64,
    test_accounts: &TestAccounts,
) -> ControllerInstantiateMsg {
    ControllerInstantiateMsg {
        stream_contract_code_id,
        vesting_code_id,
        protocol_admin: Some(test_accounts.admin.to_string()),
        fee_collector: Some(test_accounts.admin.to_string()),
        stream_creation_fee: Coin {
            denom: "fee_denom".to_string(),
            amount: 100u128.into(),
        },
        exit_fee_percent: Decimal256::percent(1),
        accepted_in_denoms: vec!["in_denom".to_string()],
        min_waiting_duration: 49,
        min_bootstrapping_duration: 49,
        min_stream_duration: 99,
    }
}

#[allow(dead_code)]
pub fn get_create_stream_msg(
    name: &str,
    url: Option<String>,
    treasury: &str,
    out_asset: Coin,
    in_denom: &str,
    bootstrapping_start_time: Timestamp,
    start_time: Timestamp,
    end_time: Timestamp,
    threshold: Option<Uint256>,
    pool_config: Option<PoolConfig>,
    vesting: Option<VestingConfig>,
) -> ControllerExecuteMsg {
    ControllerExecuteMsg::CreateStream {
        msg: Box::new(CreateStreamMsg {
            bootstraping_start_time: bootstrapping_start_time,
            treasury: treasury.to_string(),
            stream_admin: treasury.to_string(),
            name: name.to_string(),
            url,
            out_asset,
            in_denom: in_denom.to_string(),
            start_time,
            end_time,
            threshold,
            pool_config,
            vesting,
            salt: Binary::from_base64("salt").unwrap(),
        }),
    }
}
