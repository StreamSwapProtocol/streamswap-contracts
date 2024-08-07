use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal};

#[cw_serde]
pub struct Params {
    // Protocol admin, Have power to update the params and cancel streams
    pub protocol_admin: Addr,
    // Fee collector, address that will receive the stream creation fee
    pub fee_collector: Addr,
    // Stream creation fee collected from stream creator when a stream is created
    pub stream_creation_fee: Coin,
    // Exit fee percent, fee that will be charged when a user exit a stream
    pub exit_fee_percent: Decimal,
    pub stream_contract_code_id: u64,
    // Vesting contract code id
    pub vesting_code_id: u64,
    // Accepted in denoms for the stream
    pub accepted_in_denoms: Vec<String>,
    // Minumum time of a stream end_time - start_time
    pub min_stream_seconds: u64,
    // Stream starts at waiting status, then bootstrapping, this parameter is the minimum time of waiting+bootstrapping
    pub min_seconds_until_start_time: u64,
    // Stream starts at bootstrapping status, this parameter is the minimum time of waiting
    pub min_seconds_until_bootstrapping_start_time: u64,
}
