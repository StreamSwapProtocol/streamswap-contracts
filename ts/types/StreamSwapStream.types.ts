/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.7.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export type Timestamp = Uint64;
export type Uint64 = string;
export type Schedule = "saturating_linear" | {
  piecewise_linear: [number, Uint128][];
};
export type Uint128 = string;
export type PoolConfig = {
  concentrated_liquidity: {
    out_amount_clp: Uint256;
  };
};
export type Uint256 = string;
export type Binary = string;
export interface InstantiateMsg {
  bootstraping_start_time: Timestamp;
  creator_vesting?: VestingConfig | null;
  end_time: Timestamp;
  in_denom: string;
  name: string;
  out_asset: Coin;
  pool_config?: PoolConfig | null;
  salt: Binary;
  start_time: Timestamp;
  stream_admin: string;
  subscriber_vesting?: VestingConfig | null;
  threshold?: Uint256 | null;
  tos_version: string;
  treasury: string;
  url?: string | null;
}
export interface VestingConfig {
  schedule: Schedule;
  unbonding_duration_seconds: number;
  vesting_duration_seconds: number;
}
export interface Coin {
  amount: Uint128;
  denom: string;
  [k: string]: unknown;
}
export type ExecuteMsg = {
  sync_stream: {};
} | {
  subscribe: {};
} | {
  withdraw: {
    cap?: Uint256 | null;
  };
} | {
  sync_position: {};
} | {
  finalize_stream: {
    create_pool?: CreatePool | null;
    new_treasury?: string | null;
    salt?: Binary | null;
  };
} | {
  exit_stream: {
    salt?: Binary | null;
  };
} | {
  cancel_stream: {};
} | {
  stream_admin_cancel: {};
};
export type CreatePool = {
  concentrated_liquidity: {
    lower_tick: number;
    spread_factor: string;
    tick_spacing: number;
    upper_tick: number;
  };
};
export type QueryMsg = {
  params: {};
} | {
  stream: {};
} | {
  position: {
    owner: string;
  };
} | {
  list_positions: {
    limit?: number | null;
    start_after?: string | null;
  };
} | {
  average_price: {};
} | {
  last_streamed_price: {};
} | {
  to_s: {
    addr?: string | null;
  };
} | {
  creator_vesting: {};
} | {
  subscriber_vesting: {
    addr: string;
  };
};
export type Decimal256 = string;
export interface AveragePriceResponse {
  average_price: Decimal256;
}
export type String = string;
export interface LatestStreamedPriceResponse {
  current_streamed_price: Decimal256;
}
export interface PositionsResponse {
  positions: PositionResponse[];
}
export interface PositionResponse {
  exit_date: Timestamp;
  in_balance: Uint256;
  index: Decimal256;
  last_updated: Timestamp;
  owner: string;
  pending_purchase: Decimal256;
  purchased: Uint256;
  shares: Uint256;
  spent: Uint256;
}
export type Addr = string;
export interface Params {
  accepted_in_denoms: string[];
  exit_fee_percent: Decimal256;
  fee_collector: Addr;
  min_bootstrapping_duration: number;
  min_stream_duration: number;
  min_waiting_duration: number;
  protocol_admin: Addr;
  stream_contract_code_id: number;
  stream_creation_fee: Coin;
  tos_version: string;
  vesting_code_id: number;
}
export type Status = "waiting" | "bootstrapping" | "active" | "ended" | {
  finalized: FinalizedStatus;
} | "cancelled";
export type FinalizedStatus = "threshold_reached" | "threshold_not_reached";
export interface StreamResponse {
  current_streamed_price: Decimal256;
  dist_index: Decimal256;
  end_time: Timestamp;
  in_denom: string;
  in_supply: Uint256;
  last_updated: Timestamp;
  out_asset: Coin;
  out_remaining: Uint256;
  shares: Uint256;
  spent_in: Uint256;
  start_time: Timestamp;
  status: Status;
  stream_admin: string;
  treasury: string;
  url?: string | null;
}