use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Attribute, Coin, Decimal256};

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

impl Params {
    // Converts Params to attributes
    pub fn to_attributes(&self) -> Vec<Attribute> {
        vec![
            Attribute::new("protocol_admin", self.protocol_admin.to_string()),
            Attribute::new("fee_collector", self.fee_collector.to_string()),
            Attribute::new("stream_creation_fee", self.stream_creation_fee.to_string()),
            Attribute::new("exit_fee_percent", self.exit_fee_percent.to_string()),
            Attribute::new(
                "stream_contract_code_id",
                self.stream_contract_code_id.to_string(),
            ),
            Attribute::new("vesting_code_id", self.vesting_code_id.to_string()),
            Attribute::new("accepted_in_denoms", self.accepted_in_denoms.join(",")),
            Attribute::new("min_stream_duration", self.min_stream_duration.to_string()),
            Attribute::new(
                "min_bootstrapping_duration",
                self.min_bootstrapping_duration.to_string(),
            ),
            Attribute::new(
                "min_waiting_duration",
                self.min_waiting_duration.to_string(),
            ),
        ]
    }
}
