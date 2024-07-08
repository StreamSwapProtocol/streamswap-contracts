#[cfg(test)]
mod shares {
    use cosmwasm_std::{Addr, Coin, Timestamp, Uint128};
    use streamswap_types::stream::Stream;

    #[test]
    fn test_compute_shares_amount() {
        let mut stream = Stream::new(
            "test".to_string(),
            Addr::unchecked("treasury"),
            Addr::unchecked("stream_admin"),
            Some("url".to_string()),
            Coin {
                denom: "out_denom".to_string(),
                amount: Uint128::from(100u128),
            },
            "in_denom".to_string(),
            Timestamp::from_seconds(0),
            Timestamp::from_seconds(100),
            Timestamp::from_seconds(0),
            None,
            None,
        );

        // add new shares
        let shares = stream.compute_shares_amount(Uint128::from(100u128), false);
        assert_eq!(shares, Uint128::from(100u128));
        stream.in_supply = Uint128::from(100u128);
        stream.shares = shares;

        // add new shares
        stream.shares += stream.compute_shares_amount(Uint128::from(100u128), false);
        stream.in_supply += Uint128::from(100u128);
        assert_eq!(stream.shares, Uint128::from(200u128));

        // add new shares
        stream.shares += stream.compute_shares_amount(Uint128::from(250u128), false);
        assert_eq!(stream.shares, Uint128::from(450u128));
        stream.in_supply += Uint128::from(250u128);

        // remove shares
        stream.shares -= stream.compute_shares_amount(Uint128::from(100u128), true);
        assert_eq!(stream.shares, Uint128::from(350u128));
        stream.in_supply -= Uint128::from(100u128);
    }
}
