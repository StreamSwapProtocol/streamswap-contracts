#[cfg(test)]
mod shares {
    use cosmwasm_std::{Coin, Timestamp, Uint128, Uint256};
    use streamswap_stream::stream::compute_shares_amount;
    use streamswap_types::stream::StreamState;

    #[test]
    fn test_compute_shares_amount() {
        let mut stream = StreamState::new(
            Timestamp::from_seconds(0),
            Coin {
                denom: "out_denom".to_string(),
                amount: Uint128::from(100u128),
            },
            "in_denom".to_string(),
            Timestamp::from_seconds(0),
            Timestamp::from_seconds(100),
            Timestamp::from_seconds(0),
            None,
        );

        // add new shares
        let shares = compute_shares_amount(&stream, Uint256::from(100u128), false);
        assert_eq!(shares, Uint256::from(100u128));
        stream.in_supply = Uint256::from(100u128);
        stream.shares = shares;

        // add new shares
        stream.shares += compute_shares_amount(&stream, Uint256::from(100u128), false);
        stream.in_supply += Uint256::from(100u128);
        assert_eq!(stream.shares, Uint256::from(200u128));

        // add new shares
        stream.shares += compute_shares_amount(&stream, Uint256::from(250u128), false);
        assert_eq!(stream.shares, Uint256::from(450u128));
        stream.in_supply += Uint256::from(250u128);

        // remove shares
        stream.shares -= compute_shares_amount(&stream, Uint256::from(100u128), false);
        assert_eq!(stream.shares, Uint256::from(350u128));
        stream.in_supply -= Uint256::from(100u128);
    }
}
