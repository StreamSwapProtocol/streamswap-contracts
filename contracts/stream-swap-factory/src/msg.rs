use cosmwasm_std::{Coin, Decimal, Uint128};

pub struct InstantiateMsg {
    pub stream_swap_code_id: u64,
    pub admin: Option<String>,
    pub stream_creation_fee: Coin,
    pub exit_fee_percent: Decimal,
}

pub enum ExecuteMsg {}
