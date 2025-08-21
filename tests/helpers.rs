use cosmwasm_std::testing::{mock_env, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{Addr, Coin, Decimal256, DepsMut, MessageInfo, Timestamp, Uint256, Uint64};

use cw_streamswap::{contract::instantiate, msg::InstantiateMsg};

fn valid_addr() -> String {
    MOCK_CONTRACT_ADDR.to_string()
}

pub const DEFAULT_ACCEPTED_IN_DENOM: &str = "in";
pub const DEFAULT_STREAM_CREATION_DENOM: &str = "fee";

pub fn mock_info(sender: &str, funds: &[Coin]) -> MessageInfo {
    MessageInfo {
        sender: Addr::unchecked(sender),
        funds: funds.to_vec(),
    }
}

pub fn env_now() -> cosmwasm_std::Env {
    mock_env()
}

pub fn env_at(seconds: u64) -> cosmwasm_std::Env {
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(seconds);
    env
}

pub struct InstantiateBuilder {
    pub min_stream_seconds: Uint64,
    pub min_seconds_until_start_time: Uint64,
    pub stream_creation_denom: String,
    pub stream_creation_fee: Uint256,
    pub exit_fee_percent: Decimal256,
    pub fee_collector: String,
    pub protocol_admin: String,
    pub accepted_in_denom: String,
    pub tos_version: String,
}

impl Default for InstantiateBuilder {
    fn default() -> Self {
        Self {
            min_stream_seconds: Uint64::new(1000),
            min_seconds_until_start_time: Uint64::new(1000),
            stream_creation_denom: DEFAULT_STREAM_CREATION_DENOM.to_string(),
            stream_creation_fee: Uint256::from(100u128),
            exit_fee_percent: Decimal256::percent(1),
            fee_collector: valid_addr(),
            protocol_admin: valid_addr(),
            accepted_in_denom: DEFAULT_ACCEPTED_IN_DENOM.to_string(),
            tos_version: "v1".to_string(),
        }
    }
}

impl InstantiateBuilder {
    pub fn exit_fee_percent(mut self, pct: Decimal256) -> Self {
        self.exit_fee_percent = pct;
        self
    }

    pub fn stream_creation_fee(mut self, fee: Uint256) -> Self {
        self.stream_creation_fee = fee;
        self
    }

    pub fn build(self) -> InstantiateMsg {
        InstantiateMsg {
            min_stream_seconds: self.min_stream_seconds,
            min_seconds_until_start_time: self.min_seconds_until_start_time,
            stream_creation_denom: self.stream_creation_denom,
            stream_creation_fee: self.stream_creation_fee,
            exit_fee_percent: self.exit_fee_percent,
            fee_collector: self.fee_collector,
            protocol_admin: self.protocol_admin,
            accepted_in_denom: self.accepted_in_denom,
            tos_version: self.tos_version,
        }
    }
}

pub fn instantiate_defaults(deps: DepsMut) {
    let msg = InstantiateBuilder::default().build();
    instantiate(deps, env_now(), mock_info("creator", &[]), msg).unwrap();
}

pub struct CreateStreamBuilder {
    pub treasury: String,
    pub name: String,
    pub url: Option<String>,
    pub in_denom: String,
    pub out_denom: String,
    pub out_supply: Uint256,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub threshold: Option<Uint256>,
    pub tos_version: String,
}

impl Default for CreateStreamBuilder {
    fn default() -> Self {
        Self {
            treasury: valid_addr(),
            name: "name".to_string(),
            url: Some("https://sample.url".to_string()),
            in_denom: DEFAULT_ACCEPTED_IN_DENOM.to_string(),
            out_denom: "out_denom".to_string(),
            out_supply: Uint256::from(1u128),
            start_time: Timestamp::from_seconds(5_000),
            end_time: Timestamp::from_seconds(10_000),
            threshold: None,
            tos_version: "v1".to_string(),
        }
    }
}

impl CreateStreamBuilder {
    pub fn in_denom(mut self, denom: &str) -> Self {
        self.in_denom = denom.to_string();
        self
    }
    pub fn out_denom(mut self, denom: &str) -> Self {
        self.out_denom = denom.to_string();
        self
    }
    pub fn out_supply(mut self, supply: Uint256) -> Self {
        self.out_supply = supply;
        self
    }
    pub fn start_time(mut self, t: Timestamp) -> Self {
        self.start_time = t;
        self
    }
    pub fn end_time(mut self, t: Timestamp) -> Self {
        self.end_time = t;
        self
    }
    pub fn name(mut self, n: &str) -> Self {
        self.name = n.to_string();
        self
    }
    pub fn url(mut self, u: Option<&str>) -> Self {
        self.url = u.map(|s| s.to_string());
        self
    }
    pub fn tos_version(mut self, v: &str) -> Self {
        self.tos_version = v.to_string();
        self
    }
}

#[allow(dead_code)]
pub fn valid_funds_for(builder: &CreateStreamBuilder) -> Vec<Coin> {
    if builder.out_denom == DEFAULT_STREAM_CREATION_DENOM {
        // require out_denom funds equal to out_supply + creation_fee
        let total = builder.out_supply + Uint256::from(100u128);
        vec![Coin {
            denom: builder.out_denom.clone(),
            amount: total,
        }]
    } else {
        vec![
            Coin {
                denom: builder.out_denom.clone(),
                amount: builder.out_supply,
            },
            Coin {
                denom: DEFAULT_STREAM_CREATION_DENOM.to_string(),
                amount: Uint256::from(100u128),
            },
        ]
    }
}
