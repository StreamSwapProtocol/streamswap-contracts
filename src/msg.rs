use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Decimal, Timestamp, Uint128, Uint64};
use cw_utils::Scheduled;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    // Address where the sale earnings will go
    pub treasury: String,
    // Name of the sale
    pub name: String,
    // An external resource describing a sale. Can be IPFS link or a
    pub url: String,
    // Payment denom - used to buy `token_out`.
    // Also known as quote currency.
    pub token_in_denom: String,
    // Denom to sale (distributed to the investors).
    // Also known as a base currency.
    pub token_out_denom: String,
    pub token_out_supply: Uint128,
    // Unix timestamp when the sale starts.
    pub start_time: Timestamp,
    // Unix timestamp when the sale ends.
    pub end_time: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // Update the distribution index
    UpdateDistributionIndex {},

    // Subscribe to a token sale. Any use at any time before the sale end can join
    // the sale by sending `token_in` to the Sale through the Subscribe msg.
    // During the sale, user `token_in` will be automatically charged every
    // epoch to purchase `token_out`.
    Subscribe {},
    // Withdraws released stake
    Withdraw { cap: Option<Uint128> },

    ////////////////////
    /// User's operations
    ///////////////////
    TriggerPositionPurchase {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    State {},
    AccruedDistribution {
        address: String,
    },
    Holder {
        address: String,
    },
    Holders {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    Claims {
        address: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub cw20_token_addr: String,
    pub unbonding_period: u64,
    pub global_index: Decimal,
    pub total_balance: Uint128,
    pub prev_reward_balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AccruedRewardsResponse {
    pub rewards: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PositionResponse {
    pub address: String,
    pub balance: Uint128,
    pub index: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HoldersResponse {
    pub holders: Vec<PositionResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
