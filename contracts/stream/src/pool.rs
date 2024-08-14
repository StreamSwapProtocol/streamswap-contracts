use cosmwasm_std::Uint256;
use osmosis_std::types::osmosis::concentratedliquidity::v1beta1::MsgCreatePosition;

/// This function is used to calculate the in amount of the pool
pub fn calculate_in_amount_clp(
    out_amount: Uint256,
    pool_out_amount: Uint256,
    spent_in: Uint256,
) -> Uint256 {
    pool_out_amount / out_amount * spent_in
}

/// This function is used to build the create position message for the initial pool position
pub fn build_create_initial_pool_position_msg(
    pool_id: u64,
    treasury: &str,
    stream_in_denom: &str,
    in_clp: Uint256,
    stream_out_asset_denom: &str,
    pool_out_amount_clp: Uint256,
) -> MsgCreatePosition {
    MsgCreatePosition {
        pool_id,
        sender: treasury.to_string(),
        lower_tick: 0,
        upper_tick: i64::MAX,
        tokens_provided: vec![
            osmosis_std::types::cosmos::base::v1beta1::Coin {
                denom: stream_in_denom.to_string(),
                amount: in_clp.to_string(),
            },
            osmosis_std::types::cosmos::base::v1beta1::Coin {
                denom: stream_out_asset_denom.to_string(),
                amount: pool_out_amount_clp.to_string(),
            },
        ],
        token_min_amount0: "0".to_string(),
        token_min_amount1: "0".to_string(),
    }
}
