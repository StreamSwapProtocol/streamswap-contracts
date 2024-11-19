#![cfg(test)]
use crate::helpers::mock_messages::CreateStreamMsgBuilder;
use crate::helpers::suite::SuiteBuilder;
use crate::helpers::{mock_messages::get_controller_inst_msg, suite::Suite};
use cosmwasm_std::coin;
use cw_multi_test::Executor;
use streamswap_controller::error::ContractError as ControllerError;
use streamswap_types::controller::QueryMsg;

#[test]
fn controller_freeze() {
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
    // When controller is created, it is not frozen, Stream creation is allowed
    let create_stream_msg = CreateStreamMsgBuilder::new(
        "stream",
        test_accounts.creator_1.as_ref(),
        coin(100, "out_denom"),
        "in_denom",
        app.block_info().time.plus_seconds(50),
        app.block_info().time.plus_seconds(100),
        app.block_info().time.plus_seconds(200),
    )
    .build();

    let _create_stream_res = app
        .execute_contract(
            test_accounts.creator_1.clone(),
            controller_address.clone(),
            &create_stream_msg,
            &[coin(100, "fee_denom"), coin(100, "out_denom")],
        )
        .unwrap();

    // Non-admin cannot freeze controller
    let msg = streamswap_types::controller::ExecuteMsg::Freeze {};
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

    // Admin can freeze controller
    let msg = streamswap_types::controller::ExecuteMsg::Freeze {};
    app.execute_contract(
        test_accounts.admin.clone(),
        controller_address.clone(),
        &msg,
        &[],
    )
    .unwrap();

    // Query Params
    let res: bool = app
        .wrap()
        .query_wasm_smart(controller_address.clone(), &QueryMsg::Freezestate {})
        .unwrap();
    assert!(res);

    // When controller is frozen, Stream creation is not allowed
    let create_stream_msg = CreateStreamMsgBuilder::new(
        "stream",
        test_accounts.creator_1.as_ref(),
        coin(100, "out_denom"),
        "in_denom",
        app.block_info().time.plus_seconds(50),
        app.block_info().time.plus_seconds(100),
        app.block_info().time.plus_seconds(200),
    )
    .build();
    let res = app
        .execute_contract(
            test_accounts.creator_1.clone(),
            controller_address.clone(),
            &create_stream_msg,
            &[coin(100, "fee_denom"), coin(100, "out_denom")],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<ControllerError>().unwrap();
    assert_eq!(*error, ControllerError::ContractIsFrozen {});
}
