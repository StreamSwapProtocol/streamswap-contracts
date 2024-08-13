#![cfg(test)]
use crate::helpers::suite::SuiteBuilder;
use crate::helpers::{mock_messages::get_controller_inst_msg, suite::Suite};
use cosmwasm_std::{coin, Decimal256};
use cw_multi_test::Executor;
use streamswap_controller::error::ContractError as ControllerError;
use streamswap_types::controller::{ExecuteMsg, Params, QueryMsg};

#[test]
fn params_update() {
    let Suite {
        mut app,
        test_accounts,
        stream_swap_code_id,
        stream_swap_controller_code_id,
        vesting_code_id,
    } = SuiteBuilder::default().build();

    let msg = get_controller_inst_msg(stream_swap_code_id, vesting_code_id, &test_accounts);
    let controller_address = app
        .instantiate_contract(
            stream_swap_controller_code_id,
            test_accounts.admin.clone(),
            &msg,
            &[],
            "Controller".to_string(),
            None,
        )
        .unwrap();

    // Non-admin cannot update params
    let msg = ExecuteMsg::UpdateParams {
        stream_creation_fee: Some(coin(100, "fee_denom")),
        exit_fee_percent: None,
        accepted_in_denoms: None,
        fee_collector: None,
        min_bootstrapping_duration: None,
        min_waiting_duration: None,
        min_stream_duration: None,
    };
    let res = app
        .execute_contract(
            test_accounts.subscriber_1.clone(),
            controller_address.clone(),
            &msg,
            &[],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<ControllerError>().unwrap();
    assert_eq!(*error, ControllerError::Unauthorized {});

    // Update stream creation fee
    let msg = ExecuteMsg::UpdateParams {
        stream_creation_fee: Some(coin(200, "fee_denom")),
        exit_fee_percent: None,
        accepted_in_denoms: None,
        fee_collector: None,
        min_bootstrapping_duration: None,
        min_waiting_duration: None,
        min_stream_duration: None,
    };
    let _ = app
        .execute_contract(
            test_accounts.admin.clone(),
            controller_address.clone(),
            &msg,
            &[],
        )
        .unwrap();

    // Query Params
    let res: Params = app
        .wrap()
        .query_wasm_smart(controller_address.clone(), &QueryMsg::Params {})
        .unwrap();

    assert_eq!(res.stream_creation_fee, coin(200, "fee_denom"));

    // Update wrong exit fee percent
    let msg = ExecuteMsg::UpdateParams {
        stream_creation_fee: None,
        exit_fee_percent: Some(Decimal256::percent(101)),
        accepted_in_denoms: None,
        fee_collector: None,
        min_bootstrapping_duration: None,
        min_waiting_duration: None,
        min_stream_duration: None,
    };
    let res = app
        .execute_contract(
            test_accounts.admin.clone(),
            controller_address.clone(),
            &msg,
            &[],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<ControllerError>().unwrap();
    assert_eq!(*error, ControllerError::InvalidExitFeePercent {});

    // Update exit fee percent
    let msg = ExecuteMsg::UpdateParams {
        stream_creation_fee: None,
        exit_fee_percent: Some(Decimal256::percent(50)),
        accepted_in_denoms: None,
        fee_collector: None,
        min_bootstrapping_duration: None,
        min_waiting_duration: None,
        min_stream_duration: None,
    };
    let _ = app
        .execute_contract(
            test_accounts.admin.clone(),
            controller_address.clone(),
            &msg,
            &[],
        )
        .unwrap();

    // Query Params
    let res: Params = app
        .wrap()
        .query_wasm_smart(controller_address.clone(), &QueryMsg::Params {})
        .unwrap();

    assert_eq!(res.exit_fee_percent, Decimal256::percent(50));

    // Update accepted in denoms
    let msg = ExecuteMsg::UpdateParams {
        stream_creation_fee: None,
        exit_fee_percent: None,
        accepted_in_denoms: Some(vec!["denom1".to_string(), "denom2".to_string()]),
        fee_collector: None,
        min_bootstrapping_duration: None,
        min_waiting_duration: None,
        min_stream_duration: None,
    };
    let _ = app
        .execute_contract(
            test_accounts.admin.clone(),
            controller_address.clone(),
            &msg,
            &[],
        )
        .unwrap();

    // Query Params
    let res: Params = app
        .wrap()
        .query_wasm_smart(controller_address.clone(), &QueryMsg::Params {})
        .unwrap();

    assert_eq!(
        res.accepted_in_denoms,
        vec!["denom1".to_string(), "denom2".to_string()]
    );

    // Update fee collector
    let msg = ExecuteMsg::UpdateParams {
        stream_creation_fee: None,
        exit_fee_percent: None,
        accepted_in_denoms: None,
        fee_collector: test_accounts.admin_2.to_string().into(),
        min_bootstrapping_duration: None,
        min_waiting_duration: None,
        min_stream_duration: None,
    };
    let _ = app
        .execute_contract(
            test_accounts.admin.clone(),
            controller_address.clone(),
            &msg,
            &[],
        )
        .unwrap();

    // Query Params
    let res: Params = app
        .wrap()
        .query_wasm_smart(controller_address.clone(), &QueryMsg::Params {})
        .unwrap();

    assert_eq!(res.fee_collector, test_accounts.admin_2);

    // Update min stream duration
    let msg = ExecuteMsg::UpdateParams {
        stream_creation_fee: None,
        exit_fee_percent: None,
        accepted_in_denoms: None,
        fee_collector: None,
        min_bootstrapping_duration: None,
        min_waiting_duration: None,
        min_stream_duration: Some(200),
    };
    let _ = app
        .execute_contract(
            test_accounts.admin.clone(),
            controller_address.clone(),
            &msg,
            &[],
        )
        .unwrap();

    // Query Params
    let res: Params = app
        .wrap()
        .query_wasm_smart(controller_address.clone(), &QueryMsg::Params {})
        .unwrap();

    assert_eq!(res.min_stream_duration, 200);

    // Update min bootstrapping duration
    let msg = ExecuteMsg::UpdateParams {
        stream_creation_fee: None,
        exit_fee_percent: None,
        accepted_in_denoms: None,
        fee_collector: None,
        min_bootstrapping_duration: Some(200),
        min_waiting_duration: None,
        min_stream_duration: None,
    };
    let _ = app
        .execute_contract(
            test_accounts.admin.clone(),
            controller_address.clone(),
            &msg,
            &[],
        )
        .unwrap();

    // Query Params
    let res: Params = app
        .wrap()
        .query_wasm_smart(controller_address.clone(), &QueryMsg::Params {})
        .unwrap();

    assert_eq!(res.min_bootstrapping_duration, 200);
}
