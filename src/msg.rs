use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal, Decimal256, Timestamp, Uint128, Uint64};

#[cw_serde]
pub struct InstantiateMsg {
    pub min_stream_seconds: Uint64,
    pub min_seconds_until_start_time: Uint64,
    pub stream_creation_denom: String,
    pub stream_creation_fee: Uint128,
    pub fee_collector: String,
    pub protocol_admin: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    // CreateStream creates new token stream. Anyone can create a new stream.
    // Creation Fee send along msg prevents spams.
    CreateStream {
        // Address where the stream earnings will go
        treasury: String,
        // Name of the stream
        name: String,
        // An external resource describing a stream. Can be IPFS link or a
        url: String,
        // Payment denom - used to buy `token_out`.
        // Also known as quote currency.
        in_denom: String,
        // Denom to stream (distributed to the investors).
        // Also known as a base currency.
        out_denom: String,
        out_supply: Uint128,
        // Unix timestamp when the stream starts. Calculations in nano sec precision
        start_time: Timestamp,
        // Unix timestamp when the stream ends. Calculations in nano sec precision
        end_time: Timestamp,
    },
    // Update stream and calculates distribution state
    UpdateStream {
        stream_id: u64,
    },
    UpdateOperator {
        stream_id: u64,
        operator: Option<String>,
    },
    // Subscribe to a token stream. Any use at any time before the stream end can join
    // the stream by sending `token_in` to the Stream through the Subscribe msg.
    // During the stream, user `token_in` will be automatically charged every
    // epoch to purchase `token_out`.
    Subscribe {
        stream_id: u64,
        // operator can load
        position_owner: Option<String>,
        // operator can subscribe/withdraw/update position
        operator: Option<String>,
    },
    // Withdraws unspent in balance
    Withdraw {
        stream_id: u64,
        cap: Option<Uint128>,
        position_owner: Option<String>,
    },
    UpdatePosition {
        stream_id: u64,
        position_owner: Option<String>,
    },
    // FinalizeStream clean ups the stream and sends income (earned tokens_in) to the
    // Stream recipient. Returns error if called before the Stream end. Anyone can
    // call this method.
    FinalizeStream {
        stream_id: u64,
        new_treasury: Option<String>,
    },
    // ExitStream withdraws (by a user who subscribed to the stream) purchased
    // tokens_out from the pool and remained tokens_in. Must be called before
    // the stream end.
    ExitStream {
        stream_id: u64,
        position_owner: Option<String>,
    },
    //
    // Collector
    //
    // CollectFees collects fees from the contract
    CollectFees {},

    //
    // Killswitch features
    //
    // PauseStream pauses the stream. Only protocol admin and governance can pause the stream.
    PauseStream {
        stream_id: u64,
    },
    // WithdrawPaused is used to withdraw unspent position funds during pause.
    WithdrawPaused {
        stream_id: u64,
        cap: Option<Uint128>,
        position_owner: Option<String>,
    },
    // WithdrawCancelled returns the whole balance user put in the stream, both spent and unspent.
    WithdrawCancelled {
        stream_id: u64,
        position_owner: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(StreamResponse)]
    Stream { stream_id: u64 },
    #[returns(StreamsResponse)]
    ListStreams {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(PositionResponse)]
    Position { stream_id: u64, owner: String },
    #[returns(PositionsResponse)]
    ListPositions {
        stream_id: u64,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(AveragePriceResponse)]
    AveragePrice { stream_id: u64 },
    #[returns(LatestStreamedPriceResponse)]
    LastStreamedPrice { stream_id: u64 },
}

#[cw_serde]
pub struct StreamResponse {
    pub id: u64,
    pub treasury: String,
    pub dist_index: Decimal256,
    pub shares: Uint128,
    pub last_updated: Timestamp,
    pub out_denom: String,
    pub out_supply: Uint128,
    pub out_remaining: Uint128,
    pub in_denom: String,
    pub in_supply: Uint128,
    pub in_spent: Uint128,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
}

#[cw_serde]
pub struct StreamsResponse {
    pub streams: Vec<StreamResponse>,
}

#[cw_serde]
pub struct PositionResponse {
    pub stream_id: u64,
    pub owner: String,
    pub in_balance: Uint128,
    pub shares: Uint128,
    pub index: Decimal256,
    pub last_updated: Timestamp,
    pub purchased: Uint128,
    pub pending_purchase: Decimal256,
    pub spent: Uint128,
    pub operator: Option<Addr>,
}

#[cw_serde]
pub struct PositionsResponse {
    pub positions: Vec<PositionResponse>,
}

#[cw_serde]
pub struct AveragePriceResponse {
    pub average_price: Decimal,
}

#[cw_serde]
pub struct LatestStreamedPriceResponse {
    pub current_streamed_price: Decimal,
}

#[cw_serde]
pub enum SudoMsg {
    UpdateConfig {
        min_stream_duration: Option<Uint64>,
        min_duration_until_start_time: Option<Uint64>,
        stream_creation_denom: Option<String>,
        stream_creation_fee: Option<Uint128>,
        fee_collector: Option<String>,
    },
    PauseStream {
        stream_id: u64,
    },
    CancelStream {
        stream_id: u64,
    },
}

#[cw_serde]
pub struct MigrateMsg {}
