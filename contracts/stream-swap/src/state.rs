use crate::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal, Decimal256, Storage, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};
use std::ops::Mul;
#[cw_serde]
pub struct Stream {
    /// Name of the stream.
    pub name: String,
    /// Destination for the earned token_in.
    pub treasury: Addr,

    pub stream_admin: Addr,
    /// URL for more information about the stream.
    pub url: Option<String>,
    /// Proportional distribution variable to calculate the distribution of in token_out to buyers.
    pub dist_index: Decimal256,
    /// Last updated block of stream.
    pub last_updated_time: Timestamp,
    /// Denom of the `token_out`.
    pub out_asset: Coin,
    /// Total number of remaining out tokens at the time of update.
    pub out_remaining: Uint128,
    /// Denom of the `token_in`.
    pub in_denom: String,
    /// Total number of `token_in` on the buy side at latest state.
    pub in_supply: Uint128,
    /// Total number of `token_in` spent at latest state.
    pub spent_in: Uint128,
    /// Total number of shares minted.
    pub shares: Uint128,
    /// Start block when the token emission starts. in nanos.
    pub start_time: Timestamp,
    /// End block when the token emission ends.
    pub end_time: Timestamp,
    /// Price at when latest distribution is triggered.
    pub current_streamed_price: Decimal,
    /// Status of the stream. Can be `Waiting`, `Active`, `Finalized`, `Paused` or `Canceled` for kill switch.
    pub status: Status,
    /// Block height when the stream was paused.
    pub pause_time: Option<Timestamp>,
}

#[cw_serde]
pub enum Status {
    /// Waiting for start date
    Waiting,
    Active,
    Finalized,
    Paused,
    Cancelled,
}
#[allow(clippy::too_many_arguments)]
impl Stream {
    pub fn new(
        name: String,
        treasury: Addr,
        stream_admin: Addr,
        url: Option<String>,
        out_asset: Coin,
        in_denom: String,
        start_time: Timestamp,
        end_time: Timestamp,
        last_updated_time: Timestamp,
    ) -> Self {
        Stream {
            name,
            treasury,
            stream_admin,
            url,
            dist_index: Decimal256::zero(),
            last_updated_time,
            start_time,
            end_time,
            pause_time: None,
            out_asset,
            out_remaining: Uint128::zero(),
            in_denom,
            in_supply: Uint128::zero(),
            spent_in: Uint128::zero(),
            shares: Uint128::zero(),
            current_streamed_price: Decimal::zero(),
            status: Status::Waiting,
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
            shares /= self.in_supply;
        }
        shares
    }

    pub fn is_paused(&self) -> bool {
        self.status == Status::Paused
    }

    pub fn is_cancelled(&self) -> bool {
        self.status == Status::Cancelled
    }

    pub fn is_killswitch_active(&self) -> bool {
        self.status == Status::Cancelled || self.status == Status::Paused
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
    /// Creator of the position.
    pub owner: Addr,
    /// Current amount of tokens in buy pool
    pub in_balance: Uint128,
    pub shares: Uint128,
    // Index is used to calculate the distribution a position has
    pub index: Decimal256,
    // Block height when the position was last updated.
    pub last_updated_block: u64,
    // Total amount of `token_out` purchased in tokens at latest calculation
    pub purchased: Uint128,
    // Pending purchased accumulates purchases after decimal truncation
    pub pending_purchase: Decimal256,
    // Total amount of `token_in` spent tokens at latest calculation
    pub spent: Uint128,
    // Operator can update position
    pub operator: Option<Addr>,
}

impl Position {
    pub fn new(
        owner: Addr,
        in_balance: Uint128,
        shares: Uint128,
        index: Option<Decimal256>,
        last_updated_block: u64,
        operator: Option<Addr>,
    ) -> Self {
        Position {
            owner,
            in_balance,
            shares,
            index: index.unwrap_or_default(),
            last_updated_block,
            purchased: Uint128::zero(),
            pending_purchase: Decimal256::zero(),
            spent: Uint128::zero(),
            operator,
        }
    }
}

// Position (stream_id, owner_addr) -> Position
pub const POSITIONS: Map<(StreamId, &Addr), Position> = Map::new("positions");

#[cfg(test)]
#[test]
fn test_compute_shares_amount() {
    let mut stream = Stream::new(
        "test".to_string(),
        Addr::unchecked("treasury"),
        Addr::unchecked("stream_admin"),
        Some("url".to_string()),
        Coin {
            denom: "out_denom".to_string(),
            amount: Uint128::from(100u128),
        },
        "in_denom".to_string(),
        Timestamp::from_seconds(0),
        Timestamp::from_seconds(100),
        Timestamp::from_seconds(0),
    );

    // add new shares
    let shares = stream.compute_shares_amount(Uint128::from(100u128), false);
    assert_eq!(shares, Uint128::from(100u128));
    stream.in_supply = Uint128::from(100u128);
    stream.shares = shares;

    // add new shares
    stream.shares += stream.compute_shares_amount(Uint128::from(100u128), false);
    stream.in_supply += Uint128::from(100u128);
    assert_eq!(stream.shares, Uint128::from(200u128));

    // add new shares
    stream.shares += stream.compute_shares_amount(Uint128::from(250u128), false);
    assert_eq!(stream.shares, Uint128::from(450u128));
    stream.in_supply += Uint128::from(250u128);

    // remove shares
    stream.shares -= stream.compute_shares_amount(Uint128::from(100u128), true);
    assert_eq!(stream.shares, Uint128::from(350u128));
    stream.in_supply -= Uint128::from(100u128);
}
