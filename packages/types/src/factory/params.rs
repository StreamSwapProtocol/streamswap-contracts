use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal256};

#[cw_serde]
pub struct Params {
    // Protocol admin, Have power to update the params and cancel streams
    pub protocol_admin: Addr,
    // Fee collector, address that will receive the stream creation fee
    pub fee_collector: Addr,
    // Stream creation fee collected from stream creator when a stream is created
    pub stream_creation_fee: Coin,
    // Exit fee percent, fee that will be charged when a user exit a stream
    pub exit_fee_percent: Decimal256,
    pub stream_contract_code_id: u64,
    // Vesting contract code id
    pub vesting_code_id: u64,
    // Accepted in denoms for the stream
    pub accepted_in_denoms: Vec<String>,
    // Minumum time of a stream end_time - start_time
    pub min_stream_duration: u64,
    // Minimum time of bootstrapping status, start_time - bootstrapping_start_time
    pub min_bootstrapping_duration: u64,
    // Minimum time of waiting status, bootstrapping_start_time - creation_time_of_stream
    pub min_waiting_duration: u64,
}
