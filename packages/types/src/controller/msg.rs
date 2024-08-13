use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Coin, Decimal256, Timestamp, Uint256};
use cw_vesting::msg::InstantiateMsg as VestingInstantiateMsg;
use osmosis_std::types::osmosis::concentratedliquidity::poolmodel::concentrated::v1beta1::MsgCreateConcentratedPool;

#[cw_serde]
/// Message used to instantiate the controller contract.
pub struct InstantiateMsg {
    /// The code ID for the stream contract.
    pub stream_contract_code_id: u64,
    /// The code ID for the vesting contract.
    pub vesting_code_id: u64,
    /// The optional address of the protocol admin. Defaults to the sender.
    pub protocol_admin: Option<String>,
    /// The optional address of the fee collector. Defaults to the protocol admin.
    pub fee_collector: Option<String>,
    /// The fee required to create a stream. Collected from the stream creator upon stream creation.
    pub stream_creation_fee: Coin,
    /// The percentage fee charged when a user exits a stream.
    pub exit_fee_percent: Decimal256,
    /// The list of accepted denominations for the stream.
    pub accepted_in_denoms: Vec<String>,
    // Minumum time of a stream end_time - start_time
    pub min_stream_duration: u64,
    // Minimum time of bootstrapping status, start_time - bootstrapping_start_time
    pub min_bootstrapping_duration: u64,
    // Minimum time of waiting status, bootstrapping_start_time - creation_time_of_stream
    pub min_waiting_duration: u64,
}

#[cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg {
    UpdateParams {
        min_stream_duration: Option<u64>,
        min_bootstrapping_duration: Option<u64>,
        min_waiting_duration: Option<u64>,
        stream_creation_fee: Option<Coin>,
        fee_collector: Option<String>,
        accepted_in_denoms: Option<Vec<String>>,
        exit_fee_percent: Option<Decimal256>,
    },
    CreateStream {
        msg: Box<CreateStreamMsg>,
    },
    Freeze {},
    Unfreeze {},
}

#[cw_serde]
pub struct CreateStreamMsg {
    /// Treasury address, where the stream creator can withdraw the in assets at the end of the stream
    pub treasury: String,
    /// Stream admin address, where the stream creator can manage the stream, like canceling it in waiting status
    /// or finalizing it in ended status
    pub stream_admin: String,
    /// Name of the stream
    pub name: String,
    /// URL of the stream
    pub url: Option<String>,
    /// Out asset of the stream
    pub out_asset: Coin,
    /// In denom of the stream
    pub in_denom: String,
    /// Bootstrapping start time
    pub bootstraping_start_time: Timestamp,
    /// Stream start time
    pub start_time: Timestamp,
    /// Stream end time
    pub end_time: Timestamp,
    /// Optional threshold for the stream, if set, the stream will be cancelled if the threshold is not reached
    pub threshold: Option<Uint256>,
    /// CreatePool Flag
    pub create_pool: Option<CreatePool>,
    /// Vesting configuration
    pub vesting: Option<VestingInstantiateMsg>,
    // Salt is used to instantiate stream contracts deterministically.
    // Pass randomly generated value here. bech32 hashed would be ideal.
    pub salt: Binary,
}

#[cw_serde]
pub struct CreatePool {
    // amount of out tokens that will be sent to the pool
    pub out_amount_clp: Uint256,
    // osmosis concentration pool creation message
    pub msg_create_pool: MsgCreateConcentratedPool,
}
#[cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
pub enum QueryMsg {
    #[returns(crate::controller::Params)]
    Params {},
    #[returns(bool)]
    Freezestate {},
    #[returns(u64)]
    LastStreamId {},
    /// Returns list of streams paginated by `start_after` and `limit`.
    #[returns(StreamsResponse)]
    ListStreams {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct StreamsResponse {
    pub streams: Vec<StreamResponse>,
}

#[cw_serde]
pub struct StreamResponse {
    pub id: u64,
    pub address: String,
}

#[cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::MigrateFns))]
pub enum MigrateMsg {}
