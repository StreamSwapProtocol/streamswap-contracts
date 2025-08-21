use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::Timestamp;
use cosmwasm_std::{Coin, Decimal256, Uint256};
use cw_streamswap::{
    contract::{execute_create_stream, instantiate},
    ContractError,
};
mod helpers;

#[test]
fn instantiate_invalid_exit_fee_percent() {
    let mut deps = mock_dependencies();
    let msg = helpers::InstantiateBuilder::default()
        .exit_fee_percent(Decimal256::percent(101))
        .build();
    let err = instantiate(
        deps.as_mut(),
        helpers::env_now(),
        helpers::mock_info("creator", &[]),
        msg,
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidExitFeePercent {});
}

#[test]
fn instantiate_invalid_stream_creation_fee() {
    let mut deps = mock_dependencies();
    let msg = helpers::InstantiateBuilder::default()
        .stream_creation_fee(Uint256::zero())
        .build();
    let err = instantiate(
        deps.as_mut(),
        helpers::env_now(),
        helpers::mock_info("creator", &[]),
        msg,
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidStreamCreationFee {});
}

#[test]
fn create_stream_in_denom_not_accepted() {
    let mut deps = mock_dependencies();
    // set up config
    helpers::instantiate_defaults(deps.as_mut());

    // prepare stream args with wrong in_denom
    let b = helpers::CreateStreamBuilder::default().in_denom("random");
    let env = helpers::env_at(0);
    let info = helpers::mock_info("creator", &[]);
    let res = execute_create_stream(
        deps.as_mut(),
        env,
        info,
        b.treasury,
        b.name,
        b.url,
        b.in_denom,
        b.out_denom,
        b.out_supply,
        b.start_time,
        b.end_time,
        b.threshold,
        b.tos_version,
    );
    assert_eq!(res.unwrap_err(), ContractError::InDenomIsNotAccepted {});
}

#[test]
fn create_stream_end_before_start() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(10_000))
        .end_time(Timestamp::from_seconds(5_000));
    let env = helpers::env_at(0);
    let info = helpers::mock_info("creator", &[]);
    let res = execute_create_stream(
        deps.as_mut(),
        env,
        info,
        b.treasury,
        b.name,
        b.url,
        b.in_denom,
        b.out_denom,
        b.out_supply,
        b.start_time,
        b.end_time,
        b.threshold,
        b.tos_version,
    );
    assert_eq!(res.unwrap_err(), ContractError::StreamInvalidEndTime {});
}

#[test]
fn create_stream_start_in_past() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(5_000))
        .end_time(Timestamp::from_seconds(10_000));
    let env = helpers::env_at(20_000);
    let info = helpers::mock_info("creator", &[]);
    let res = execute_create_stream(
        deps.as_mut(),
        env,
        info,
        b.treasury,
        b.name,
        b.url,
        b.in_denom,
        b.out_denom,
        b.out_supply,
        b.start_time,
        b.end_time,
        b.threshold,
        b.tos_version,
    );
    assert_eq!(res.unwrap_err(), ContractError::StreamInvalidStartTime {});
}

#[test]
fn create_stream_duration_too_short() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // default min_stream_seconds is 1000; set duration < 1000
    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(5_000))
        .end_time(Timestamp::from_seconds(5_500));
    let env = helpers::env_at(0);
    let info = helpers::mock_info("creator", &[]);
    let res = execute_create_stream(
        deps.as_mut(),
        env,
        info,
        b.treasury,
        b.name,
        b.url,
        b.in_denom,
        b.out_denom,
        b.out_supply,
        b.start_time,
        b.end_time,
        b.threshold,
        b.tos_version,
    );
    assert_eq!(res.unwrap_err(), ContractError::StreamDurationTooShort {});
}

#[test]
fn create_stream_starts_too_soon() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // default min_seconds_until_start_time is 1000; set start - now < 1000
    let b = helpers::CreateStreamBuilder::default()
        .start_time(Timestamp::from_seconds(5_500))
        .end_time(Timestamp::from_seconds(7_000));
    let env = helpers::env_at(5_000);
    let info = helpers::mock_info("creator", &[]);
    let res = execute_create_stream(
        deps.as_mut(),
        env,
        info,
        b.treasury,
        b.name,
        b.url,
        b.in_denom,
        b.out_denom,
        b.out_supply,
        b.start_time,
        b.end_time,
        b.threshold,
        b.tos_version,
    );
    assert_eq!(res.unwrap_err(), ContractError::StreamStartsTooSoon {});
}

#[test]
fn create_stream_same_denom_each_side() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    let b = helpers::CreateStreamBuilder::default()
        .in_denom(helpers::DEFAULT_ACCEPTED_IN_DENOM)
        .out_denom(helpers::DEFAULT_ACCEPTED_IN_DENOM);
    let env = helpers::env_at(0);
    let info = helpers::mock_info("creator", &[]);
    let res = execute_create_stream(
        deps.as_mut(),
        env,
        info,
        b.treasury,
        b.name,
        b.url,
        b.in_denom,
        b.out_denom,
        b.out_supply,
        b.start_time,
        b.end_time,
        b.threshold,
        b.tos_version,
    );
    assert_eq!(res.unwrap_err(), ContractError::SameDenomOnEachSide {});
}

#[test]
fn create_stream_zero_out_supply() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    let b = helpers::CreateStreamBuilder::default().out_supply(Uint256::zero());
    let env = helpers::env_at(0);
    let info = helpers::mock_info("creator", &[]);
    let res = execute_create_stream(
        deps.as_mut(),
        env,
        info,
        b.treasury,
        b.name,
        b.url,
        b.in_denom,
        b.out_denom,
        b.out_supply,
        b.start_time,
        b.end_time,
        b.threshold,
        b.tos_version,
    );
    assert_eq!(res.unwrap_err(), ContractError::ZeroOutSupply {});
}

#[test]
fn create_stream_invalid_tos_version() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    let b = helpers::CreateStreamBuilder::default().tos_version("v2");
    let env = helpers::env_at(0);
    let info = helpers::mock_info("creator", &[]);
    let res = execute_create_stream(
        deps.as_mut(),
        env,
        info,
        b.treasury,
        b.name,
        b.url,
        b.in_denom,
        b.out_denom,
        b.out_supply,
        b.start_time,
        b.end_time,
        b.threshold,
        b.tos_version,
    );
    assert_eq!(res.unwrap_err(), ContractError::InvalidToSVersion {});
}

#[test]
fn create_stream_else_branch_missing_out_funds() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // out_denom != fee, but provide no funds
    let b = helpers::CreateStreamBuilder::default().out_denom("token");
    let env = helpers::env_at(0);
    let info = helpers::mock_info("creator", &[]);
    let res = execute_create_stream(
        deps.as_mut(),
        env,
        info,
        b.treasury,
        b.name,
        b.url,
        b.in_denom,
        b.out_denom,
        b.out_supply,
        b.start_time,
        b.end_time,
        b.threshold,
        b.tos_version,
    );
    assert_eq!(res.unwrap_err(), ContractError::NoFundsSent {});
}

#[test]
fn create_stream_else_branch_wrong_out_amount() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    let b = helpers::CreateStreamBuilder::default().out_denom("token");
    let env = helpers::env_at(0);
    let funds = vec![
        Coin {
            denom: "token".to_string(),
            amount: b.out_supply + Uint256::from(1u128),
        },
        Coin {
            denom: helpers::DEFAULT_STREAM_CREATION_DENOM.to_string(),
            amount: Uint256::from(100u128),
        },
    ];
    let info = helpers::mock_info("creator", &funds);
    let res = execute_create_stream(
        deps.as_mut(),
        env,
        info,
        b.treasury,
        b.name,
        b.url,
        b.in_denom,
        b.out_denom,
        b.out_supply,
        b.start_time,
        b.end_time,
        b.threshold,
        b.tos_version,
    );
    assert_eq!(
        res.unwrap_err(),
        ContractError::StreamOutSupplyFundsRequired {}
    );
}

#[test]
fn create_stream_else_branch_missing_creation_fee() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    let b = helpers::CreateStreamBuilder::default().out_denom("token");
    let env = helpers::env_at(0);
    let funds = vec![Coin {
        denom: "token".to_string(),
        amount: b.out_supply,
    }];
    let info = helpers::mock_info("creator", &funds);
    let res = execute_create_stream(
        deps.as_mut(),
        env,
        info,
        b.treasury,
        b.name,
        b.url,
        b.in_denom,
        b.out_denom,
        b.out_supply,
        b.start_time,
        b.end_time,
        b.threshold,
        b.tos_version,
    );
    assert_eq!(res.unwrap_err(), ContractError::NoFundsSent {});
}

#[test]
fn create_stream_else_branch_wrong_creation_fee_amount() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    let b = helpers::CreateStreamBuilder::default().out_denom("token");
    let env = helpers::env_at(0);
    let funds = vec![
        Coin {
            denom: "token".to_string(),
            amount: b.out_supply,
        },
        Coin {
            denom: helpers::DEFAULT_STREAM_CREATION_DENOM.to_string(),
            amount: Uint256::from(99u128),
        },
    ];
    let info = helpers::mock_info("creator", &funds);
    let res = execute_create_stream(
        deps.as_mut(),
        env,
        info,
        b.treasury,
        b.name,
        b.url,
        b.in_denom,
        b.out_denom,
        b.out_supply,
        b.start_time,
        b.end_time,
        b.threshold,
        b.tos_version,
    );
    assert_eq!(
        res.unwrap_err(),
        ContractError::StreamCreationFeeRequired {}
    );
}

#[test]
fn create_stream_else_branch_invalid_extra_funds() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    let b = helpers::CreateStreamBuilder::default().out_denom("token");
    let env = helpers::env_at(0);
    let funds = vec![
        Coin {
            denom: "token".to_string(),
            amount: b.out_supply,
        },
        Coin {
            denom: helpers::DEFAULT_STREAM_CREATION_DENOM.to_string(),
            amount: Uint256::from(100u128),
        },
        Coin {
            denom: "random".to_string(),
            amount: Uint256::from(1u128),
        },
    ];
    let info = helpers::mock_info("creator", &funds);
    let res = execute_create_stream(
        deps.as_mut(),
        env,
        info,
        b.treasury,
        b.name,
        b.url,
        b.in_denom,
        b.out_denom,
        b.out_supply,
        b.start_time,
        b.end_time,
        b.threshold,
        b.tos_version,
    );
    assert_eq!(res.unwrap_err(), ContractError::InvalidFunds {});
}

#[test]
fn create_stream_fee_branch_missing_funds() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // out_denom equals fee denom; expect NoFundsSent without funds
    let b =
        helpers::CreateStreamBuilder::default().out_denom(helpers::DEFAULT_STREAM_CREATION_DENOM);
    let env = helpers::env_at(0);
    let info = helpers::mock_info("creator", &[]);
    let res = execute_create_stream(
        deps.as_mut(),
        env,
        info,
        b.treasury,
        b.name,
        b.url,
        b.in_denom,
        b.out_denom,
        b.out_supply,
        b.start_time,
        b.end_time,
        b.threshold,
        b.tos_version,
    );
    assert_eq!(res.unwrap_err(), ContractError::NoFundsSent {});
}

#[test]
fn create_stream_fee_branch_wrong_total_amount() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    let b =
        helpers::CreateStreamBuilder::default().out_denom(helpers::DEFAULT_STREAM_CREATION_DENOM);
    let env = helpers::env_at(0);
    let funds = vec![Coin {
        denom: helpers::DEFAULT_STREAM_CREATION_DENOM.to_string(),
        amount: b.out_supply + Uint256::from(99u128),
    }];
    let info = helpers::mock_info("creator", &funds);
    let res = execute_create_stream(
        deps.as_mut(),
        env,
        info,
        b.treasury,
        b.name,
        b.url,
        b.in_denom,
        b.out_denom,
        b.out_supply,
        b.start_time,
        b.end_time,
        b.threshold,
        b.tos_version,
    );
    assert_eq!(
        res.unwrap_err(),
        ContractError::StreamOutSupplyFundsRequired {}
    );
}

#[test]
fn create_stream_fee_branch_invalid_extra_funds() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    let b =
        helpers::CreateStreamBuilder::default().out_denom(helpers::DEFAULT_STREAM_CREATION_DENOM);
    let env = helpers::env_at(0);
    let funds = vec![
        Coin {
            denom: helpers::DEFAULT_STREAM_CREATION_DENOM.to_string(),
            amount: b.out_supply + Uint256::from(100u128),
        },
        Coin {
            denom: "random".to_string(),
            amount: Uint256::from(1u128),
        },
    ];
    let info = helpers::mock_info("creator", &funds);
    let res = execute_create_stream(
        deps.as_mut(),
        env,
        info,
        b.treasury,
        b.name,
        b.url,
        b.in_denom,
        b.out_denom,
        b.out_supply,
        b.start_time,
        b.end_time,
        b.threshold,
        b.tos_version,
    );
    assert_eq!(res.unwrap_err(), ContractError::InvalidFunds {});
}
