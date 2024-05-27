use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Decimal, Timestamp, Uint128, Uint64};
#[cw_serde]
pub struct InstantiateMsg {
    pub stream_swap_code_id: u64,
    pub protocol_admin: Option<String>,
    pub fee_collector: Option<String>,
    pub stream_creation_fee: Coin,
    pub exit_fee_percent: Decimal,
    pub accepted_in_denoms: Vec<String>,
    pub min_stream_seconds: u64,
    pub min_seconds_until_start_time: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateParams {
        min_stream_seconds: Option<u64>,
        min_seconds_until_start_time: Option<u64>,
        stream_creation_fee: Option<Coin>,
        fee_collector: Option<String>,
        accepted_in_denoms: Option<Vec<String>>,
        exit_fee_percent: Option<Decimal>,
    },
    CreateStream {
        msg: CreateStreamMsg,
    },
    Freeze {},
}

#[cw_serde]
pub struct CreateStreamMsg {
    pub treasury: String,
    pub stream_admin: String,
    pub name: String,
    pub url: Option<String>,
    pub out_asset: Coin,
    pub in_denom: String,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub threshold: Option<Uint128>,
}

#[cw_serde]
pub enum QueryMsg {
    Params {},
    Freezestate {},
    LastStreamId {},
}
