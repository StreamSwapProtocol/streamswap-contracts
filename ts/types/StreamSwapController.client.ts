/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.7.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
import { Decimal256, Uint128, InstantiateMsg, Coin, ExecuteMsg, Timestamp, Uint64, Schedule, PoolConfig, Uint256, Binary, CreateStreamMsg, VestingConfig, QueryMsg, Boolean, StreamsResponse, StreamResponse, Addr, Params } from "./StreamSwapController.types";
export interface StreamSwapControllerReadOnlyInterface {
  contractAddress: string;
  params: () => Promise<Params>;
  freezestate: () => Promise<Boolean>;
  lastStreamId: () => Promise<Uint64>;
  listStreams: ({
    limit,
    startAfter
  }: {
    limit?: number;
    startAfter?: number;
  }) => Promise<StreamsResponse>;
}
export class StreamSwapControllerQueryClient implements StreamSwapControllerReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;

  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.params = this.params.bind(this);
    this.freezestate = this.freezestate.bind(this);
    this.lastStreamId = this.lastStreamId.bind(this);
    this.listStreams = this.listStreams.bind(this);
  }

  params = async (): Promise<Params> => {
    return this.client.queryContractSmart(this.contractAddress, {
      params: {}
    });
  };
  freezestate = async (): Promise<Boolean> => {
    return this.client.queryContractSmart(this.contractAddress, {
      freezestate: {}
    });
  };
  lastStreamId = async (): Promise<Uint64> => {
    return this.client.queryContractSmart(this.contractAddress, {
      last_stream_id: {}
    });
  };
  listStreams = async ({
    limit,
    startAfter
  }: {
    limit?: number;
    startAfter?: number;
  }): Promise<StreamsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      list_streams: {
        limit,
        start_after: startAfter
      }
    });
  };
}
export interface StreamSwapControllerInterface extends StreamSwapControllerReadOnlyInterface {
  contractAddress: string;
  sender: string;
  updateParams: ({
    acceptedInDenoms,
    exitFeePercent,
    feeCollector,
    minBootstrappingDuration,
    minStreamDuration,
    minWaitingDuration,
    streamCreationFee
  }: {
    acceptedInDenoms?: string[];
    exitFeePercent?: Decimal256;
    feeCollector?: string;
    minBootstrappingDuration?: number;
    minStreamDuration?: number;
    minWaitingDuration?: number;
    streamCreationFee?: Coin;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  createStream: ({
    msg
  }: {
    msg: CreateStreamMsg;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  freeze: (fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  unfreeze: (fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
}
export class StreamSwapControllerClient extends StreamSwapControllerQueryClient implements StreamSwapControllerInterface {
  client: SigningCosmWasmClient;
  sender: string;
  contractAddress: string;

  constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
    super(client, contractAddress);
    this.client = client;
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.updateParams = this.updateParams.bind(this);
    this.createStream = this.createStream.bind(this);
    this.freeze = this.freeze.bind(this);
    this.unfreeze = this.unfreeze.bind(this);
  }

  updateParams = async ({
    acceptedInDenoms,
    exitFeePercent,
    feeCollector,
    minBootstrappingDuration,
    minStreamDuration,
    minWaitingDuration,
    streamCreationFee
  }: {
    acceptedInDenoms?: string[];
    exitFeePercent?: Decimal256;
    feeCollector?: string;
    minBootstrappingDuration?: number;
    minStreamDuration?: number;
    minWaitingDuration?: number;
    streamCreationFee?: Coin;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_params: {
        accepted_in_denoms: acceptedInDenoms,
        exit_fee_percent: exitFeePercent,
        fee_collector: feeCollector,
        min_bootstrapping_duration: minBootstrappingDuration,
        min_stream_duration: minStreamDuration,
        min_waiting_duration: minWaitingDuration,
        stream_creation_fee: streamCreationFee
      }
    }, fee, memo, _funds);
  };
  createStream = async ({
    msg
  }: {
    msg: CreateStreamMsg;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      create_stream: {
        msg
      }
    }, fee, memo, _funds);
  };
  freeze = async (fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      freeze: {}
    }, fee, memo, _funds);
  };
  unfreeze = async (fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      unfreeze: {}
    }, fee, memo, _funds);
  };
}