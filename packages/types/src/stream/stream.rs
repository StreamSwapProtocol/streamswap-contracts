use crate::factory::CreatePool;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal, Decimal256, Timestamp, Uint128};
use cw_vesting::msg::InstantiateMsg as VestingInstantiateMsg;

#[cw_serde]
pub struct Stream {
    /// Name of the stream
    pub name: String,
    /// Treasury address, where the stream creator can withdraw the in assets at the end of the stream
    pub treasury: Addr,
    /// Stream admin address, where the stream creator can manage the stream, like canceling it in waiting status
    /// or finalizing it in ended status
    pub stream_admin: Addr,
    /// URL of the stream
    pub url: Option<String>,
    /// Distribution index, used to calculate the amount of out assets to be distributed
    pub dist_index: Decimal256,
    /// Out asset of the stream
    pub out_asset: Coin,
    /// Remaining out asset to be distributed
    pub out_remaining: Uint128,
    /// In denom of the stream
    pub in_denom: String,
    /// In supply of the stream
    pub in_supply: Uint128,
    /// Spent in of the stream, the total amount of in assets spent
    /// At any time spent_in + in_supply = total in assets
    pub spent_in: Uint128,
    /// Shares of the stream, used to calculate the amount of out assets to be distributed among subscribers
    pub shares: Uint128,
    /// Current streamed price, the price of in asset in out asset
    pub current_streamed_price: Decimal,
    /// Status info of the stream
    pub status_info: StatusInfo,
    /// Create pool message, used to create a pool for the stream once the stream ends
    pub create_pool: Option<CreatePool>,
    /// Vesting configuration, used to create a vesting contract for subscribers once the stream ends
    pub vesting: Option<VestingInstantiateMsg>,
}

#[cw_serde]
pub enum Status {
    /// Waiting status is when the stream is created. In this status, no one can interact with the stream.
    Waiting,
    /// Bootstrapping status is when the stream is bootstrapping.
    /// In this status, subscriber and withdraw are permitted. But no spending is allowed on each side
    Bootstrapping,
    /// Active status is when the stream is active. In this status, spending is allowed on each side.
    Active,
    /// Ended status is when the stream is ended.
    /// In this status, Subscriber can exit the stream, creator can finalize and collect accumulated in assets.
    Ended,
    /// Finalized status is when the stream is finalized. In this status, Subscriber can exit the stream.
    Finalized,
    /// Cancelled status is when the stream is cancelled.
    /// In this status, Subscriber can exit the stream and collect full in assets.
    /// Creator can collect full out assets.
    Cancelled,
}

#[cw_serde]
pub struct StatusInfo {
    /// Status of the stream
    pub status: Status,
    /// Bootstrapping start time of the stream
    pub bootstrapping_start_time: Timestamp,
    /// Start time of the stream
    pub start_time: Timestamp,
    /// End time of the stream
    pub end_time: Timestamp,
    /// Last updated time of the status info
    pub last_updated: Timestamp,
}

impl StatusInfo {
    pub fn new(
        now: Timestamp,
        bootstrapping_start_time: Timestamp,
        start_time: Timestamp,
        end_time: Timestamp,
    ) -> Self {
        StatusInfo {
            status: Status::Waiting,
            bootstrapping_start_time,
            start_time,
            end_time,
            last_updated: now,
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
        create_pool: Option<CreatePool>,
        vesting: Option<VestingInstantiateMsg>,
    ) -> Self {
        Stream {
            name,
            treasury,
            stream_admin,
            url,
            dist_index: Decimal256::zero(),
            out_asset: out_asset.clone(),
            out_remaining: out_asset.amount,
            in_denom,
            in_supply: Uint128::zero(),
            spent_in: Uint128::zero(),
            shares: Uint128::zero(),
            current_streamed_price: Decimal::zero(),
            status_info: StatusInfo::new(now, bootstrapping_start_time, start_time, end_time),
            create_pool,
            vesting,
        }
    }

    pub fn is_active(&self) -> bool {
        self.status_info.status == Status::Active
    }

    pub fn is_finalized(&self) -> bool {
        self.status_info.status == Status::Finalized
    }

    pub fn is_waiting(&self) -> bool {
        self.status_info.status == Status::Waiting
    }

    pub fn is_cancelled(&self) -> bool {
        self.status_info.status == Status::Cancelled
    }

    pub fn is_bootstrapping(&self) -> bool {
        self.status_info.status == Status::Bootstrapping
    }

    pub fn is_ended(&self) -> bool {
        self.status_info.status == Status::Ended
    }
}
