use cosmwasm_std::{from_json, to_json_binary, Addr, Api, Binary, BlockInfo, Querier, Storage, Empty, Coin, CustomMsg, CustomQuery};
use cw_multi_test::{error::AnyResult, AppResponse, CosmosRouter, Stargate, Contract, ContractWrapper,Module, AcceptingModule};
use crate::contract::{execute, instantiate, query};

pub fn contract_streamswap() -> Box<dyn Contract<Empty>> {
    Box::new(ContractWrapper::new(execute, instantiate, query))
}

pub struct MyStargateKeeper {}

impl Stargate for MyStargateKeeper {
    fn execute<ExecC, QueryC>(
        &self,
        _api: &dyn Api,
        storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        sender: Addr,
        type_url: String,
        value: Binary,
    ) -> AnyResult<AppResponse> {
        /*
        if type_url == *"/osmosis.concentratedliquidity.poolmodel.concentrated.v1beta1.MsgCreateConcentratedPool" {
            let parsed_msg: Result<MsgCreateConcentratedPool, DecodeError> = Message::decode(value.as_slice());
            if let Ok(msg) = parsed_msg {
                let collection = Collection {
                    denom: Some(Denom {
                        creator: sender.to_string(),
                        data: msg.data,
                        name: msg.name,
                        id: msg.id,
                        preview_uri: msg.preview_uri,
                        description: msg.description,
                        schema: msg.schema,
                        symbol: msg.symbol,
                        uri: msg.uri,
                        uri_hash: msg.uri_hash,
                        royalty_receivers: msg.royalty_receivers,
                    }),
                    onfts: vec![],
                };
                let key = format!("collections:{}:{}", COLLECTION_PREFIX, sender);
                let serialized_collection =
                    to_json_binary(&collection).expect("Failed to serialize Collection");
                storage.set(key.as_bytes(), &serialized_collection);
            }
        }

         */
        Ok(AppResponse::default())
    }

    fn query(
        &self,
        _api: &dyn Api,
        _storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        path: String,
        data: Binary,
    ) -> AnyResult<Binary> {
        /*
        if path == *"/OmniFlix.onft.v1beta1.Query/Params" {
            let params = QueryParamsResponse {
                params: Some(Params {
                    denom_creation_fee: Some(Coin {
                        denom: "uflix".to_string(),
                        amount: "1000000".to_string(),
                    }),
                }),
            };
            return Ok(to_json_binary(&params)?);
        }

         */
        Ok(data)
    }
}
