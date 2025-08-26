use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::{Addr, BankMsg, Coin, CosmosMsg, Decimal256, Timestamp, Uint256, Uint64};

use cw_streamswap::contract::{
    execute, execute_create_stream, execute_finalize_stream, execute_update_position,
    execute_update_stream, list_positions, list_streams, query_position, query_stream, sudo,
};
use cw_streamswap::msg::{ExecuteMsg, InstantiateMsg, SudoMsg};
use cw_streamswap::ContractError;

mod helpers;

#[test]
fn killswitch_pause_protocol_admin() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Actors and mk_info
    let treasury: Addr = helpers::mock_info("treasury", &[]).sender;
    let creator1: Addr = helpers::mock_info("creator1", &[]).sender;
    let protocol_admin: Addr = helpers::mock_info("protocol_admin", &[]).sender;
    let non_protocol_admin: Addr = helpers::mock_info("non_protocol_admin", &[]).sender;
    let position1: Addr = helpers::mock_info("position1", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Create stream
    let start = Timestamp::from_seconds(1_000_000);
    let end = Timestamp::from_seconds(5_000_000);
    let out_supply = Uint256::from(1_000_000_000_000u128);
    let out_denom = "out_denom";
    let env = helpers::env_at(0);
    let info = mk_info(
        &creator1,
        vec![
            Coin::new(out_supply.to_string().parse::<u128>().unwrap(), out_denom),
            Coin::new(100u128, helpers::DEFAULT_STREAM_CREATION_DENOM),
        ],
    );
    execute_create_stream(
        deps.as_mut(),
        env,
        info,
        treasury.to_string(),
        "test".to_string(),
        Some("https://sample.url".to_string()),
        helpers::DEFAULT_ACCEPTED_IN_DENOM.to_string(),
        out_denom.to_string(),
        out_supply,
        start,
        end,
        None,
        "v1".to_string(),
    )
    .unwrap();

    // first subscription (ensure position exists)
    let env = helpers::env_at(start.plus_seconds(1_000_000).seconds());
    let info = mk_info(
        &position1,
        vec![Coin::new(3_000u128, helpers::DEFAULT_ACCEPTED_IN_DENOM)],
    );
    let msg = ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // non protocol admin can't pause
    let env = helpers::env_at(start.seconds() + 100);
    let err = execute(
        deps.as_mut(),
        env,
        mk_info(&non_protocol_admin, vec![]),
        ExecuteMsg::PauseStream { stream_id: 1 },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // can't pause before start time
    let env = helpers::env_at(start.minus_seconds(500_000).seconds());
    let err = execute(
        deps.as_mut(),
        env,
        mk_info(&protocol_admin, vec![]),
        ExecuteMsg::PauseStream { stream_id: 1 },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::StreamNotStarted {});

    // can't pause after end time
    let env = helpers::env_at(end.plus_seconds(500_000).seconds());
    let err = execute(
        deps.as_mut(),
        env,
        mk_info(&protocol_admin, vec![]),
        ExecuteMsg::PauseStream { stream_id: 1 },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::StreamEnded {});

    // protocol admin can pause
    let env = helpers::env_at(start.plus_seconds(1_000_001).seconds());
    execute(
        deps.as_mut(),
        env,
        mk_info(&protocol_admin, vec![]),
        ExecuteMsg::PauseStream { stream_id: 1 },
    )
    .unwrap();

    // can't pause if already paused
    let env = helpers::env_at(start.plus_seconds(1_000_005).seconds());
    let err = execute(
        deps.as_mut(),
        env,
        mk_info(&protocol_admin, vec![]),
        ExecuteMsg::PauseStream { stream_id: 1 },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::StreamKillswitchActive {});

    // can't subscribe new
    let env = helpers::env_at(start.plus_seconds(1_000_002).seconds());
    let info = mk_info(
        &Addr::clone(&helpers::mock_info("position2", &[]).sender),
        vec![Coin::new(3_000u128, helpers::DEFAULT_ACCEPTED_IN_DENOM)],
    );
    let msg = ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    let err = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::StreamKillswitchActive {});

    // can't subscribe more
    let info = mk_info(
        &position1,
        vec![Coin::new(3_000u128, helpers::DEFAULT_ACCEPTED_IN_DENOM)],
    );
    let msg = ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    let err = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::StreamKillswitchActive {});

    // can't withdraw
    let info = mk_info(&position1, vec![]);
    let msg = ExecuteMsg::Withdraw {
        stream_id: 1,
        cap: None,
        operator_target: None,
    };
    let err = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::StreamKillswitchActive {});

    // can't update stream
    let err = execute_update_stream(deps.as_mut(), env.clone(), 1).unwrap_err();
    assert_eq!(err, ContractError::StreamPaused {});

    // can't update position
    let info = mk_info(&position1, vec![]);
    let err = execute_update_position(deps.as_mut(), env, info, 1, None).unwrap_err();
    assert_eq!(err, ContractError::StreamPaused {});

    // can't finalize
    let env = helpers::env_at(end.plus_seconds(1_000_002).seconds());
    let info = mk_info(&treasury, vec![]);
    let err = execute_finalize_stream(deps.as_mut(), env, info, 1, None).unwrap_err();
    assert_eq!(err, ContractError::StreamKillswitchActive {});

    // can't exit
    let env = helpers::env_at(end.plus_seconds(1_000_002).seconds());
    let info = mk_info(&position1, vec![]);
    let msg = ExecuteMsg::ExitStream {
        stream_id: 1,
        operator_target: None,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::StreamKillswitchActive {});
}

#[test]
fn killswitch_resume_protocol_admin() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Actors
    let treasury: Addr = helpers::mock_info("treasury", &[]).sender;
    let creator1: Addr = helpers::mock_info("creator1", &[]).sender;
    let protocol_admin: Addr = helpers::mock_info("protocol_admin", &[]).sender;
    let non_protocol_admin: Addr = helpers::mock_info("non_protocol_admin", &[]).sender;
    let position1: Addr = helpers::mock_info("position1", &[]).sender;
    let position2: Addr = helpers::mock_info("position2", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Create stream
    let start = Timestamp::from_seconds(1_000_000);
    let end = Timestamp::from_seconds(5_000_000);
    let out_supply = Uint256::from(1_000_000_000_000u128);
    let out_denom = "out_denom";
    let env = helpers::env_at(0);
    let info = mk_info(
        &creator1,
        vec![
            Coin::new(out_supply.to_string().parse::<u128>().unwrap(), out_denom),
            Coin::new(100u128, helpers::DEFAULT_STREAM_CREATION_DENOM),
        ],
    );
    execute_create_stream(
        deps.as_mut(),
        env,
        info,
        treasury.to_string(),
        "test".to_string(),
        Some("https://sample.url".to_string()),
        helpers::DEFAULT_ACCEPTED_IN_DENOM.to_string(),
        out_denom.to_string(),
        out_supply,
        start,
        end,
        None,
        "v1".to_string(),
    )
    .unwrap();

    // first subscription
    let env = helpers::env_at(start.plus_seconds(1_000_000).seconds());
    let info = mk_info(
        &position1,
        vec![Coin::new(3_000u128, helpers::DEFAULT_ACCEPTED_IN_DENOM)],
    );
    let msg = ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // can't resume if not paused
    let env = helpers::env_at(start.plus_seconds(1_000_003).seconds());
    let err = execute(
        deps.as_mut(),
        env,
        mk_info(&protocol_admin, vec![]),
        ExecuteMsg::ResumeStream { stream_id: 1 },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::StreamNotPaused {});

    // pause
    let env = helpers::env_at(start.plus_seconds(1_000_001).seconds());
    execute(
        deps.as_mut(),
        env,
        mk_info(&protocol_admin, vec![]),
        ExecuteMsg::PauseStream { stream_id: 1 },
    )
    .unwrap();

    // can't subscribe new during pause
    let env = helpers::env_at(start.plus_seconds(1_000_002).seconds());
    let info = mk_info(
        &position2,
        vec![Coin::new(3_000u128, helpers::DEFAULT_ACCEPTED_IN_DENOM)],
    );
    let msg = ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::StreamKillswitchActive {});

    // non protocol admin can't resume
    let env = helpers::env_at(start.plus_seconds(1_000_003).seconds());
    let err = execute(
        deps.as_mut(),
        env,
        mk_info(&non_protocol_admin, vec![]),
        ExecuteMsg::ResumeStream { stream_id: 1 },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // protocol admin can resume
    let env = helpers::env_at(start.plus_seconds(1_000_003).seconds());
    execute(
        deps.as_mut(),
        env,
        mk_info(&protocol_admin, vec![]),
        ExecuteMsg::ResumeStream { stream_id: 1 },
    )
    .unwrap();

    // can subscribe after resume
    let env = helpers::env_at(start.plus_seconds(1_000_004).seconds());
    let info = mk_info(
        &position2,
        vec![Coin::new(3_000u128, helpers::DEFAULT_ACCEPTED_IN_DENOM)],
    );
    let msg = ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[1].key, "stream_id");
    assert_eq!(res.attributes[2].key, "in_balance");
    assert_eq!(res.attributes[3].key, "shares");
    assert_eq!(res.attributes[4].key, "index");
    assert_eq!(res.attributes[5].key, "last_updated");
    assert_eq!(res.attributes[6].key, "pending_purchase");
    assert_eq!(res.attributes[7].key, "purchased");
    assert_eq!(res.attributes[8].key, "spent");
    assert_eq!(res.attributes[9].key, "tos_version");
    assert_eq!(res.attributes[10].key, "stream_new_in_supply");
    assert_eq!(res.attributes[11].key, "stream_previous_in_supply");
    assert_eq!(res.attributes[12].key, "stream_new_shares");
    assert_eq!(res.attributes[13].key, "stream_previous_shares");
    assert_eq!(res.attributes[14].key, "stream_status");
    // values (must match the original test)
    assert_eq!(res.attributes[0].value, "subscribe");
    assert_eq!(res.attributes[1].value, "1");
    assert_eq!(res.attributes[2].value, "3000");
    assert_eq!(res.attributes[3].value, "3000");
    assert_eq!(res.attributes[4].value, "222.222");
    assert_eq!(res.attributes[5].value, "2000004.000000000");
    assert_eq!(res.attributes[6].value, "0");
    assert_eq!(res.attributes[7].value, "0");
    assert_eq!(res.attributes[8].value, "0");
    assert_eq!(res.attributes[9].value, "v1");
    assert_eq!(res.attributes[10].value, "6000");
    assert_eq!(res.attributes[11].value, "6000");
    assert_eq!(res.attributes[12].value, "6000");
    assert_eq!(res.attributes[13].value, "6000");
    assert_eq!(res.attributes[14].value, "Active");

    // protocol admin can pause then cancel then resume should fail with cancelled
    let env = helpers::env_at(start.plus_seconds(1_000_005).seconds());
    execute(
        deps.as_mut(),
        env,
        mk_info(&protocol_admin, vec![]),
        ExecuteMsg::PauseStream { stream_id: 1 },
    )
    .unwrap();

    let env = helpers::env_at(start.plus_seconds(1_000_006).seconds());
    execute(
        deps.as_mut(),
        env,
        mk_info(&protocol_admin, vec![]),
        ExecuteMsg::CancelStream { stream_id: 1 },
    )
    .unwrap();

    let env = helpers::env_at(start.plus_seconds(1_000_007).seconds());
    let err = execute(
        deps.as_mut(),
        env,
        mk_info(&protocol_admin, vec![]),
        ExecuteMsg::ResumeStream { stream_id: 1 },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::StreamIsCancelled {});
}

#[test]
fn killswitch_cancel_protocol_admin() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Actors
    let treasury: Addr = helpers::mock_info("treasury", &[]).sender;
    let creator1: Addr = helpers::mock_info("creator1", &[]).sender;
    let protocol_admin: Addr = helpers::mock_info("protocol_admin", &[]).sender;
    let non_protocol_admin: Addr = helpers::mock_info("non_protocol_admin", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Create stream
    let start = Timestamp::from_seconds(1_000_000);
    let end = Timestamp::from_seconds(5_000_000);
    let out_supply = Uint256::from(1_000_000_000_000u128);
    let out_denom = "out_denom";
    let env = helpers::env_at(0);
    let info = mk_info(
        &creator1,
        vec![
            Coin::new(out_supply.to_string().parse::<u128>().unwrap(), out_denom),
            Coin::new(100u128, helpers::DEFAULT_STREAM_CREATION_DENOM),
        ],
    );
    execute_create_stream(
        deps.as_mut(),
        env,
        info,
        treasury.to_string(),
        "test".to_string(),
        Some("https://sample.url".to_string()),
        helpers::DEFAULT_ACCEPTED_IN_DENOM.to_string(),
        out_denom.to_string(),
        out_supply,
        start,
        end,
        None,
        "v1".to_string(),
    )
    .unwrap();

    // Subscribe
    let env = helpers::env_at(start.seconds());
    let info = mk_info(
        &creator1,
        vec![Coin::new(
            2_000_000_000_000u128,
            helpers::DEFAULT_ACCEPTED_IN_DENOM,
        )],
    );
    let msg = ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: Some(helpers::mock_info("operator", &[]).sender.to_string()),
        tos_version: "v1".to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // non protocol admin can't cancel
    let env = helpers::env_at(start.plus_seconds(1_000_000).seconds());
    let err = execute(
        deps.as_mut(),
        env,
        mk_info(&non_protocol_admin, vec![]),
        ExecuteMsg::CancelStream { stream_id: 1 },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // can't cancel without pause
    let env = helpers::env_at(start.plus_seconds(1_000_000).seconds());
    let err = execute(
        deps.as_mut(),
        env,
        mk_info(&protocol_admin, vec![]),
        ExecuteMsg::CancelStream { stream_id: 1 },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::StreamNotPaused {});

    // pause then cancel
    let env = helpers::env_at(start.plus_seconds(2_000_000).seconds());
    execute(
        deps.as_mut(),
        env,
        mk_info(&protocol_admin, vec![]),
        ExecuteMsg::PauseStream { stream_id: 1 },
    )
    .unwrap();

    let env = helpers::env_at(start.plus_seconds(2_500_000).seconds());
    let res = execute(
        deps.as_mut(),
        env,
        mk_info(&protocol_admin, vec![]),
        ExecuteMsg::CancelStream { stream_id: 1 },
    )
    .unwrap();

    // out tokens and creation fee refunded to treasury
    assert_eq!(res.messages.len(), 2);
    if let cosmwasm_std::SubMsg {
        msg: CosmosMsg::Bank(BankMsg::Send { to_address, amount }),
        ..
    } = &res.messages[0]
    {
        assert_eq!(to_address, &treasury.to_string());
        assert_eq!(amount.len(), 1);
        assert_eq!(amount[0].denom, out_denom.to_string());
        assert_eq!(amount[0].amount, out_supply);
    } else {
        panic!("unexpected message variant");
    }
    if let cosmwasm_std::SubMsg {
        msg: CosmosMsg::Bank(BankMsg::Send { to_address, amount }),
        ..
    } = &res.messages[1]
    {
        assert_eq!(to_address, &treasury.to_string());
        assert_eq!(amount.len(), 1);
        assert_eq!(
            amount[0].denom,
            helpers::DEFAULT_STREAM_CREATION_DENOM.to_string()
        );
        assert_eq!(amount[0].amount, Uint256::from(100u128));
    } else {
        panic!("unexpected message variant");
    }

    // can't cancel cancelled
    let env = helpers::env_at(start.plus_seconds(2_500_000).seconds());
    let err = execute(
        deps.as_mut(),
        env,
        mk_info(&protocol_admin, vec![]),
        ExecuteMsg::CancelStream { stream_id: 1 },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::StreamIsCancelled {});
}
#[test]
fn killswitch_withdraw_paused_flow() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Actors
    let treasury: Addr = helpers::mock_info("treasury", &[]).sender;
    let creator1: Addr = helpers::mock_info("creator1", &[]).sender;
    let protocol_admin: Addr = helpers::mock_info("protocol_admin", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Create stream
    let start = Timestamp::from_seconds(1_000_000);
    let end = Timestamp::from_seconds(5_000_000);
    let out_supply = Uint256::from(1_000_000_000_000u128);
    let out_denom = "out_denom";
    let env = helpers::env_at(0);
    let info = mk_info(
        &creator1,
        vec![
            Coin::new(out_supply.to_string().parse::<u128>().unwrap(), out_denom),
            Coin::new(100u128, helpers::DEFAULT_STREAM_CREATION_DENOM),
        ],
    );
    execute_create_stream(
        deps.as_mut(),
        env,
        info,
        treasury.to_string(),
        "test".to_string(),
        Some("https://sample.url".to_string()),
        helpers::DEFAULT_ACCEPTED_IN_DENOM.to_string(),
        out_denom.to_string(),
        out_supply,
        start,
        end,
        None,
        "v1".to_string(),
    )
    .unwrap();

    // subscription
    let env = helpers::env_at(start.seconds());
    let funds = Coin::new(2_000_000_000_000u128, helpers::DEFAULT_ACCEPTED_IN_DENOM);
    let info = mk_info(&creator1, vec![funds.clone()]);
    let msg = ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: Some(helpers::mock_info("operator", &[]).sender.to_string()),
        tos_version: "v1".to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // normal withdraw (not paused) with cap succeeds before pause
    let env = helpers::env_at(start.plus_seconds(5_000).seconds());
    let cap = Uint256::from(25_000_000u128);
    execute(
        deps.as_mut(),
        env.clone(),
        mk_info(&creator1, vec![]),
        ExecuteMsg::Withdraw {
            stream_id: 1,
            cap: Some(cap),
            operator_target: None,
        },
    )
    .unwrap();

    let position = query_position(deps.as_ref(), env.clone(), 1, creator1.to_string()).unwrap();
    assert_eq!(position.in_balance, Uint256::from(1_997_475_000_000u128));
    assert_eq!(position.spent, Uint256::from(2_500_000_000u128));
    assert_eq!(position.purchased, Uint256::from(1_250_000_000u128));
    // first fund amount should be equal to in_balance + spent + cap
    assert_eq!(position.in_balance + position.spent + cap, funds.amount);

    // pause
    let env = helpers::env_at(start.plus_seconds(6_000).seconds());
    execute(
        deps.as_mut(),
        env,
        mk_info(&protocol_admin, vec![]),
        ExecuteMsg::PauseStream { stream_id: 1 },
    )
    .unwrap();

    // withdraw paused with cap
    let env = helpers::env_at(start.plus_seconds(7_000).seconds());
    execute(
        deps.as_mut(),
        env,
        mk_info(&creator1, vec![]),
        ExecuteMsg::WithdrawPaused {
            stream_id: 1,
            cap: Some(Uint256::from(25_000_000u128)),
            operator_target: None,
        },
    )
    .unwrap();

    // withdraw after pause (remaining)
    let env = helpers::env_at(start.plus_seconds(7_000).seconds());
    let _res = execute(
        deps.as_mut(),
        env,
        mk_info(&creator1, vec![]),
        ExecuteMsg::WithdrawPaused {
            stream_id: 1,
            cap: None,
            operator_target: None,
        },
    )
    .unwrap();

    // stream not updated; check state
    let env = helpers::env_at(start.plus_seconds(8_000).seconds());
    let stream_new = query_stream(deps.as_ref(), env, 1).unwrap();
    assert_eq!(stream_new.in_supply, Uint256::zero());
    assert_eq!(stream_new.shares, Uint256::zero());

    // position updated
    let position = query_position(
        deps.as_ref(),
        helpers::env_at(start.plus_seconds(8_001).seconds()),
        1,
        creator1.to_string(),
    )
    .unwrap();
    assert_eq!(position.in_balance, Uint256::zero());
    assert_eq!(position.spent, Uint256::from(2_999_993_742u128));
    assert_eq!(position.purchased, Uint256::from(1_499_999_998u128));
    assert_eq!(position.shares, Uint256::zero());
}

#[test]
fn killswitch_sudo_resume_updates_end_time() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Actors
    let treasury: Addr = helpers::mock_info("treasury", &[]).sender;
    let creator1: Addr = helpers::mock_info("creator1", &[]).sender;
    let protocol_admin: Addr = helpers::mock_info("protocol_admin", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Create stream
    let start = Timestamp::from_seconds(1_000_000);
    let end = Timestamp::from_seconds(5_000_000);
    let out_supply = Uint256::from(1_000_000_000_000u128);
    let out_denom = "out_denom";
    let env = helpers::env_at(0);
    let info = mk_info(
        &creator1,
        vec![
            Coin::new(out_supply.to_string().parse::<u128>().unwrap(), out_denom),
            Coin::new(100u128, helpers::DEFAULT_STREAM_CREATION_DENOM),
        ],
    );
    execute_create_stream(
        deps.as_mut(),
        env,
        info,
        treasury.to_string(),
        "test".to_string(),
        Some("https://sample.url".to_string()),
        helpers::DEFAULT_ACCEPTED_IN_DENOM.to_string(),
        out_denom.to_string(),
        out_supply,
        start,
        end,
        None,
        "v1".to_string(),
    )
    .unwrap();

    // first subscription
    let env = helpers::env_at(start.plus_seconds(1_000_000).seconds());
    let info = mk_info(
        &Addr::clone(&helpers::mock_info("position1", &[]).sender),
        vec![Coin::new(3_000u128, helpers::DEFAULT_ACCEPTED_IN_DENOM)],
    );
    let msg = ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // pause
    let pause_date = start.plus_seconds(2_000_000);
    let env = helpers::env_at(pause_date.seconds());
    execute(
        deps.as_mut(),
        env,
        mk_info(&protocol_admin, vec![]),
        ExecuteMsg::PauseStream { stream_id: 1 },
    )
    .unwrap();

    // resume via sudo
    let resume_date = start.plus_seconds(3_000_000);
    let env = helpers::env_at(resume_date.seconds());
    sudo(deps.as_mut(), env, SudoMsg::ResumeStream { stream_id: 1 }).unwrap();

    // new end date is correct
    let new_end_date = end.plus_nanos(resume_date.nanos() - pause_date.nanos());
    let stream = query_stream(deps.as_ref(), helpers::env_at(0), 1).unwrap();
    assert_eq!(stream.end_time, new_end_date);
}

#[test]
fn killswitch_sudo_pause_stream() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Actors
    let treasury: Addr = helpers::mock_info("treasury", &[]).sender;
    let creator1: Addr = helpers::mock_info("creator1", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Create stream
    let start = Timestamp::from_seconds(1_000_000);
    let end = Timestamp::from_seconds(5_000_000);
    let out_supply = Uint256::from(1_000_000_000_000u128);
    let out_denom = "out_denom";
    let env = helpers::env_at(0);
    let info = mk_info(
        &creator1,
        vec![
            Coin::new(out_supply.to_string().parse::<u128>().unwrap(), out_denom),
            Coin::new(100u128, helpers::DEFAULT_STREAM_CREATION_DENOM),
        ],
    );
    execute_create_stream(
        deps.as_mut(),
        env,
        info,
        treasury.to_string(),
        "test".to_string(),
        Some("https://sample.url".to_string()),
        helpers::DEFAULT_ACCEPTED_IN_DENOM.to_string(),
        out_denom.to_string(),
        out_supply,
        start,
        end,
        None,
        "v1".to_string(),
    )
    .unwrap();

    // not started
    let env = helpers::env_at(500_000);
    let err = sudo(deps.as_mut(), env, SudoMsg::PauseStream { stream_id: 1 }).unwrap_err();
    assert_eq!(err, ContractError::StreamNotStarted {});

    // ended
    let env = helpers::env_at(6_000_000);
    let err = sudo(deps.as_mut(), env, SudoMsg::PauseStream { stream_id: 1 }).unwrap_err();
    assert_eq!(err, ContractError::StreamEnded {});

    // success
    let env = helpers::env_at(3_000_000);
    let res = sudo(deps.as_mut(), env, SudoMsg::PauseStream { stream_id: 1 }).unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new()
            .add_attribute("action", "sudo_pause_stream")
            .add_attribute("stream_id", "1")
            .add_attribute("is_paused", "true")
            .add_attribute("pause_date", "3000000.000000000")
    );

    // double pause
    let env = helpers::env_at(4_000_000);
    let err = sudo(deps.as_mut(), env, SudoMsg::PauseStream { stream_id: 1 }).unwrap_err();
    assert_eq!(err, ContractError::StreamKillswitchActive {});
}

#[test]
fn killswitch_range_queries() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Actors
    let treasury: Addr = helpers::mock_info("treasury", &[]).sender;
    let creator1: Addr = helpers::mock_info("creator1", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    let start = Timestamp::from_seconds(2_000);
    let end = Timestamp::from_seconds(1_000_000);
    let out_supply = Uint256::from(1_000_000u128);
    let out_denom = "out_denom";

    // first stream
    let env = helpers::env_at(1);
    let info = mk_info(
        &creator1,
        vec![
            Coin::new(out_supply.to_string().parse::<u128>().unwrap(), out_denom),
            Coin::new(100u128, helpers::DEFAULT_STREAM_CREATION_DENOM),
        ],
    );
    execute_create_stream(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        treasury.to_string(),
        "test".to_string(),
        Some("https://sample.url".to_string()),
        helpers::DEFAULT_ACCEPTED_IN_DENOM.to_string(),
        out_denom.to_string(),
        out_supply,
        start,
        end,
        None,
        "v1".to_string(),
    )
    .unwrap();

    // second stream
    execute_create_stream(
        deps.as_mut(),
        env,
        info,
        treasury.to_string(),
        "test".to_string(),
        Some("https://sample.url".to_string()),
        helpers::DEFAULT_ACCEPTED_IN_DENOM.to_string(),
        out_denom.to_string(),
        out_supply,
        start,
        end,
        None,
        "v1".to_string(),
    )
    .unwrap();

    let res = list_streams(deps.as_ref(), None, None).unwrap();
    assert_eq!(res.streams.len(), 2);

    // subscriptions to first stream
    let env = helpers::env_at(start.plus_seconds(100).seconds());
    let info = mk_info(
        &creator1,
        vec![Coin::new(1_000_000u128, helpers::DEFAULT_ACCEPTED_IN_DENOM)],
    );
    let msg = ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    let env = helpers::env_at(start.plus_seconds(100).seconds());
    let info = mk_info(
        &Addr::clone(&helpers::mock_info("creator2", &[]).sender),
        vec![Coin::new(1_000_000u128, helpers::DEFAULT_ACCEPTED_IN_DENOM)],
    );
    let msg = ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: None,
        tos_version: "v1".to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    let res = list_positions(deps.as_ref(), 1, None, None).unwrap();
    assert_eq!(res.positions.len(), 2);
}

#[test]
fn killswitch_exit_cancel() {
    let mut deps = mock_dependencies();
    helpers::instantiate_defaults(deps.as_mut());

    // Actors
    let treasury: Addr = helpers::mock_info("treasury", &[]).sender;
    let creator1: Addr = helpers::mock_info("creator1", &[]).sender;
    let protocol_admin: Addr = helpers::mock_info("protocol_admin", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Create stream
    let start = Timestamp::from_seconds(1_000_000);
    let end = Timestamp::from_seconds(5_000_000);
    let out_supply = Uint256::from(1_000_000_000_000u128);
    let out_denom = "out_denom";
    let env = helpers::env_at(0);
    let info = mk_info(
        &creator1,
        vec![
            Coin::new(out_supply.to_string().parse::<u128>().unwrap(), out_denom),
            Coin::new(100u128, helpers::DEFAULT_STREAM_CREATION_DENOM),
        ],
    );
    execute_create_stream(
        deps.as_mut(),
        env,
        info,
        treasury.to_string(),
        "test".to_string(),
        Some("https://sample.url".to_string()),
        helpers::DEFAULT_ACCEPTED_IN_DENOM.to_string(),
        out_denom.to_string(),
        out_supply,
        start,
        end,
        None,
        "v1".to_string(),
    )
    .unwrap();

    // subscription
    let env = helpers::env_at(start.seconds());
    let info = mk_info(
        &creator1,
        vec![Coin::new(
            2_000_000_000_000u128,
            helpers::DEFAULT_ACCEPTED_IN_DENOM,
        )],
    );
    let msg = ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: Some(helpers::mock_info("operator", &[]).sender.to_string()),
        tos_version: "v1".to_string(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // cant cancel without pause (sudo)
    let env = helpers::env_at(start.plus_seconds(1_000_000).seconds());
    let err = sudo(deps.as_mut(), env, SudoMsg::CancelStream { stream_id: 1 }).unwrap_err();
    assert_eq!(err, ContractError::StreamNotPaused {});

    // pause
    let env = helpers::env_at(start.plus_seconds(2_000_000).seconds());
    execute(
        deps.as_mut(),
        env,
        mk_info(&protocol_admin, vec![]),
        ExecuteMsg::PauseStream { stream_id: 1 },
    )
    .unwrap();

    // can't exit before cancel (ExitCancelled)
    let env = helpers::env_at(start.plus_seconds(2_250_000).seconds());
    let err = execute(
        deps.as_mut(),
        env,
        mk_info(&creator1, vec![]),
        ExecuteMsg::ExitCancelled {
            stream_id: 1,
            operator_target: None,
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::StreamNotCancelled {});

    // cancel
    let env = helpers::env_at(start.plus_seconds(2_500_000).seconds());
    let res = sudo(deps.as_mut(), env, SudoMsg::CancelStream { stream_id: 1 }).unwrap();
    assert_eq!(res.messages.len(), 2);
    // first message
    if let cosmwasm_std::SubMsg {
        msg: CosmosMsg::Bank(BankMsg::Send { to_address, amount }),
        ..
    } = &res.messages[0]
    {
        assert_eq!(to_address, &treasury.to_string());
        assert_eq!(amount.len(), 1);
        assert_eq!(amount[0].denom, out_denom.to_string());
        assert_eq!(amount[0].amount, out_supply);
    } else {
        panic!("unexpected message variant");
    }
    // second message
    if let cosmwasm_std::SubMsg {
        msg: CosmosMsg::Bank(BankMsg::Send { to_address, amount }),
        ..
    } = &res.messages[1]
    {
        assert_eq!(to_address, &treasury.to_string());
        assert_eq!(amount.len(), 1);
        assert_eq!(
            amount[0].denom,
            helpers::DEFAULT_STREAM_CREATION_DENOM.to_string()
        );
        assert_eq!(amount[0].amount, Uint256::from(100u128));
    } else {
        panic!("unexpected message variant");
    }

    // random operator can't exit on behalf
    let env = helpers::env_at(start.plus_seconds(2_250_000).seconds());
    let err = execute(
        deps.as_mut(),
        env,
        mk_info(
            &Addr::clone(&helpers::mock_info("random", &[]).sender),
            vec![],
        ),
        ExecuteMsg::ExitCancelled {
            stream_id: 1,
            operator_target: Some(creator1.to_string()),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // exit after cancellation
    let env = helpers::env_at(start.plus_seconds(3_000_000).seconds());
    let res = execute(
        deps.as_mut(),
        env,
        mk_info(&creator1, vec![]),
        ExecuteMsg::ExitCancelled {
            stream_id: 1,
            operator_target: None,
        },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);
    if let cosmwasm_std::SubMsg {
        msg: CosmosMsg::Bank(BankMsg::Send { to_address, amount }),
        ..
    } = &res.messages[0]
    {
        assert_eq!(to_address, &creator1.to_string());
        assert_eq!(
            amount,
            &vec![Coin::new(
                2_000_000_000_000u128,
                helpers::DEFAULT_ACCEPTED_IN_DENOM
            )]
        );
    } else {
        panic!("unexpected message variant");
    }
}

#[test]
fn killswitch_treasury_cancel_stream_window() {
    let mut deps = mock_dependencies();

    // instantiate with custom min_seconds_until_start_time = 100_000
    let msg = InstantiateMsg {
        min_stream_seconds: Uint64::new(1000),
        min_seconds_until_start_time: Uint64::new(100_000),
        stream_creation_denom: helpers::DEFAULT_STREAM_CREATION_DENOM.to_string(),
        stream_creation_fee: Uint256::from(100u128),
        exit_fee_percent: Decimal256::percent(1),
        fee_collector: helpers::mock_info("collector", &[]).sender.to_string(),
        protocol_admin: helpers::mock_info("protocol_admin", &[]).sender.to_string(),
        accepted_in_denom: "in_denom".to_string(),
        tos_version: "v1".to_string(),
    };
    cw_streamswap::contract::instantiate(
        deps.as_mut(),
        helpers::env_now(),
        helpers::mock_info("creator", &[]),
        msg,
    )
    .unwrap();

    // Actors
    let treasury: Addr = helpers::mock_info("treasury", &[]).sender;
    let creator: Addr = helpers::mock_info("creator", &[]).sender;
    let mk_info = |sender: &Addr, funds: Vec<Coin>| cosmwasm_std::MessageInfo {
        sender: sender.clone(),
        funds,
    };

    // Create stream
    let start = Timestamp::from_seconds(1_000_000);
    let end = Timestamp::from_seconds(5_000_000);
    let out_supply = Uint256::from(500u128);
    let out_denom = "out_denom";
    let in_denom = "in_denom";
    let env = helpers::env_at(0);
    let info = mk_info(
        &creator,
        vec![
            Coin::new(out_supply.to_string().parse::<u128>().unwrap(), out_denom),
            Coin::new(100u128, helpers::DEFAULT_STREAM_CREATION_DENOM),
        ],
    );
    execute_create_stream(
        deps.as_mut(),
        env,
        info,
        treasury.to_string(),
        "test".to_string(),
        Some("https://sample.url".to_string()),
        in_denom.to_string(),
        out_denom.to_string(),
        out_supply,
        start,
        end,
        Some(Uint256::from(1_000u128)),
        "v1".to_string(),
    )
    .unwrap();

    // Cancel period should be active until now+100_000

    // Cancel stream with wrong address
    let env = helpers::env_at(100);
    let err = execute(
        deps.as_mut(),
        env,
        helpers::mock_info("random", &[]),
        ExecuteMsg::TreasuryCancelStream { stream_id: 1 },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Try subscribing to the stream during cancel period
    let env = helpers::env_at(100);
    let info = mk_info(
        &Addr::clone(&helpers::mock_info("subscriber", &[]).sender),
        vec![Coin::new(250u128, in_denom)],
    );
    let msg = ExecuteMsg::Subscribe {
        stream_id: 1,
        operator_target: None,
        operator: Some(helpers::mock_info("operator", &[]).sender.to_string()),
        tos_version: "v1".to_string(),
    };
    let err = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::TreasuryCancelPeriodActive {});

    // Try canceling outside the cancel period
    let env = helpers::env_at(100_000 + 1);
    let err = execute(
        deps.as_mut(),
        env.clone(),
        helpers::mock_info("treasury", &[]),
        ExecuteMsg::TreasuryCancelStream { stream_id: 1 },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::TreasuryCancelPeriodEnded {});

    // Query the stream
    let _ = query_stream(deps.as_ref(), env.clone(), 1).unwrap();

    // Cancel at exact end of period
    let env = helpers::env_at(100_000);
    let res = execute(
        deps.as_mut(),
        env.clone(),
        helpers::mock_info("treasury", &[]),
        ExecuteMsg::TreasuryCancelStream { stream_id: 1 },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);
    if let cosmwasm_std::SubMsg {
        msg: CosmosMsg::Bank(BankMsg::Send { to_address, amount }),
        ..
    } = &res.messages[0]
    {
        assert_eq!(to_address, &treasury.to_string());
        assert_eq!(amount.len(), 1);
        assert_eq!(amount[0].denom, out_denom.to_string());
        assert_eq!(amount[0].amount, out_supply);
    } else {
        panic!("unexpected message variant");
    }

    // Querying the stream again should error since it's removed
    let _err = query_stream(deps.as_ref(), env, 1).unwrap_err();
}
