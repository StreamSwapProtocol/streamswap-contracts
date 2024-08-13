#![cfg(test)]
use crate::helpers::suite::SuiteBuilder;
use crate::helpers::{mock_messages::get_controller_inst_msg, suite::Suite};
use cosmwasm_std::{coin, Decimal256};
use cw_multi_test::Executor;
use streamswap_types::controller::Params;
use streamswap_types::controller::QueryMsg;

#[test]
fn controller_proper_instantiate() {
    //let mut setup_res = setup();
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

    // Query Params
    let res: Params = app
        .wrap()
        .query_wasm_smart(controller_address, &QueryMsg::Params {})
        .unwrap();
    assert_eq!(res.stream_creation_fee, coin(100, "fee_denom"));
    assert_eq!(res.exit_fee_percent, Decimal256::percent(1));
    assert_eq!(res.stream_contract_code_id, stream_swap_code_id);
    assert_eq!(res.accepted_in_denoms, vec!["in_denom".to_string()]);
    assert_eq!(res.fee_collector, test_accounts.admin.to_string());
    assert_eq!(res.min_waiting_duration, 49);
    assert_eq!(res.min_bootstrapping_duration, 49);
    assert_eq!(res.min_stream_duration, 99);
}
