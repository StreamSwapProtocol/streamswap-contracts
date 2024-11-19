/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.7.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
import { Timestamp, Uint64, Schedule, Uint128, PoolConfig, Uint256, Binary, InstantiateMsg, VestingConfig, Coin, ExecuteMsg, CreatePool, QueryMsg, Decimal256, AveragePriceResponse, LatestStreamedPriceResponse, PositionsResponse, PositionResponse, Addr, Params, Status, StreamResponse } from "./StreamSwapStream.types";
export interface StreamSwapStreamReadOnlyInterface {
  contractAddress: string;
  params: () => Promise<Params>;
  stream: () => Promise<StreamResponse>;
  position: ({
    owner
  }: {
    owner: string;
  }) => Promise<PositionResponse>;
  listPositions: ({
    limit,
    startAfter
  }: {
    limit?: number;
    startAfter?: string;
  }) => Promise<PositionsResponse>;
  averagePrice: () => Promise<AveragePriceResponse>;
  lastStreamedPrice: () => Promise<LatestStreamedPriceResponse>;
  threshold: () => Promise<Uint128>;
}
export class StreamSwapStreamQueryClient implements StreamSwapStreamReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;

  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.params = this.params.bind(this);
    this.stream = this.stream.bind(this);
    this.position = this.position.bind(this);
    this.listPositions = this.listPositions.bind(this);
    this.averagePrice = this.averagePrice.bind(this);
    this.lastStreamedPrice = this.lastStreamedPrice.bind(this);
    this.threshold = this.threshold.bind(this);
  }

  params = async (): Promise<Params> => {
    return this.client.queryContractSmart(this.contractAddress, {
      params: {}
    });
  };
  stream = async (): Promise<StreamResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      stream: {}
    });
  };
  position = async ({
    owner
  }: {
    owner: string;
  }): Promise<PositionResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      position: {
        owner
      }
    });
  };
  listPositions = async ({
    limit,
    startAfter
  }: {
    limit?: number;
    startAfter?: string;
  }): Promise<PositionsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      list_positions: {
        limit,
        start_after: startAfter
      }
    });
  };
  averagePrice = async (): Promise<AveragePriceResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      average_price: {}
    });
  };
  lastStreamedPrice = async (): Promise<LatestStreamedPriceResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      last_streamed_price: {}
    });
  };
  threshold = async (): Promise<Uint128> => {
    return this.client.queryContractSmart(this.contractAddress, {
      threshold: {}
    });
  };
}
export interface StreamSwapStreamInterface extends StreamSwapStreamReadOnlyInterface {
  contractAddress: string;
  sender: string;
  syncStream: (fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  subscribe: (fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  withdraw: ({
    cap
  }: {
    cap?: Uint256;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  syncPosition: (fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  finalizeStream: ({
    createPool,
    newTreasury,
    salt
  }: {
    createPool?: CreatePool;
    newTreasury?: string;
    salt?: Binary;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  exitStream: ({
    salt
  }: {
    salt?: Binary;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  exitCancelled: (fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  cancelStream: (fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  cancelStreamWithThreshold: (fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  streamAdminCancel: (fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
}
export class StreamSwapStreamClient extends StreamSwapStreamQueryClient implements StreamSwapStreamInterface {
  client: SigningCosmWasmClient;
  sender: string;
  contractAddress: string;

  constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
    super(client, contractAddress);
    this.client = client;
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.syncStream = this.syncStream.bind(this);
    this.subscribe = this.subscribe.bind(this);
    this.withdraw = this.withdraw.bind(this);
    this.syncPosition = this.syncPosition.bind(this);
    this.finalizeStream = this.finalizeStream.bind(this);
    this.exitStream = this.exitStream.bind(this);
    this.exitCancelled = this.exitCancelled.bind(this);
    this.cancelStream = this.cancelStream.bind(this);
    this.cancelStreamWithThreshold = this.cancelStreamWithThreshold.bind(this);
    this.streamAdminCancel = this.streamAdminCancel.bind(this);
  }

  syncStream = async (fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      sync_stream: {}
    }, fee, memo, _funds);
  };
  subscribe = async (fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      subscribe: {}
    }, fee, memo, _funds);
  };
  withdraw = async ({
    cap
  }: {
    cap?: Uint256;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      withdraw: {
        cap
      }
    }, fee, memo, _funds);
  };
  syncPosition = async (fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      sync_position: {}
    }, fee, memo, _funds);
  };
  finalizeStream = async ({
    createPool,
    newTreasury,
    salt
  }: {
    createPool?: CreatePool;
    newTreasury?: string;
    salt?: Binary;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      finalize_stream: {
        create_pool: createPool,
        new_treasury: newTreasury,
        salt
      }
    }, fee, memo, _funds);
  };
  exitStream = async ({
    salt
  }: {
    salt?: Binary;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      exit_stream: {
        salt
      }
    }, fee, memo, _funds);
  };
  exitCancelled = async (fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      exit_cancelled: {}
    }, fee, memo, _funds);
  };
  cancelStream = async (fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      cancel_stream: {}
    }, fee, memo, _funds);
  };
  cancelStreamWithThreshold = async (fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      cancel_stream_with_threshold: {}
    }, fee, memo, _funds);
  };
  streamAdminCancel = async (fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      stream_admin_cancel: {}
    }, fee, memo, _funds);
  };
}