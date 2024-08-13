use cosmwasm_schema::write_api;

use streamswap_types::controller::CreateStreamMsg;
use streamswap_types::stream::{ExecuteMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: CreateStreamMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
