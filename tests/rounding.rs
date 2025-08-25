use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::{Coin, Decimal256, Timestamp, Uint256};
use cw_streamswap::contract::execute;
use cw_streamswap::msg::ExecuteMsg;
use std::str::FromStr;

mod helpers;

#[test]
fn test_rounding_leftover() {
    let mut deps = mock_dependencies();

    // Setup: Instantiate the contract
    helpers::instantiate_defaults(deps.as_mut());

    // Define actors
    let treasury = helpers::mock_info("creator", &[]);
    let subscriber1 = helpers::mock_info("creator1", &[]);
    let subscriber2 = helpers::mock_info("creator2", &[]);

    // Setup: Create a stream with specific parameters
    let start = Timestamp::from_seconds(1_000_000);
    let end = Timestamp::from_seconds(5_000_000);
    let out_supply = Uint256::from(1_000_000_000_000u128);
    let out_denom = "out_denom";

    let b = helpers::CreateStreamBuilder::default()
        .start_time(start)
        .end_time(end)
        .out_supply(out_supply)
        .out_denom(out_denom);

    let env = helpers::env_at(1);
    let funds = vec![
        Coin {
            denom: out_denom.to_string(),
            amount: out_supply,
        },
        Coin {
            denom: helpers::DEFAULT_STREAM_CREATION_DENOM.to_string(),
            amount: Uint256::from(100u128),
        },
    ];
    let mut treasury_funded = treasury.clone();
    treasury_funded.funds = funds;
    execute(deps.as_mut(), env, treasury_funded, b.build()).unwrap();

    // Setup: First subscription
    let env = helpers::env_at(1_000_000 + 100);
    let mut s1 = subscriber1.clone();
    s1.funds = vec![Coin::new(1_000_000_000u128, "in")];
    let msg = ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    execute(deps.as_mut(), env, s1, msg).unwrap();

    // Setup: Second subscription
    let env = helpers::env_at(1_000_000 + 100_000);
    let mut s2 = subscriber2.clone();
    s2.funds = vec![Coin::new(3_000_000_000u128, "in")];
    let msg = ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    execute(deps.as_mut(), env, s2, msg).unwrap();

    // Test 1: Update position creator1 during stream
    let env = helpers::env_at(1_000_000 + 3_000_000);
    let info = subscriber1.clone();
    let msg = ExecuteMsg::UpdatePosition {
        stream_id: 1,
        operator_target: None,
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Verify position creator1
    let position = cw_streamswap::contract::query_position(
        deps.as_ref(),
        env.clone(),
        1,
        subscriber1.sender.to_string(),
    )
    .unwrap();
    assert_eq!(
        position.index,
        Decimal256::from_str("202.813614449380587585").unwrap()
    );
    assert_eq!(position.purchased, Uint256::from(202_813_614_449u128));
    assert_eq!(position.spent, Uint256::from(749_993_750u128));
    assert_eq!(position.in_balance, Uint256::from(250_006_250u128));

    let stream = cw_streamswap::contract::query_stream(deps.as_ref(), env, 1).unwrap();
    assert_eq!(
        stream.dist_index,
        Decimal256::from_str("202.813614449380587585").unwrap()
    );

    // Test 2: Update position creator2 during stream
    let env = helpers::env_at(1_000_000 + 3_575_000);
    let info = subscriber2.clone();
    let msg = ExecuteMsg::UpdatePosition {
        stream_id: 1,
        operator_target: None,
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Verify position creator2
    let position = cw_streamswap::contract::query_position(
        deps.as_ref(),
        env.clone(),
        1,
        subscriber2.sender.to_string(),
    )
    .unwrap();
    assert_eq!(
        position.index,
        Decimal256::from_str("238.074595237060799266").unwrap()
    );
    assert_eq!(position.purchased, Uint256::from(655672748445u128));
    assert_eq!(position.spent, Uint256::from(2673076923u128));
    assert_eq!(position.in_balance, Uint256::from(326923077u128));

    let stream = cw_streamswap::contract::query_stream(deps.as_ref(), env, 1).unwrap();
    assert_eq!(
        stream.dist_index,
        Decimal256::from_str("238.074595237060799266").unwrap()
    );

    // Test 3: Update position creator1 after stream ends
    let env = helpers::env_at(5_000_000 + 1);
    let info = subscriber1.clone();
    let msg = ExecuteMsg::UpdatePosition {
        stream_id: 1,
        operator_target: None,
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let stream = cw_streamswap::contract::query_stream(deps.as_ref(), env.clone(), 1).unwrap();
    assert_eq!(
        stream.dist_index,
        Decimal256::from_str("264.137059297637397644").unwrap()
    );
    assert_eq!(stream.in_supply, Uint256::zero());

    let position1 = cw_streamswap::contract::query_position(
        deps.as_ref(),
        env,
        1,
        subscriber1.sender.to_string(),
    )
    .unwrap();
    assert_eq!(
        position1.index,
        Decimal256::from_str("264.137059297637397644").unwrap()
    );
    assert_eq!(position1.spent, Uint256::from(1_000_000_000u128));
    assert_eq!(position1.in_balance, Uint256::zero());

    // Test 4: Update position creator2 after stream ends
    let env = helpers::env_at(5_000_000 + 1);
    let info = subscriber2.clone();
    let msg = ExecuteMsg::UpdatePosition {
        stream_id: 1,
        operator_target: None,
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let stream = cw_streamswap::contract::query_stream(deps.as_ref(), env.clone(), 1).unwrap();
    assert_eq!(
        stream.dist_index,
        Decimal256::from_str("264.137059297637397644").unwrap()
    );
    assert_eq!(stream.in_supply, Uint256::zero());

    let position2 = cw_streamswap::contract::query_position(
        deps.as_ref(),
        env,
        1,
        subscriber2.sender.to_string(),
    )
    .unwrap();
    assert_eq!(
        position2.index,
        Decimal256::from_str("264.137059297637397644").unwrap()
    );
    assert_eq!(position2.spent, Uint256::from(3_000_000_000u128));
    assert_eq!(position2.in_balance, Uint256::zero());

    // Test 5: Verify rounding behavior
    assert_eq!(stream.out_remaining, Uint256::zero());
    assert_eq!(
        position1
            .purchased
            .checked_add(position2.purchased)
            .unwrap(),
        // 1 difference due to rounding
        stream.out_supply.checked_sub(Uint256::from(1u128)).unwrap()
    );
}
