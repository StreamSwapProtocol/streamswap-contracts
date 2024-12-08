#![cfg(test)]
use crate::helpers::mock_messages::{get_controller_inst_msg, CreateStreamMsgBuilder};
use crate::helpers::suite::{Suite, SuiteBuilder};
use crate::helpers::utils::get_wasm_attribute_with_key;
use cosmwasm_std::coin;
use cosmwasm_std::Binary;
use cw_multi_test::Executor;
use streamswap_types::controller::{QueryMsg, StreamsResponse};

#[test]
fn test_list_streams() {
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

    let create_stream_msg = CreateStreamMsgBuilder::new(
        "stream",
        test_accounts.creator_1.as_ref(),
        coin(100, "out_denom"),
        "in_denom",
        app.block_info().time.plus_seconds(50),
        app.block_info().time.plus_seconds(100),
        app.block_info().time.plus_seconds(200),
    )
    .salt(
        Binary::from_base64("dGlnaHRseXB1YmxpY2h1cnJ5Y2FyZWZ1bHJ1bGVyYm93d2FpdHZhcG9ydHJ1dGhicmk")
            .unwrap(),
    )
    .build();

    let res = app
        .execute_contract(
            test_accounts.creator_1.clone(),
            controller_address.clone(),
            &create_stream_msg,
            &[coin(100, "fee_denom"), coin(100, "out_denom")],
        )
        .unwrap();
    let stream_addr1 = get_wasm_attribute_with_key(res, "stream_contract_addr".to_string());

    let create_stream_msg = CreateStreamMsgBuilder::new(
        "stream2",
        test_accounts.creator_2.as_ref(),
        coin(200, "out_denom"),
        "in_denom",
        app.block_info().time.plus_seconds(50),
        app.block_info().time.plus_seconds(100),
        app.block_info().time.plus_seconds(200),
    )
    .salt(
        Binary::from_base64("bmVlZHNpbnRlcmVzdGtub3dudGhlbWRyYXdlc3BlY2lhbGx5d29ubm90aWNldmFsdWU")
            .unwrap(),
    )
    .build();

    let res = app
        .execute_contract(
            test_accounts.creator_2.clone(),
            controller_address.clone(),
            &create_stream_msg,
            &[coin(100, "fee_denom"), coin(200, "out_denom")],
        )
        .unwrap();
    let stream_addr2 = get_wasm_attribute_with_key(res, "stream_contract_addr".to_string());

    let res: StreamsResponse = app
        .wrap()
        .query_wasm_smart(
            controller_address.clone(),
            &QueryMsg::ListStreams {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();

    assert_eq!(res.streams.len(), 2);
    assert_eq!(res.streams[0].id, 1);
    assert_eq!(res.streams[0].address, stream_addr1);

    assert_eq!(res.streams[1].id, 2);
    assert_eq!(res.streams[1].address, stream_addr2);
}
