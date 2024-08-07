use crate::factory::CreatePool;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal, Decimal256, Timestamp, Uint128};
use cw_vesting::msg::InstantiateMsg as VestingInstantiateMsg;

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
    pub status_info: StatusInfo,
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
pub struct StatusInfo {
    pub status: Status,
    pub bootstrapping_start_time: Timestamp,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
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
            last_updated: now,
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
