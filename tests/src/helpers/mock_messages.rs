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

pub struct CreateStreamMsgBuilder {
    name: String,
    url: Option<String>,
    treasury: String,
    out_asset: Coin,
    in_denom: String,
    bootstrapping_start_time: Timestamp,
    start_time: Timestamp,
    end_time: Timestamp,
    threshold: Option<Uint256>,
    pool_config: Option<PoolConfig>,
    subscriber_vesting: Option<VestingConfig>,
    salt: Binary,
}

impl CreateStreamMsgBuilder {
    pub fn new(
        name: &str,
        treasury: &str,
        out_asset: Coin,
        in_denom: &str,
        bootstrapping_start_time: Timestamp,
        start_time: Timestamp,
        end_time: Timestamp,
    ) -> Self {
        Self {
            name: name.to_string(),
            url: None,
            treasury: treasury.to_string(),
            out_asset,
            in_denom: in_denom.to_string(),
            bootstrapping_start_time,
            start_time,
            end_time,
            threshold: None,
            pool_config: None,
            subscriber_vesting: None,
            salt: Binary::from_base64("salt").unwrap(),
        }
    }

    pub fn url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    pub fn threshold(mut self, threshold: Uint256) -> Self {
        self.threshold = Some(threshold);
        self
    }

    pub fn pool_config(mut self, pool_config: PoolConfig) -> Self {
        self.pool_config = Some(pool_config);
        self
    }

    pub fn subscriber_vesting(mut self, subscriber_vesting: VestingConfig) -> Self {
        self.subscriber_vesting = Some(subscriber_vesting);
        self
    }

    #[allow(dead_code)]
    pub fn salt(mut self, salt: Binary) -> Self {
        self.salt = salt;
        self
    }

    pub fn build(self) -> ControllerExecuteMsg {
        ControllerExecuteMsg::CreateStream {
            msg: Box::new(CreateStreamMsg {
                bootstraping_start_time: self.bootstrapping_start_time,
                treasury: self.treasury.clone(),
                stream_admin: self.treasury,
                name: self.name,
                url: self.url,
                out_asset: self.out_asset,
                in_denom: self.in_denom,
                start_time: self.start_time,
                end_time: self.end_time,
                threshold: self.threshold,
                pool_config: self.pool_config,
                subscriber_vesting: self.subscriber_vesting,
                salt: self.salt,
            }),
        }
    }
}
