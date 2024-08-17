use crate::controller::CreatePool;
use crate::stream::Status;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Coin, Decimal256, Timestamp, Uint128, Uint256};

#[cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg {
    /// SyncStream synchronizes stream and distribution to reflect the current state of the stream.
    SyncStream {},
    Subscribe {},
    /// Withdraw unspent tokens in balance.
    Withdraw {
        cap: Option<Uint256>,
    },
    /// SyncPosition sync the position of the user.
    /// syncs position index to the current state of the stream.
    SyncPosition {},
    /// FinalizeStream clean ups the stream and sends income (earned tokens_in) to the
    /// Stream recipient. Returns error if called before the Stream end. Anyone can
    /// call this method.
    FinalizeStream {
        new_treasury: Option<String>,
        create_pool: Option<CreatePool>,
    },
    /// ExitStream withdraws (by a user who subscribed to the stream) purchased
    /// tokens_out from the pool and remained tokens_in. Must be called after
    /// the stream ends.
    ExitStream {
        /// Salt is required for vested address generation
        salt: Option<Binary>,
    },
    //
    // Killswitch features
    //
    /// ExitCancelled returns the whole balance user put in the stream, both spent and unspent.
    ExitCancelled {},
    CancelStream {},
    CancelStreamWithThreshold {},
    StreamAdminCancel {},
}

#[cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
pub enum QueryMsg {
    /// Returns current configuration.
    #[returns(crate::controller::Params)]
    Params {},
    /// Returns a stream's current state.
    #[returns(StreamResponse)]
    Stream {},
    /// Returns list of streams paginated by `start_after` and `limit`.
    // #[returns(StreamsResponse)]
    // ListStreams {
    //     start_after: Option<u64>,
    //     limit: Option<u32>,
    // },
    /// Returns current state of a position.
    #[returns(PositionResponse)]
    Position { owner: String },
    /// Returns list of positions paginated by `start_after` and `limit`.
    #[returns(PositionsResponse)]
    ListPositions {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Returns average price of a stream sale.
    #[returns(AveragePriceResponse)]
    AveragePrice {},
    /// Returns currently streaming price of a sale.
    #[returns(LatestStreamedPriceResponse)]
    LastStreamedPrice {},
    #[returns(Uint128)]
    Threshold {},
}

#[cw_serde]
pub struct ConfigResponse {
    /// Minimum seconds for a stream to last.
    pub min_stream_seconds: u64,
    /// Minimum seconds until the start time of a stream.
    pub min_seconds_until_start_time: u64,
    /// Denom accepted for subscription.
    pub accepted_in_denom: String,
    /// Denom used as fee for creating a stream.
    pub stream_creation_denom: String,
    /// Creation fee amount.
    pub stream_creation_fee: Uint128,
    /// This percentage represents the fee that will be collected from the investors.
    pub exit_fee_percent: Decimal256,
    /// Address of the fee collector.
    pub fee_collector: String,
    /// Address of the protocol admin.
    pub protocol_admin: String,
}
#[cw_serde]
pub struct StreamResponse {
    /// Address of the treasury where the stream earnings will be sent.
    pub treasury: String,
    /// URL of the stream.
    pub url: Option<String>,
    /// Proportional distribution variable to calculate the distribution of in token_out to buyers.
    pub dist_index: Decimal256,
    /// Last updated time of stream.
    pub last_updated: Timestamp,

    pub out_asset: Coin,
    /// Total number of remaining out tokens at the time of update.
    pub out_remaining: Uint256,
    /// Denom of the `token_in`.
    pub in_denom: String,
    /// Total number of `token_in` on the buy side at latest state.
    pub in_supply: Uint256,
    /// Total number of `token_in` spent at latest state.
    pub spent_in: Uint256,
    /// Total number of shares minted.
    pub shares: Uint256,
    /// start time when the token emission starts. in nanos.
    pub start_time: Timestamp,
    /// end time when the token emission ends.
    pub end_time: Timestamp,
    /// Price at when latest distribution is triggered.
    pub current_streamed_price: Decimal256,
    /// Status of the stream. Can be `Waiting`, `Active`, `Finalzed`, `Paused` or `Canceled` for kill switch.
    pub status: Status,
    /// Address of the stream admin.
    pub stream_admin: String,
}

#[cw_serde]
pub struct StreamsResponse {
    pub streams: Vec<(u32, StreamResponse)>,
}

#[cw_serde]
pub struct PositionResponse {
    /// Creator of the position.
    pub owner: String,
    /// Current amount of tokens in buy pool
    pub in_balance: Uint256,
    pub shares: Uint256,
    // Index is used to calculate the distribution a position has
    pub index: Decimal256,
    // Last_updated_time is the time when the position was last updated
    pub last_updated: Timestamp,
    // Total amount of `token_out` purchased in tokens at latest calculation
    pub purchased: Uint256,
    // Pending purchased accumulates purchases after decimal truncation
    pub pending_purchase: Decimal256,
    // Total amount of `token_in` spent tokens at latest calculation
    pub spent: Uint256,
    // Exit date of the position
    pub exit_date: Timestamp,
}

#[cw_serde]
pub struct PositionsResponse {
    pub positions: Vec<PositionResponse>,
}

#[cw_serde]
pub struct AveragePriceResponse {
    pub average_price: Decimal256,
}

#[cw_serde]
pub struct LatestStreamedPriceResponse {
    pub current_streamed_price: Decimal256,
}

#[cw_serde]
pub struct MigrateMsg {}
