use crate::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Storage, Timestamp, Uint128, Uint64};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    // minimum sale duration in unix seconds
    pub min_stream_duration: Uint64,
    // min duration between start time and current time in unix seconds
    pub min_duration_until_start_time: Uint64,
    pub stream_creation_denom: String,
    pub stream_creation_fee: Uint128,
    pub fee_collector: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct Stream {
    // Name of the stream
    pub name: String,
    // Destination for the earned token_in
    pub treasury: Addr,
    // URL for more information about the stream
    pub url: String,
    // Proportional distribution variable to calculate the distribution of in token_out to buyers.
    pub dist_index: Decimal,
    // last calculated stage of stream, %0 -> %100
    pub current_stage: Decimal,
    // denom of the `token_out`
    pub out_denom: String,
    // total number of `token_out` to be sold during the continuous stream.
    pub out_supply: Uint128,
    // total number of `token_out` sold at latest state
    pub current_out: Uint128,
    // denom of the `token_in`
    pub in_denom: String,
    // total number of `token_in` on the buy side at latest state
    pub in_supply: Uint128,
    // total number of `token_in` spent at latest state
    pub spent_in: Uint128,
    // TODO: convert to Timestamp
    // start time when the token emission starts. in nanos
    pub start_time: Timestamp,
    // end time when the token emission ends. Can't be bigger than start +
    // 139years (to avoid round overflow)
    pub end_time: Timestamp,
    // price at when latest distribution is triggered
    pub current_streamed_price: Uint128,
}

impl Stream {
    pub fn new(
        name: String,
        treasury: Addr,
        url: String,
        out_denom: String,
        out_supply: Uint128,
        in_denom: String,
        start_time: Timestamp,
        end_time: Timestamp,
    ) -> Self {
        Stream {
            name,
            treasury,
            url,
            dist_index: Decimal::zero(),
            current_stage: Decimal::zero(),
            out_denom,
            out_supply,
            current_out: Uint128::zero(),
            in_denom,
            in_supply: Uint128::zero(),
            spent_in: Uint128::zero(),
            start_time,
            end_time,
            current_streamed_price: Uint128::zero(),
        }
    }
}
type StreamId = u64;
pub const STREAMS: Map<StreamId, Stream> = Map::new("stream");
const STREAM_ID_COUNTER: Item<StreamId> = Item::new("stream_id_counter");
pub fn next_stream_id(store: &mut dyn Storage) -> Result<u64, ContractError> {
    let id: u64 = STREAM_ID_COUNTER.may_load(store)?.unwrap_or_default() + 1;
    STREAM_ID_COUNTER.save(store, &id)?;
    Ok(id)
}

#[cw_serde]
pub struct Position {
    // creator of the position
    pub owner: Addr,
    // current amount of tokens in buy pool
    pub in_balance: Uint128,
    // index is used to calculate the distribution a position has
    pub index: Decimal,
    pub current_stage: Decimal,
    // total amount of `token_out` purchased in tokens at latest calculation
    pub purchased: Uint128,
    // total amount of `token_in` spent tokens at latest calculation
    pub spent: Uint128,
    // finalized becomes true when position is finalized and tokens are sent to the recipient
    pub exited: bool,
}

impl Position {
    pub fn new(owner: Addr, in_balance: Uint128, index: Option<Decimal>) -> Self {
        Position {
            owner,
            in_balance,
            index: index.unwrap_or_default(),
            current_stage: Decimal::zero(),
            purchased: Uint128::zero(),
            spent: Uint128::zero(),
            exited: false,
        }
    }
}

// Position (stream_id, owner_addr) -> Position
pub const POSITIONS: Map<(StreamId, &Addr), Position> = Map::new("positions");
