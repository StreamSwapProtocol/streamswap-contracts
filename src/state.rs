use crate::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Decimal256, Storage, Timestamp, Uint128, Uint64};
use cw_storage_plus::{Item, Map};
use std::ops::Mul;

#[cw_serde]
pub struct Config {
    // minimum sale duration in unix seconds
    pub min_stream_seconds: Uint64,
    // min duration between start time and current time in unix seconds
    pub min_seconds_until_start_time: Uint64,
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
    pub dist_index: Decimal256,
    // last updated time of stream
    pub last_updated: Timestamp,
    // denom of the `token_out`
    pub out_denom: String,
    // total number of `token_out` to be sold during the continuous stream.
    pub out_supply: Uint128,
    // total number of remaining out tokens at the time of update
    pub out_remaining: Uint128,
    // denom of the `token_in`
    pub in_denom: String,
    // total number of `token_in` on the buy side at latest state
    pub in_supply: Uint128,
    // total number of `token_in` spent at latest state
    pub spent_in: Uint128,
    pub shares: Uint128,
    // start time when the token emission starts. in nanos
    pub start_time: Timestamp,
    // end time when the token emission ends. Can't be bigger than start +
    // 139years (to avoid round overflow)
    pub end_time: Timestamp,
    // price at when latest distribution is triggered
    pub current_streamed_price: Decimal,
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
        last_updated: Timestamp,
    ) -> Self {
        Stream {
            name,
            treasury,
            url,
            dist_index: Decimal256::zero(),
            last_updated,
            out_denom,
            out_supply,
            out_remaining: out_supply,
            in_denom,
            in_supply: Uint128::zero(),
            spent_in: Uint128::zero(),
            shares: Uint128::zero(),
            start_time,
            end_time,
            current_streamed_price: Decimal::zero(),
        }
    }

    // compute amount of shares that should be minted for a new subscription amount
    pub fn compute_shares_amount(&self, amount_in: Uint128, round_up: bool) -> Uint128 {
        if self.shares.is_zero() || amount_in.is_zero() {
            return amount_in;
        }
        let mut shares = self.shares.mul(amount_in);
        if round_up {
            shares = (shares + self.in_supply - Uint128::one()) / self.in_supply;
        } else {
            shares = shares / self.in_supply;
        }
        shares
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
    pub shares: Uint128,
    // index is used to calculate the distribution a position has
    pub index: Decimal256,
    pub last_updated: Timestamp,
    // total amount of `token_out` purchased in tokens at latest calculation
    pub purchased: Uint128,
    // pending purchased accumulates purchases after decimal truncation
    pub pending_purchase: Decimal256,
    // total amount of `token_in` spent tokens at latest calculation
    pub spent: Uint128,
    // operator can update position
    pub operator: Option<Addr>,
}

impl Position {
    pub fn new(
        owner: Addr,
        in_balance: Uint128,
        shares: Uint128,
        index: Option<Decimal256>,
        last_updated: Timestamp,
        operator: Option<Addr>,
    ) -> Self {
        Position {
            owner,
            in_balance,
            shares,
            index: index.unwrap_or_default(),
            last_updated,
            purchased: Uint128::zero(),
            pending_purchase: Decimal256::zero(),
            spent: Uint128::zero(),
            operator,
        }
    }
}

// Position (stream_id, owner_addr) -> Position
pub const POSITIONS: Map<(StreamId, &Addr), Position> = Map::new("positions");
