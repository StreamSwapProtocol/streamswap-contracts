use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Decimal, Timestamp, Uint128};
use cw_vesting::msg::InstantiateMsg as VestingInstantiateMsg;
use osmosis_std::types::osmosis::concentratedliquidity::poolmodel::concentrated::v1beta1::MsgCreateConcentratedPool;

#[cw_serde]
pub struct InstantiateMsg {
    pub stream_swap_code_id: u64,
    pub vesting_code_id: u64,
    pub protocol_admin: Option<String>,
    pub fee_collector: Option<String>,
    pub stream_creation_fee: Coin,
    pub exit_fee_percent: Decimal,
    pub accepted_in_denoms: Vec<String>,
    pub min_stream_seconds: u64,
    pub min_seconds_until_start_time: u64,
    pub min_seconds_until_bootstrapping_start_time: u64,
}

#[cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
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
        msg: Box<CreateStreamMsg>,
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
    pub bootstraping_start_time: Timestamp,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub threshold: Option<Uint128>,
    /// CreatePool Flag
    pub create_pool: Option<CreatePool>,
    /// Vesting configuration
    pub vesting: Option<VestingInstantiateMsg>,
}

#[cw_serde]
pub struct CreatePool {
    // amount of out tokens that will be sent to the pool
    pub out_amount_clp: Uint128,
    // osmosis concentration pool creation message
    pub msg_create_pool: MsgCreateConcentratedPool,
}
#[cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
pub enum QueryMsg {
    #[returns(crate::factory::Params)]
    Params {},
    #[returns(bool)]
    Freezestate {},
    #[returns(u64)]
    LastStreamId {},
}

#[cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::MigrateFns))]
pub enum MigrateMsg {}
