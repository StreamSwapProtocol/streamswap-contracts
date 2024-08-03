use cosmwasm_schema::cw_serde;
use cw_orch::{interface, prelude::*};
use cw_vesting::contract::{execute, instantiate, query};
use cw_vesting::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

pub const CONTRACT_ID: &str = "cw_vesting";

#[cw_serde]
pub struct MigrateMsg {}

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg, id = CONTRACT_ID)]
pub struct VestingContract;

impl<Chain> Uploadable for VestingContract<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path(CONTRACT_ID)
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper() -> Box<dyn MockContract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(execute, instantiate, query))
    }
}
