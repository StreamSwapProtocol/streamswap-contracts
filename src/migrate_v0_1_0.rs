use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    Addr, Decimal, Decimal256, Fraction, StdResult, Storage, Timestamp, Uint128, Uint256, Uint64,
};
use cw_storage_plus::{Item, Map};

use crate::state::{Status, Stream, StreamId, CONFIG, STREAMS};

#[cw_serde]
pub struct StreamV0_1_0 {
    /// Name of the stream.
    pub name: String,
    /// Destination for the earned token_in.
    pub treasury: Addr,
    /// URL for more information about the stream.
    pub url: Option<String>,
    /// Proportional distribution variable to calculate the distribution of in token_out to buyers.
    pub dist_index: Decimal256,
    /// last updated time of stream.
    pub last_updated: Timestamp,
    /// denom of the `token_out`.
    pub out_denom: String,
    /// total number of `token_out` to be sold during the continuous stream.
    pub out_supply: Uint128,
    /// total number of remaining out tokens at the time of update.
    pub out_remaining: Uint128,
    /// denom of the `token_in`.
    pub in_denom: String,
    /// total number of `token_in` on the buy side at latest state.
    pub in_supply: Uint128,
    /// total number of `token_in` spent at latest state.
    pub spent_in: Uint128,
    /// total number of shares minted.
    pub shares: Uint128,
    /// start time when the token emission starts. in nanos.
    pub start_time: Timestamp,
    /// end time when the token emission ends.
    pub end_time: Timestamp,
    /// price at when latest distribution is triggered.
    pub current_streamed_price: Decimal,
    /// Status of the stream. Can be `Waiting`, `Active`, `Finalized`, `Paused` or `Canceled` for kill switch.
    pub status: Status,
    /// Date when the stream was paused.
    pub pause_date: Option<Timestamp>,
    /// Stream creation fee denom. Saved under here to avoid any changes in config to efect existing streams.
    pub stream_creation_denom: String,
    /// Stream creation fee amount. Saved under here to avoid any changes in config to efect existing streams.
    pub stream_creation_fee: Uint128,
    /// Stream swap fee in percent. Saved under here to avoid any changes in config to efect existing streams.
    pub stream_exit_fee_percent: Decimal,
}

#[cw_serde]
pub struct PositionV0_1_0 {
    /// creator of the position.
    pub owner: Addr,
    /// current amount of tokens in buy pool
    pub in_balance: Uint128,
    pub shares: Uint128,
    // index is used to calculate the distribution a position has
    pub index: Decimal256,
    pub last_updated: Timestamp,
    // total amount of `token_out` purchased in tokens at latest calculation
    pub purchased: Uint128,
    // pending purchased accumulates purchases after decimal truncation
    pub pending_purchase: Decimal256,
    // total amount of `token_in` spent tokens at latest calculation
    pub spent: Uint128,
    // operator can update position
    pub operator: Option<Addr>,
}

#[cw_serde]
pub struct ConfigV0_1_0 {
    /// Minimum sale duration in unix seconds
    pub min_stream_seconds: Uint64,
    /// Minimum duration between start time and current time in unix seconds
    pub min_seconds_until_start_time: Uint64,
    /// Accepted in_denom to buy out_tokens
    pub accepted_in_denom: String,
    /// Accepted stream creation fee denom
    pub stream_creation_denom: String,
    /// Stream creation fee amount
    pub stream_creation_fee: Uint128,
    /// in/buy token exit fee in percent
    pub exit_fee_percent: Decimal256,
    /// Address of the fee collector
    pub fee_collector: Addr,
    /// protocol admin can pause streams in case of emergency.
    pub protocol_admin: Addr,
}

pub const OLD_STREAMS: Map<StreamId, StreamV0_1_0> = Map::new("streams");
pub const OLD_POSITIONS: Map<(StreamId, Addr), PositionV0_1_0> = Map::new("positions");
pub const OLD_CONFIG: Item<ConfigV0_1_0> = Item::new("config");

pub fn migrate_v0_1_0(storage: &mut dyn Storage) -> StdResult<()> {
    // Migrate the state from v0.1.0 to v0.1.4
    let old_streams = OLD_STREAMS
        .range(storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;

    // migrate streams
    for (id, stream) in old_streams {
        let new_stream = Stream {
            name: stream.name,
            treasury: stream.treasury,
            url: stream.url,
            dist_index: stream.dist_index,
            last_updated: stream.last_updated,
            out_denom: stream.out_denom,
            out_supply: Uint256::from_uint128(stream.out_supply),
            out_remaining: Uint256::from_uint128(stream.out_remaining),
            in_denom: stream.in_denom,
            in_supply: Uint256::from_uint128(stream.in_supply),
            spent_in: Uint256::from_uint128(stream.spent_in),
            shares: Uint256::from_uint128(stream.shares),
            start_time: stream.start_time,
            end_time: stream.end_time,
            current_streamed_price: Decimal256::from_ratio(
                stream.current_streamed_price.numerator(),
                stream.current_streamed_price.denominator(),
            ),
            status: stream.status,
            pause_date: stream.pause_date,
            stream_creation_denom: stream.stream_creation_denom,
            stream_creation_fee: stream.stream_creation_fee,
            stream_exit_fee_percent: Decimal256::from_ratio(
                stream.stream_exit_fee_percent.numerator(),
                stream.stream_exit_fee_percent.denominator(),
            ),
            tos_version: "".to_string(),
        };
        // Remove the old stream and save the new stream
        OLD_STREAMS.remove(storage, id);
        STREAMS.save(storage, id, &new_stream)?;
    }

    let old_config = OLD_CONFIG.load(storage)?;
    let new_config = crate::state::Config {
        min_stream_seconds: old_config.min_stream_seconds,
        min_seconds_until_start_time: old_config.min_seconds_until_start_time,
        accepted_in_denom: old_config.accepted_in_denom,
        stream_creation_denom: old_config.stream_creation_denom,
        stream_creation_fee: old_config.stream_creation_fee,
        exit_fee_percent: old_config.exit_fee_percent,
        fee_collector: old_config.fee_collector,
        protocol_admin: old_config.protocol_admin,
        tos_version: "".to_string(),
    };
    OLD_CONFIG.remove(storage);
    CONFIG.save(storage, &new_config)?;

    Ok(())
}

#[cfg(test)]
mod test_migrate {
    use crate::migrate_v0_1_0::{
        migrate_v0_1_0, ConfigV0_1_0, PositionV0_1_0, StreamV0_1_0, OLD_CONFIG, OLD_POSITIONS,
        OLD_STREAMS,
    };
    use crate::state::{Status, POSITIONS, STREAMS};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{Addr, Decimal, Decimal256, StdResult, Timestamp, Uint128, Uint64};
    use std::str::FromStr;

    #[test]
    fn test_migrate_v0_1_0() {
        // Create a mock DepsMut instance
        let mut deps = mock_dependencies();

        // Set up the old streams
        let old_streams = vec![
            (
                1u64,
                StreamV0_1_0 {
                    name: "Stream 1".to_string(),
                    treasury: Addr::unchecked("treasury1"),
                    url: Some("https://example.com/stream1".to_string()),
                    dist_index: Decimal256::from_str("0.0000000000088125").unwrap(),
                    last_updated: Timestamp::from_nanos(1721759663200729304),
                    out_denom: "token_out".to_string(),
                    out_supply: Uint128::new(79000000000),
                    out_remaining: Uint128::new(77236714240),
                    in_denom: "token_in".to_string(),
                    in_supply: Uint128::new(195633986821792674629),
                    spent_in: Uint128::new(4466013178207325371),
                    shares: Uint128::new(200100238743948046562),
                    start_time: Timestamp::from_nanos(1721759440000000000),
                    end_time: Timestamp::from_nanos(1721769440000000000),
                    current_streamed_price: Decimal::from_str("2544566918.67091152427620297")
                        .unwrap(),
                    status: Status::Active,
                    pause_date: None,
                    stream_creation_denom: "stream_creation_denom".to_string(),
                    stream_creation_fee: Uint128::new(10000000),
                    stream_exit_fee_percent: Decimal::from_str("0.1").unwrap(),
                },
            ),
            // Add more old streams if needed
        ];
        // Save the old streams and positions to storage
        for (id, stream) in old_streams.clone() {
            OLD_STREAMS
                .save(deps.as_mut().storage, id, &stream)
                .unwrap();
        }

        let old_config = ConfigV0_1_0 {
            min_stream_seconds: Uint64::new(86400),
            min_seconds_until_start_time: Uint64::new(86400),
            accepted_in_denom: "accepted_in_denom".to_string(),
            stream_creation_denom: "stream_creation_denom".to_string(),
            stream_creation_fee: Uint128::new(10000000),
            exit_fee_percent: Decimal256::percent(1),
            fee_collector: Addr::unchecked("fee_collector"),
            protocol_admin: Addr::unchecked("protocol_admin"),
        };

        OLD_CONFIG.save(deps.as_mut().storage, &old_config).unwrap();
        // Call the migrate_v0_1_0 function
        migrate_v0_1_0(&mut deps.storage).unwrap();

        // Assert that the old streams and positions have been migrated to the new format
        let new_streams: StdResult<Vec<_>> = STREAMS
            .range(
                deps.as_ref().storage,
                None,
                None,
                cosmwasm_std::Order::Ascending,
            )
            .collect();
        assert_eq!(new_streams.unwrap().len(), old_streams.len());

        let new_config = crate::state::CONFIG.load(deps.as_ref().storage).unwrap();
        assert_eq!(new_config.min_stream_seconds, old_config.min_stream_seconds);
        assert_eq!(
            new_config.min_seconds_until_start_time,
            old_config.min_seconds_until_start_time
        );
        assert_eq!(new_config.accepted_in_denom, old_config.accepted_in_denom);
        assert_eq!(
            new_config.stream_creation_denom,
            old_config.stream_creation_denom
        );
        assert_eq!(
            new_config.stream_creation_fee,
            old_config.stream_creation_fee
        );
        assert_eq!(new_config.exit_fee_percent, old_config.exit_fee_percent);
        assert_eq!(new_config.fee_collector, old_config.fee_collector);
        assert_eq!(new_config.protocol_admin, old_config.protocol_admin);
        // Assert that the tos_version has been set to an empty string
        assert_eq!(new_config.tos_version, "".to_string());
    }

    #[test]
    fn test_execute_migrate_position() {
        let mut deps = mock_dependencies();

        // Set up the old streams and positions
        let old_streams = vec![(
            1u64,
            StreamV0_1_0 {
                name: "Stream 1".to_string(),
                treasury: Addr::unchecked("treasury1"),
                url: Some("https://example.com/stream1".to_string()),
                dist_index: Decimal256::from_str("0.0000000000088125").unwrap(),
                last_updated: Timestamp::from_nanos(1721759663200729304),
                out_denom: "token_out".to_string(),
                out_supply: Uint128::new(79000000000),
                out_remaining: Uint128::new(77236714240),
                in_denom: "token_in".to_string(),
                in_supply: Uint128::new(195633986821792674629),
                spent_in: Uint128::new(4466013178207325371),
                shares: Uint128::new(200100238743948046562),
                start_time: Timestamp::from_nanos(1721759440000000000),
                end_time: Timestamp::from_nanos(1721769440000000000),
                current_streamed_price: Decimal::from_str("2544566918.67091152427620297").unwrap(),
                status: Status::Active,
                pause_date: None,
                stream_creation_denom: "stream_creation_denom".to_string(),
                stream_creation_fee: Uint128::new(10000000),
                stream_exit_fee_percent: Decimal::from_str("0.1").unwrap(),
            },
        )];
        // Save the old streams and positions to storage
        for (id, stream) in old_streams.clone() {
            OLD_STREAMS
                .save(deps.as_mut().storage, id, &stream)
                .unwrap();
        }

        let old_config = ConfigV0_1_0 {
            min_stream_seconds: Uint64::new(86400),
            min_seconds_until_start_time: Uint64::new(86400),
            accepted_in_denom: "accepted_in_denom".to_string(),
            stream_creation_denom: "stream_creation_denom".to_string(),
            stream_creation_fee: Uint128::new(10000000),
            exit_fee_percent: Decimal256::percent(1),
            fee_collector: Addr::unchecked("fee_collector"),
            protocol_admin: Addr::unchecked("protocol_admin"),
        };

        OLD_CONFIG.save(deps.as_mut().storage, &old_config).unwrap();

        let position = PositionV0_1_0 {
            owner: Addr::unchecked("owner"),
            in_balance: Uint128::new(1000000),
            shares: Uint128::new(1000000),
            index: Decimal256::percent(1),
            last_updated: Timestamp::from_nanos(1721759663200729304),
            purchased: Uint128::new(1000000),
            pending_purchase: Decimal256::percent(1),
            spent: Uint128::new(1000000),
            operator: Some(Addr::unchecked("operator")),
        };

        OLD_POSITIONS
            .save(
                deps.as_mut().storage,
                (1u64, Addr::unchecked("owner")),
                &position,
            )
            .unwrap();

        // Call the migrate_v0_1_0 function
        migrate_v0_1_0(&mut deps.storage).unwrap();

        let env = mock_env();
        let mock_info = mock_info("owner", &[]);

        // Now stream and config is migrated,
        let res = crate::contract::execute_migrate_position(
            deps.as_mut(),
            env.clone(),
            mock_info.clone(),
            1u64,
        )
        .unwrap();
        assert_eq!(res.attributes[0], ("action", "migrate_position"));
        assert_eq!(res.attributes[1], ("stream_id", "1"));
        assert_eq!(res.attributes[2], ("owner", "owner"));

        // Check deps.storage for the migrated position
        let new_position = POSITIONS
            .may_load(deps.as_ref().storage, (1u64, &Addr::unchecked("owner")))
            .unwrap()
            .unwrap();
        assert_eq!(new_position.owner, Addr::unchecked("owner"));

        // Ensure the old position has been removed
        let res = OLD_POSITIONS.load(&deps.storage, (1u64, Addr::unchecked("owner")));
        assert_eq!(res.is_err(), true);
        // Ensure the new position has been saved
        let res = POSITIONS.load(&deps.storage, (1u64, &Addr::unchecked("owner")));
        assert_eq!(res.is_ok(), true);

        // Ensure the old config has been removed
        let res = OLD_CONFIG.load(&deps.storage);
        assert_eq!(res.is_err(), true);

        // Ensure the new config has been saved
        let res = crate::state::CONFIG.load(&deps.storage);
        assert_eq!(res.is_ok(), true);

        // Ensure the old streams have been removed
        let res = OLD_STREAMS.load(&deps.storage, 1u64);
        assert_eq!(res.is_err(), true);

        // Ensure the new streams have been saved
        let res = STREAMS.load(&deps.storage, 1u64);
        assert_eq!(res.is_ok(), true);
    }
}
