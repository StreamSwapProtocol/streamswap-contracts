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

/// This function is used to build the MsgCreatePosition for the initial pool position
pub fn build_create_initial_position_msg(
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

#[cfg(test)]
mod pool_test {
    use super::*;
    use osmosis_std::types::cosmos::base::v1beta1::Coin;

    #[test]
    fn test_calculate_in_amount_clp() {
        let out_amount = Uint256::from(100u64);
        let pool_out_amount = Uint256::from(1000u64);
        let spent_in = Uint256::from(10u64);

        let result = calculate_in_amount_clp(out_amount, pool_out_amount, spent_in);
        let expected = Uint256::from(100u64);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_create_initial_pool_position_msg() {
        let pool_id = 1;
        let treasury = "treasury_address";
        let stream_in_denom = "in_denom";
        let in_clp = Uint256::from(1000u64);
        let stream_out_asset_denom = "out_denom";
        let pool_out_amount_clp = Uint256::from(2000u64);

        let result = build_create_initial_position_msg(
            pool_id,
            treasury,
            stream_in_denom,
            in_clp,
            stream_out_asset_denom,
            pool_out_amount_clp,
        );

        let expected = MsgCreatePosition {
            pool_id,
            sender: treasury.to_string(),
            lower_tick: 0,
            upper_tick: i64::MAX,
            tokens_provided: vec![
                Coin {
                    denom: stream_in_denom.to_string(),
                    amount: in_clp.to_string(),
                },
                Coin {
                    denom: stream_out_asset_denom.to_string(),
                    amount: pool_out_amount_clp.to_string(),
                },
            ],
            token_min_amount0: "0".to_string(),
            token_min_amount1: "0".to_string(),
        };

        assert_eq!(result, expected);
    }
}
