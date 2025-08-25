use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::{BankMsg, Coin, CosmosMsg, Timestamp, Uint256};
use cw_streamswap::contract::{execute, execute_finalize_stream};
use cw_streamswap::ContractError;

mod helpers;

#[test]
fn operator_unauthorized_subscribe_on_behalf() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());
    // Define actors
    let treasury = helpers::mock_info("creator", &[]);
    let owner = helpers::mock_info("creator1", &[]);
    let random = helpers::mock_info("random", &[]);
    // Create stream
    let start = Timestamp::from_seconds(1_000_000);
    let end = Timestamp::from_seconds(5_000_000);
    let out_supply = Uint256::from(1_000_000u128);
    let b = helpers::CreateStreamBuilder::default()
        .start_time(start)
        .end_time(end)
        .out_supply(out_supply)
        .out_denom("out_denom");
    let env = helpers::env_at(0);
    let funds = vec![
        Coin {
            denom: "out_denom".to_string(),
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
    let env = helpers::env_at(start.seconds() + 100);
    let mut random_funded = random.clone();
    random_funded.funds = vec![Coin::new(1_000_000u128, "in")];
    let msg = cw_streamswap::msg::ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: Some(owner.sender.to_string()),
        operator: None,
        tos_version: "v1".to_string(),
    };
    let err = execute(deps.as_mut(), env, random_funded, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn operator_only_owner_can_update() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    let treasury = helpers::mock_info("creator", &[]);
    let owner = helpers::mock_info("creator1", &[]);
    let other = helpers::mock_info("creator2", &[]);

    let start = Timestamp::from_seconds(1_000_000);
    let end = Timestamp::from_seconds(5_000_000);
    let out_supply = Uint256::from(1_000_000u128);
    let b = helpers::CreateStreamBuilder::default()
        .start_time(start)
        .end_time(end)
        .out_supply(out_supply)
        .out_denom("out_denom");
    let env0 = helpers::env_at(0);
    let funds = vec![
        Coin {
            denom: "out_denom".to_string(),
            amount: out_supply,
        },
        Coin {
            denom: helpers::DEFAULT_STREAM_CREATION_DENOM.to_string(),
            amount: Uint256::from(100u128),
        },
    ];
    let mut treasury_funded = treasury.clone();
    treasury_funded.funds = funds;
    execute(deps.as_mut(), env0, treasury_funded, b.build()).unwrap();
    // Owner subscribes first
    let env = helpers::env_at(start.seconds() + 100);
    let mut owner_funded = owner.clone();
    owner_funded.funds = vec![Coin::new(1_000_000u128, "in")];
    execute(
        deps.as_mut(),
        env,
        owner_funded,
        cw_streamswap::msg::ExecuteMsg::Subscribe {
            stream_id: 1,
            operator_target: None,
            operator: None,
            tos_version: "v1".to_string(),
        },
    )
    .unwrap();
    // Other cannot update owner's position
    let env = helpers::env_at(start.seconds() + 100);
    let msg = cw_streamswap::msg::ExecuteMsg::UpdatePosition {
        stream_id: 1,
        operator_target: Some(owner.sender.to_string()),
    };
    let err = execute(deps.as_mut(), env, other, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn operator_set_and_use_operator() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());
    let treasury = helpers::mock_info("creator", &[]);
    let owner = helpers::mock_info("creator1", &[]);
    let operator = helpers::mock_info("operator1", &[]);
    let random = helpers::mock_info("random", &[]);
    let start = Timestamp::from_seconds(1_000_000);
    let end = Timestamp::from_seconds(5_000_000);
    let out_supply = Uint256::from(1_000_000u128);
    let b = helpers::CreateStreamBuilder::default()
        .start_time(start)
        .end_time(end)
        .out_supply(out_supply)
        .out_denom("out_denom");
    let env0 = helpers::env_at(0);
    let funds = vec![
        Coin {
            denom: "out_denom".to_string(),
            amount: out_supply,
        },
        Coin {
            denom: helpers::DEFAULT_STREAM_CREATION_DENOM.to_string(),
            amount: Uint256::from(100u128),
        },
    ];
    let mut treasury_funded = treasury.clone();
    treasury_funded.funds = funds;
    execute(deps.as_mut(), env0, treasury_funded, b.build()).unwrap();
    // Owner subscribes
    let env = helpers::env_at(start.seconds() + 100);
    let mut owner_funded = owner.clone();
    owner_funded.funds = vec![Coin::new(1_000_000u128, "in")];
    execute(
        deps.as_mut(),
        env,
        owner_funded,
        cw_streamswap::msg::ExecuteMsg::Subscribe {
            stream_id: 1,
            operator_target: None,
            operator: None,
            tos_version: "v1".to_string(),
        },
    )
    .unwrap();
    // Owner sets operator
    let env = helpers::env_at(start.seconds() + 100);
    execute(
        deps.as_mut(),
        env,
        owner.clone(),
        cw_streamswap::msg::ExecuteMsg::UpdateOperator {
            stream_id: 1,
            new_operator: Some(operator.sender.to_string()),
        },
    )
    .unwrap();
    // Operator subscribes on behalf of owner
    let env = helpers::env_at(start.seconds() + 100);
    let mut operator_funded = operator.clone();
    operator_funded.funds = vec![Coin::new(1_000_000u128, "in")];
    let res = execute(
        deps.as_mut(),
        env,
        operator_funded,
        cw_streamswap::msg::ExecuteMsg::Subscribe {
            stream_id: 1,
            operator_target: Some(owner.sender.to_string()),
            operator: None,
            tos_version: "v1".to_string(),
        },
    )
    .unwrap();
    assert_eq!(res.attributes[0].value, "subscribe");
    // Operator update position
    let env = helpers::env_at(start.seconds() + 100);
    let res = execute(
        deps.as_mut(),
        env,
        operator.clone(),
        cw_streamswap::msg::ExecuteMsg::UpdatePosition {
            stream_id: 1,
            operator_target: Some(owner.sender.to_string()),
        },
    )
    .unwrap();
    assert_eq!(res.attributes[0].value, "update_position");
    // Finalize and check exit auth
    let env = helpers::env_at(end.seconds() + 1);
    execute_finalize_stream(deps.as_mut(), env.clone(), treasury.clone(), 1, None).unwrap();
    // Random cannot exit
    let err = cw_streamswap::contract::execute_exit_stream(
        deps.as_mut(),
        env.clone(),
        random.clone(),
        1,
        Some(owner.sender.to_string()),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
    // Operator can exit to owner
    let res = cw_streamswap::contract::execute_exit_stream(
        deps.as_mut(),
        env,
        operator,
        1,
        Some(owner.sender.to_string()),
    )
    .unwrap();
    match res.messages.first().unwrap().msg.clone() {
        CosmosMsg::Bank(BankMsg::Send { to_address, .. }) => {
            assert_eq!(to_address, owner.sender.to_string())
        }
        _ => panic!("unexpected message"),
    }
}
