use crate::factory::CreatePool;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal, Decimal256, Fraction, Timestamp, Uint128};
use cw_vesting::msg::InstantiateMsg as VestingInstantiateMsg;
use std::ops::Mul;

#[cw_serde]
pub struct Stream {
    /// Name of the stream.
    pub name: String,
    /// Destination for the earned `token_in`.
    pub treasury: Addr,
    /// Admin-Creator of the stream.
    pub stream_admin: Addr,
    /// URL for more information about the stream.
    pub url: Option<String>,
    /// Proportional distribution variable to calculate the distribution of `token_out` to buyers.
    pub dist_index: Decimal256,
    /// Last updated block of the stream.
    pub last_updated: Timestamp,
    /// Denom of the `token_out`.
    pub out_asset: Coin,
    /// Total number of remaining `token_out` at the time of update.
    pub out_remaining: Uint128,
    /// Denom of the `token_in`.
    pub in_denom: String,
    /// Total number of `token_in` on the buy side at the latest state.
    pub in_supply: Uint128,
    /// Total number of `token_in` spent at the latest state.
    pub spent_in: Uint128,
    /// Total number of shares minted.
    pub shares: Uint128,
    /// Price at which the latest distribution is triggered.
    pub current_streamed_price: Decimal,
    /// Status of the stream. Can be `Waiting`, `Active`, `Finalized`, `Paused` or `Canceled` for kill switch.
    pub status: StreamStatus,
    /// Create Pool message.
    pub create_pool: Option<CreatePool>,
    /// Vesting configuration.
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
        // If bootstrapping time is not reached yet, keep the status as Waiting.
        // If bootstrapping time is reached, change the status to Bootstrapping.
        // If start time is reached, change the status to Active.
        // If end time is reached, change the status to Ended.
        // If the stream is already finalized or cancelled, do not change the status.
        if self.status == Status::Finalized || self.status == Status::Cancelled {
            return;
        }
        if now < self.bootstrapping_start_time {
            self.status = Status::Waiting;
        } else if now >= self.bootstrapping_start_time && now < self.start_time {
            self.status = Status::Bootstrapping;
        } else if now >= self.start_time && now < self.end_time {
            self.status = Status::Active;
        } else if now >= self.end_time {
            self.status = Status::Ended;
        }
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
            status: StreamStatus::new(now, bootstrapping_start_time, start_time, end_time),
            create_pool,
            vesting,
        }
    }

    // Compute amount of shares that should be minted for a new subscription amount.
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

    pub fn update(&mut self, now: Timestamp) {
        let diff = calculate_diff(
            self.status.start_time,
            self.status.end_time,
            self.last_updated,
            now,
        );

        let mut new_distribution_balance = Uint128::zero();

        // If no in balance in the contract, no need to update.
        // If diff not changed, this means either stream not started or no in balance so far.

        if !self.shares.is_zero() && !diff.is_zero() {
            // New distribution balance is the amount of in tokens that has been distributed since last update.
            // Distribution is linear for now.
            new_distribution_balance = self
                .out_remaining
                .multiply_ratio(diff.numerator(), diff.denominator());
            // Spent in tokens is the amount of in tokens that has been spent since last update.
            // Spending is linear and goes to zero at the end of the stream.
            let spent_in = self
                .in_supply
                .multiply_ratio(diff.numerator(), diff.denominator());

            // Increase total spent_in of the stream.
            self.spent_in += spent_in;
            // Decrease in_supply of the stream.
            self.in_supply -= spent_in;

            // If no new distribution balance, no need to update the price, out_remaining and dist_index.
            if !new_distribution_balance.is_zero() {
                // Decrease amount to be distributed of the stream.
                self.out_remaining -= new_distribution_balance;
                // Update distribution index. A position's share of the distribution is calculated by
                // multiplying the share by the distribution index.
                self.dist_index += Decimal256::from_ratio(new_distribution_balance, self.shares);
                self.current_streamed_price =
                    Decimal::from_ratio(spent_in, new_distribution_balance);
            }
        }

        self.last_updated = now;

        // Update status of the stream.
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
    // diff = (now - last_updated) / (end_time - last_updated)
    // -------Waiting-------|----Bootstrapping----|-------Active-------|-------Ended-------|

    // TODO - More readable
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

    // Add Stream creation validate fuction
}
