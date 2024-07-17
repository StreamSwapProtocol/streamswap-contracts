use crate::factory::CreatePool;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal, Decimal256, Timestamp, Uint128};
use cw_vesting::msg::InstantiateMsg as VestingInstantiateMsg;
use std::ops::Mul;

#[cw_serde]
pub struct Stream {
    /// Name of the stream.
    pub name: String,
    /// Destination for the earned token_in.
    pub treasury: Addr,
    /// Admin-Creator of the stream.
    pub stream_admin: Addr,
    /// URL for more information about the stream.
    pub url: Option<String>,
    /// Proportional distribution variable to calculate the distribution of in token_out to buyers.
    pub dist_index: Decimal256,
    /// Last updated block of stream.
    pub last_updated: Timestamp,
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
    /// Price at when latest distribution is triggered.
    pub current_streamed_price: Decimal,
    /// Status of the stream. Can be `Waiting`, `Active`, `Finalized`, `Paused` or `Canceled` for kill switch.
    pub status: StreamStatus,
    /// Create Pool message
    pub create_pool: Option<CreatePool>,
    /// Vesting configuration
    pub vesting: Option<VestingInstantiateMsg>,
}

#[cw_serde]
pub enum Status {
    Waiting,
    Boothstraping,
    Active,
    Ended,
    Finalized,
    Cancelled,
}

#[cw_serde]
pub struct StreamStatus {
    pub status: Status,
    pub bootstraping_start_time: Timestamp,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
}

impl StreamStatus {
    pub fn new(
        bootstraping_start_time: Timestamp,
        start_time: Timestamp,
        end_time: Timestamp,
    ) -> Self {
        StreamStatus {
            status: Status::Waiting,
            bootstraping_start_time,
            start_time,
            end_time,
        }
    }
    pub fn update_status(&mut self, now: Timestamp) {
        // If bootstraping time is not reached yet, keep the status as Waiting
        // If bootstraping time is reached, change the status to Boothstraping
        // If start time is reached, change the status to Active
        // If end time is reached, change the status to Ended
        if now < self.bootstraping_start_time {
            self.status = Status::Waiting;
        }
        if now >= self.bootstraping_start_time && now < self.start_time {
            self.status = Status::Boothstraping;
        }
        if now >= self.start_time && now < self.end_time {
            self.status = Status::Active;
        }
        if now >= self.end_time {
            self.status = Status::Ended;
        }
    }
}

impl Stream {
    pub fn new(
        name: String,
        treasury: Addr,
        stream_admin: Addr,
        url: Option<String>,
        out_asset: Coin,
        in_denom: String,
        boothstraping_start_time: Timestamp,
        start_time: Timestamp,
        end_time: Timestamp,
        last_updated: Timestamp,
        create_pool: Option<CreatePool>,
        vesting: Option<VestingInstantiateMsg>,
    ) -> Self {
        Stream {
            name,
            treasury,
            stream_admin,
            url,
            dist_index: Decimal256::zero(),
            last_updated,
            out_asset: out_asset.clone(),
            out_remaining: out_asset.amount,
            in_denom,
            in_supply: Uint128::zero(),
            spent_in: Uint128::zero(),
            shares: Uint128::zero(),
            current_streamed_price: Decimal::zero(),
            status: StreamStatus::new(boothstraping_start_time, start_time, end_time),
            create_pool,
            vesting,
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

    pub fn update_status(&mut self, now: Timestamp) {
        self.status.update_status(now);
    }
}
