use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::{Coin, Decimal256, Timestamp, Uint256};
use cw_streamswap::{self, contract::execute};
use std::str::FromStr;

mod helpers;

#[test]
fn update_stream_no_subscriptions() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Create a stream that starts in the future
    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(1_000_000))
        .end_time(Timestamp::from_seconds(5_000_000));
    let env = helpers::env_at(1);
    let funds = vec![
        Coin {
            denom: "out_denom".to_string(),
            amount: b.out_supply,
        },
        Coin {
            denom: helpers::DEFAULT_STREAM_CREATION_DENOM.to_string(),
            amount: Uint256::from(100u128),
        },
    ];
    let info = helpers::mock_info("creator", &funds);
    execute(deps.as_mut(), env, info, b.build()).unwrap();

    // Update stream without subscriptions - no distribution should occur
    let env = helpers::env_at(1_000_000 + 100); // After start time
    let res = cw_streamswap::contract::execute_update_stream(deps.as_mut(), env, 1).unwrap();

    // Verify no distribution occurred - match original test exactly
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "update_stream");
    assert_eq!(res.attributes[1].key, "stream_id");
    assert_eq!(res.attributes[1].value, "1");
    assert_eq!(res.attributes[2].key, "new_distribution_amount");
    assert_eq!(res.attributes[2].value, "0");
    assert_eq!(res.attributes[3].key, "dist_index");
    assert_eq!(res.attributes[3].value, "0");
    assert_eq!(res.attributes[4].key, "last_updated");
    assert_eq!(res.attributes[5].key, "start_time");
    assert_eq!(res.attributes[6].key, "end_time");
    assert_eq!(res.attributes[7].key, "in_denom");
    assert_eq!(res.attributes[8].key, "out_denom");
    assert_eq!(res.attributes[9].key, "in_supply");
    assert_eq!(res.attributes[10].key, "out_supply");
    assert_eq!(res.attributes[11].key, "out_remaining");
    assert_eq!(res.attributes[12].key, "spent_in");
    assert_eq!(res.attributes[13].key, "shares");
    assert_eq!(res.attributes[14].key, "current_streamed_price");
    assert_eq!(res.attributes[15].key, "status");
    assert_eq!(res.attributes[15].value, "Waiting");
    assert_eq!(res.attributes[16].key, "tos_version");
    assert_eq!(res.attributes[16].value, "v1");
}

#[test]
fn update_stream_first_subscription() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Create a stream
    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(1_000_000))
        .end_time(Timestamp::from_seconds(5_000_000));
    let env = helpers::env_at(1);
    let funds = vec![
        Coin {
            denom: "out_denom".to_string(),
            amount: b.out_supply,
        },
        Coin {
            denom: helpers::DEFAULT_STREAM_CREATION_DENOM.to_string(),
            amount: Uint256::from(100u128),
        },
    ];
    let info = helpers::mock_info("creator", &funds);
    execute(deps.as_mut(), env, info, b.build()).unwrap();

    // First subscription - dist_index should remain 0 (no prior distribution)
    let env = helpers::env_at(1_000_000 + 100);
    let info = helpers::mock_info("creator1", &[Coin::new(1_000_000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();

    // Query stream after first subscription - dist_index should still be 0 (no prior distribution)
    let env = helpers::env_at(1_000_000 + 200);
    let stream = cw_streamswap::contract::query_stream(deps.as_ref(), env, 1).unwrap();
    assert_eq!(stream.dist_index, Decimal256::zero());
}

#[test]
fn update_stream_with_subscribers() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Create a stream
    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(1_000_000))
        .end_time(Timestamp::from_seconds(5_000_000));
    let env = helpers::env_at(1);
    let funds = vec![
        Coin {
            denom: "out_denom".to_string(),
            amount: b.out_supply,
        },
        Coin {
            denom: helpers::DEFAULT_STREAM_CREATION_DENOM.to_string(),
            amount: Uint256::from(100u128),
        },
    ];
    let info = helpers::mock_info("creator", &funds);
    execute(deps.as_mut(), env, info, b.build()).unwrap();

    // First subscription
    let env = helpers::env_at(1_000_000 + 100);
    let info = helpers::mock_info("creator1", &[Coin::new(1_000_000u128, "in")]);
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();

    // Update stream again with existing subscribers - dist_index should increase
    let env = helpers::env_at(1_000_000 + 300);
    let _res = cw_streamswap::contract::execute_update_stream(deps.as_mut(), env, 1).unwrap();

    // Query stream - dist_index should now be non-zero
    let env = helpers::env_at(1_000_000 + 300);
    let stream = cw_streamswap::contract::query_stream(deps.as_ref(), env, 1).unwrap();
    assert_eq!(stream.dist_index, Decimal256::from_str("0.00005").unwrap());
}
