use crate::controller::{PoolConfig, VestingConfig};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal256, Timestamp, Uint256};
use streamswap_utils::to_uint256;

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
    pub out_remaining: Uint256,
    /// In denom of the stream
    pub in_denom: String,
    /// In supply of the stream
    pub in_supply: Uint256,
    /// Spent in of the stream, the total amount of in assets spent
    /// At any time spent_in + in_supply = total in assets
    pub spent_in: Uint256,
    /// Shares of the stream, used to calculate the amount of out assets to be distributed among subscribers
    pub shares: Uint256,
    /// Current streamed price, the price of in asset in out asset
    pub current_streamed_price: Decimal256,
    /// Status info of the stream
    pub status_info: StatusInfo,
    /// Pool Configuration for the pre stream
    pub pool_config: Option<PoolConfig>,
    /// Subscriber Vesting configuration, used to create a vesting contract for subscribers once the stream ends
    pub subscriber_vesting: Option<VestingConfig>,
    /// Creator Vesting configuration, used to create a vesting contract for creator once the stream ends
    pub creator_vesting: Option<VestingConfig>,
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

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Status::Waiting => write!(f, "Waiting"),
            Status::Bootstrapping => write!(f, "Bootstrapping"),
            Status::Active => write!(f, "Active"),
            Status::Ended => write!(f, "Ended"),
            Status::Finalized => write!(f, "Finalized"),
            Status::Cancelled => write!(f, "Cancelled"),
        }
    }
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
        pool_config: Option<PoolConfig>,
        subscriber_vesting: Option<VestingConfig>,
        creator_vesting: Option<VestingConfig>,
    ) -> Self {
        Stream {
            name,
            treasury,
            stream_admin,
            url,
            dist_index: Decimal256::zero(),
            out_asset: out_asset.clone(),
            out_remaining: to_uint256(out_asset.amount),
            in_denom,
            in_supply: Uint256::zero(),
            spent_in: Uint256::zero(),
            shares: Uint256::zero(),
            current_streamed_price: Decimal256::zero(),
            status_info: StatusInfo::new(now, bootstrapping_start_time, start_time, end_time),
            pool_config,
            subscriber_vesting,
            creator_vesting,
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
