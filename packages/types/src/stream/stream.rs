use crate::factory::CreatePool;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal, Decimal256, Fraction, Timestamp, Uint128};
use cw_vesting::msg::InstantiateMsg as VestingInstantiateMsg;
use std::ops::Mul;

#[cw_serde]
pub struct Stream {
    pub name: String,
    pub treasury: Addr,
    pub stream_admin: Addr,
    pub url: Option<String>,
    pub dist_index: Decimal256,
    pub last_updated: Timestamp,
    pub out_asset: Coin,
    pub out_remaining: Uint128,
    pub in_denom: String,
    pub in_supply: Uint128,
    pub spent_in: Uint128,
    pub shares: Uint128,
    pub current_streamed_price: Decimal,
    pub status: StreamStatus,
    pub create_pool: Option<CreatePool>,
    pub vesting: Option<VestingInstantiateMsg>,
}

#[cw_serde]
pub enum Status {
    Waiting,
    Bootstrapping,
    Active,
    Ended,
    Finalized,
    Cancelled,
}

#[cw_serde]
pub struct StreamStatus {
    pub status: Status,
    pub bootstrapping_start_time: Timestamp,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub last_updated: Timestamp,
}

impl StreamStatus {
    pub fn new(
        now: Timestamp,
        bootstrapping_start_time: Timestamp,
        start_time: Timestamp,
        end_time: Timestamp,
    ) -> Self {
        StreamStatus {
            status: Status::Waiting,
            bootstrapping_start_time,
            start_time,
            end_time,
            last_updated: now,
        }
    }

    pub fn update_status(&mut self, now: Timestamp) {
        if matches!(self.status, Status::Finalized | Status::Cancelled) {
            return;
        }
        self.status = match now {
            _ if now < self.bootstrapping_start_time => Status::Waiting,
            _ if now >= self.bootstrapping_start_time && now < self.start_time => {
                Status::Bootstrapping
            }
            _ if now >= self.start_time && now < self.end_time => Status::Active,
            _ if now >= self.end_time => Status::Ended,
            _ => self.status.clone(),
        };
    }
}

impl Stream {
    pub fn new(
        now: Timestamp,
        name: String,
        treasury: Addr,
        stream_admin: Addr,
        url: Option<String>,
        out_asset: Coin,
        in_denom: String,
        bootstrapping_start_time: Timestamp,
        start_time: Timestamp,
        end_time: Timestamp,
        create_pool: Option<CreatePool>,
        vesting: Option<VestingInstantiateMsg>,
    ) -> Self {
        Stream {
            name,
            treasury,
            stream_admin,
            url,
            dist_index: Decimal256::zero(),
            last_updated: now,
            out_asset: out_asset.clone(),
            out_remaining: out_asset.amount,
            in_denom,
            in_supply: Uint128::zero(),
            spent_in: Uint128::zero(),
            shares: Uint128::zero(),
            current_streamed_price: Decimal::zero(),
            status: StreamStatus::new(now, bootstrapping_start_time, start_time, end_time),
            create_pool,
            vesting,
        }
    }

    pub fn compute_shares_amount(&self, amount_in: Uint128, round_up: bool) -> Uint128 {
        if self.shares.is_zero() || amount_in.is_zero() {
            return amount_in;
        }
        let shares = self.shares.mul(amount_in);
        if round_up {
            (shares + self.in_supply - Uint128::one()) / self.in_supply
        } else {
            shares / self.in_supply
        }
    }

    pub fn update_status(&mut self, now: Timestamp) {
        self.status.update_status(now);
    }

    pub fn update(&mut self, now: Timestamp) {
        let diff = calculate_diff(
            self.status.start_time,
            self.status.end_time,
            self.last_updated,
            now,
        );

        if !self.shares.is_zero() && !diff.is_zero() {
            let new_distribution_balance = self
                .out_remaining
                .multiply_ratio(diff.numerator(), diff.denominator());
            let spent_in = self
                .in_supply
                .multiply_ratio(diff.numerator(), diff.denominator());

            self.spent_in += spent_in;
            self.in_supply -= spent_in;

            if !new_distribution_balance.is_zero() {
                self.out_remaining -= new_distribution_balance;
                self.dist_index += Decimal256::from_ratio(new_distribution_balance, self.shares);
                self.current_streamed_price =
                    Decimal::from_ratio(spent_in, new_distribution_balance);
            }
        }

        self.last_updated = now;
        self.update_status(now);
    }

    pub fn is_active(&self) -> bool {
        self.status.status == Status::Active
    }

    pub fn is_finalized(&self) -> bool {
        self.status.status == Status::Finalized
    }

    pub fn is_waiting(&self) -> bool {
        self.status.status == Status::Waiting
    }

    pub fn is_cancelled(&self) -> bool {
        self.status.status == Status::Cancelled
    }

    pub fn is_bootstrapping(&self) -> bool {
        self.status.status == Status::Bootstrapping
    }

    pub fn is_ended(&self) -> bool {
        self.status.status == Status::Ended
    }
}

fn calculate_diff(
    start_time: Timestamp,
    end_time: Timestamp,
    mut last_updated: Timestamp,
    now: Timestamp,
) -> Decimal {
    if now < start_time || last_updated >= end_time {
        return Decimal::zero();
    }

    if last_updated < start_time {
        last_updated = start_time;
    }

    let now = if now > end_time { end_time } else { now };

    let numerator = now.nanos().saturating_sub(last_updated.nanos());
    let denominator = end_time.nanos().saturating_sub(last_updated.nanos());

    if denominator == 0 || numerator == 0 {
        Decimal::zero()
    } else {
        Decimal::from_ratio(numerator, denominator)
    }
}
