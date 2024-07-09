use cosmwasm_std::{to_json_binary, Addr, Api, Binary, BlockInfo, Querier, Storage};
use cw_multi_test::error::anyhow;
use cw_multi_test::{error::AnyResult, AppResponse, CosmosRouter, Stargate};
use osmosis_std::shim::Any;
use osmosis_std::types::cosmos::base::v1beta1::Coin;
use osmosis_std::types::osmosis::concentratedliquidity::poolmodel::concentrated::v1beta1::MsgCreateConcentratedPool;
use osmosis_std::types::osmosis::concentratedliquidity::v1beta1::Pool;
use osmosis_std::types::osmosis::poolmanager::v1beta1::{NumPoolsResponse, Params, ParamsResponse};
use prost::{DecodeError, Message};
use schemars::_serde_json::to_vec;

pub struct MyStargateKeeper {}

impl Stargate for MyStargateKeeper {
    fn execute<ExecC, QueryC>(
        &self,
        _api: &dyn Api,
        storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        _sender: Addr,
        type_url: String,
        value: Binary,
    ) -> AnyResult<AppResponse> {
        if type_url == *"/osmosis.concentratedliquidity.poolmodel.concentrated.v1beta1.MsgCreateConcentratedPool" {
            let parsed_msg: Result<MsgCreateConcentratedPool, DecodeError> = Message::decode(value.as_slice());
            if let Ok(msg) = parsed_msg{
                let pool = Pool {
                  token0: msg.denom0.clone(),
                    token1: msg.denom1.clone(),
                    id: 1,
                    ..Default::default()
                };
                let key = format!("pools:{}", pool.id);
                let serialized_pool = to_json_binary(&pool).expect("Failed to serialize Pool");
                storage.set(key.as_bytes(), &serialized_pool);
            }
        }
        Ok(AppResponse::default())
    }

    fn query(
        &self,
        _api: &dyn Api,
        storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        path: String,
        _data: Binary,
    ) -> AnyResult<Binary> {
        match path.as_str() {
            "/osmosis.poolmanager.v1beta1.Query/Params" => {
                let params = ParamsResponse {
                    params: Some(Params {
                        pool_creation_fee: vec![Coin {
                            denom: "uosmo".to_string(),
                            amount: "1000000".to_string(),
                        }],
                        taker_fee_params: None,
                        authorized_quote_denoms: vec![],
                    }),
                };
                Ok(to_json_binary(&params)?)
            }
            "/osmosis.poolmanager.v1beta1.Query/NumPools" => {
                let res = NumPoolsResponse { num_pools: 1 };
                Ok(to_json_binary(&res)?)
            }
            "/osmosis.concentratedliquidity.v1beta1.Query/Pools" => {
                let key = "pools:".to_string();
                let pools = storage
                    .range(Some(key.as_bytes()), None, cosmwasm_std::Order::Ascending)
                    .map(|item| {
                        let value = item.1;
                        let pool: Pool =
                            Message::decode(value.as_slice()).expect("Failed to decode Pool");
                        pool
                    })
                    .collect::<Vec<Pool>>();
                let res =
                    osmosis_std::types::osmosis::concentratedliquidity::v1beta1::PoolsResponse {
                        pools: pools
                            .iter()
                            .map(|p| Any {
                                type_url: "/osmosis.concentratedliquidity.v1beta1.Pool".to_string(),
                                value: to_vec(p).unwrap(),
                            })
                            .collect::<Vec<_>>(),
                        pagination: None,
                    };
                Ok(to_json_binary(&res)?)
            }
            _ => Err(anyhow!("Unknown query path")),
        }
    }
}
