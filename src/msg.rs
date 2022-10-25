use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal, Timestamp, Uint128, Uint64};

#[cw_serde]
pub struct InstantiateMsg {
    pub min_sale_duration: Uint64,
    pub min_duration_until_start_time: Uint64,
    pub sale_creation_denom: String,
    pub sale_creation_fee: Uint128,
    pub fee_collector: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Update the distribution index
    UpdateDistributionIndex {
        sale_id: u64,
    },

    // CreateSale creates new token sale. Anyone can create a new sale.
    // params.SaleBond OSMO will be charged as a bond (returned in FinalizeSale)
    // to avoid spams.
    CreateSale {
        // Address where the sale earnings will go
        treasury: String,
        // Name of the sale
        name: String,
        // An external resource describing a sale. Can be IPFS link or a
        url: String,
        // Payment denom - used to buy `token_out`.
        // Also known as quote currency.
        token_in_denom: String,
        // Denom to sale (distributed to the investors).
        // Also known as a base currency.
        token_out_denom: String,
        token_out_supply: Uint128,
        // Unix timestamp when the sale starts.
        start_time: Timestamp,
        // Unix timestamp when the sale ends.
        end_time: Timestamp,
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

    // FinalizeSale clean ups the sale and sends income (earned tokens_in) to the
    // Sale recipient. Returns error if called before the Sale end. Anyone can
    // call this method.
    FinalizeSale {
        sale_id: u64,
        new_treasury: Option<String>,
    },

    // ExitSale withdraws (by a user who subscribed to the sale) purchased
    // tokens_out from the pool and remained tokens_in. Must be called before
    // the sale end.
    ExitSale {
        sale_id: u64,
        recipient: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(SaleResponse)]
    Sale { sale_id: u64 },
    #[returns(SalesResponse)]
    ListSales {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(PositionResponse)]
    Position { sale_id: u64, owner: String },
    #[returns(PositionsResponse)]
    ListPositions {
        sale_id: u64,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(AveragePriceResponse)]
    AveragePrice { sale_id: u64 },
    #[returns(LatestStreamedPriceResponse)]
    LastStreamedPrice { sale_id: u64 },
}

#[cw_serde]
pub struct SaleResponse {
    pub id: u64,
    pub treasury: String,
    pub dist_index: Decimal,
    pub latest_stage: Decimal,
    pub token_out_denom: String,
    pub token_out_supply: Uint128,
    pub total_out_sold: Uint128,
    pub token_in_denom: String,
    pub total_in_supply: Uint128,
    pub total_in_spent: Uint128,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
}

#[cw_serde]
pub struct SalesResponse {
    pub sales: Vec<SaleResponse>,
}

#[cw_serde]
pub struct PositionResponse {
    pub sale_id: u64,
    pub owner: String,
    pub in_balance: Uint128,
    pub index: Decimal,
    pub current_stage: Decimal,
    pub purchased: Uint128,
    pub spent: Uint128,
    pub exited: bool,
}

#[cw_serde]
pub struct PositionsResponse {
    pub positions: Vec<PositionResponse>,
}

#[cw_serde]
pub struct AveragePriceResponse {
    pub average_price: Uint128,
}

#[cw_serde]
pub struct LatestStreamedPriceResponse {
    pub current_streamed_price: Uint128,
}

#[cw_serde]
pub enum SudoMsg {
    UpdateConfig {
        min_sale_duration: Option<Uint64>,
        min_duration_until_start_time: Option<Uint64>,
        sale_creation_denom: Option<String>,
        sale_creation_fee: Option<Uint128>,
        fee_collector: Option<String>,
    },
}

#[cw_serde]
pub struct MigrateMsg {}
