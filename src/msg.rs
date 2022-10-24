use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Decimal, Timestamp, Uint128};

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
    UpdateDistributionIndex {
        sale_id: u64,
    },

    // Subscribe to a token sale. Any use at any time before the sale end can join
    // the sale by sending `token_in` to the Sale through the Subscribe msg.
    // During the sale, user `token_in` will be automatically charged every
    // epoch to purchase `token_out`.
    Subscribe {
        sale_id: u64,
    },
    // Withdraws released stake
    Withdraw {
        sale_id: u64,
        cap: Option<Uint128>,
        recipient: Option<String>,
    },

    TriggerPositionPurchase {
        sale_id: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Returns the current state of the sale
    Sale { sale_id: u64 },
    // Returns the current state of the position
    Position { sale_id: u64, owner: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
