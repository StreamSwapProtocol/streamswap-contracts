/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.7.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { MsgExecuteContractEncodeObject } from "@cosmjs/cosmwasm-stargate";
import { MsgExecuteContract } from "cosmjs-types/cosmwasm/wasm/v1/tx";
import { toUtf8 } from "@cosmjs/encoding";
import { Timestamp, Uint64, Schedule, Uint128, PoolConfig, Uint256, Binary, InstantiateMsg, VestingConfig, Coin, ExecuteMsg, CreatePool, QueryMsg, Decimal256, AveragePriceResponse, String, LatestStreamedPriceResponse, PositionsResponse, PositionResponse, Addr, Params, Status, FinalizedStatus, StreamResponse } from "./StreamSwapStream.types";
export interface StreamSwapStreamMsg {
  contractAddress: string;
  sender: string;
  syncStream: (_funds?: Coin[]) => MsgExecuteContractEncodeObject;
  subscribe: (_funds?: Coin[]) => MsgExecuteContractEncodeObject;
  withdraw: ({
    cap
  }: {
    cap?: Uint256;
  }, _funds?: Coin[]) => MsgExecuteContractEncodeObject;
  syncPosition: (_funds?: Coin[]) => MsgExecuteContractEncodeObject;
  finalizeStream: ({
    createPool,
    newTreasury,
    salt
  }: {
    createPool?: CreatePool;
    newTreasury?: string;
    salt?: Binary;
  }, _funds?: Coin[]) => MsgExecuteContractEncodeObject;
  exitStream: ({
    salt
  }: {
    salt?: Binary;
  }, _funds?: Coin[]) => MsgExecuteContractEncodeObject;
  cancelStream: (_funds?: Coin[]) => MsgExecuteContractEncodeObject;
  streamAdminCancel: (_funds?: Coin[]) => MsgExecuteContractEncodeObject;
}
export class StreamSwapStreamMsgComposer implements StreamSwapStreamMsg {
  sender: string;
  contractAddress: string;

  constructor(sender: string, contractAddress: string) {
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.syncStream = this.syncStream.bind(this);
    this.subscribe = this.subscribe.bind(this);
    this.withdraw = this.withdraw.bind(this);
    this.syncPosition = this.syncPosition.bind(this);
    this.finalizeStream = this.finalizeStream.bind(this);
    this.exitStream = this.exitStream.bind(this);
    this.cancelStream = this.cancelStream.bind(this);
    this.streamAdminCancel = this.streamAdminCancel.bind(this);
  }

  syncStream = (_funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          sync_stream: {}
        })),
        funds: _funds
      })
    };
  };
  subscribe = (_funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          subscribe: {}
        })),
        funds: _funds
      })
    };
  };
  withdraw = ({
    cap
  }: {
    cap?: Uint256;
  }, _funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          withdraw: {
            cap
          }
        })),
        funds: _funds
      })
    };
  };
  syncPosition = (_funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          sync_position: {}
        })),
        funds: _funds
      })
    };
  };
  finalizeStream = ({
    createPool,
    newTreasury,
    salt
  }: {
    createPool?: CreatePool;
    newTreasury?: string;
    salt?: Binary;
  }, _funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          finalize_stream: {
            create_pool: createPool,
            new_treasury: newTreasury,
            salt
          }
        })),
        funds: _funds
      })
    };
  };
  exitStream = ({
    salt
  }: {
    salt?: Binary;
  }, _funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          exit_stream: {
            salt
          }
        })),
        funds: _funds
      })
    };
  };
  cancelStream = (_funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          cancel_stream: {}
        })),
        funds: _funds
      })
    };
  };
  streamAdminCancel = (_funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          stream_admin_cancel: {}
        })),
        funds: _funds
      })
    };
  };
}