use cosmwasm_std::{coin, Addr, BlockInfo, Coin, Timestamp};
use cw_multi_test::{App, BankSudo, ContractWrapper, SudoMsg};
use cw_streamswap::contract::{
    execute as streamswap_execute, instantiate as streamswap_instantiate, query as streamswap_query,
};
use cw_streamswap_factory::contract::{
    execute as factory_execute, instantiate as factory_instantiate, query as factory_query,
};

pub fn setup() -> SetupResponse {
    let mut app = App::default();
    let admin = Addr::unchecked("admin");
    let creator = Addr::unchecked("stream_creator");
    let subscriber = Addr::unchecked("subscriber");

    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_seconds(1_000),
    });

    // All three accounts need to have some tokens
    mint_to_address(
        &mut app,
        admin.to_string(),
        vec![coin(1_000_000, "in_denom")],
    );
    mint_to_address(
        &mut app,
        creator.to_string(),
        vec![coin(1_000_000, "in_denom")],
    );
    mint_to_address(
        &mut app,
        subscriber.to_string(),
        vec![coin(1_000_000, "in_denom")],
    );

    mint_to_address(
        &mut app,
        admin.to_string(),
        vec![coin(1_000_000, "out_denom")],
    );
    mint_to_address(
        &mut app,
        creator.to_string(),
        vec![coin(1_000_000, "out_denom")],
    );
    mint_to_address(
        &mut app,
        subscriber.to_string(),
        vec![coin(1_000_000, "out_denom")],
    );

    mint_to_address(
        &mut app,
        admin.to_string(),
        vec![coin(1_000_000, "fee_token")],
    );
    mint_to_address(
        &mut app,
        creator.to_string(),
        vec![coin(1_000_000, "fee_token")],
    );
    mint_to_address(
        &mut app,
        subscriber.to_string(),
        vec![coin(1_000_000, "fee_token")],
    );
    mint_to_address(&mut app, admin.to_string(), vec![coin(1_000_000, "random")]);
    mint_to_address(
        &mut app,
        creator.to_string(),
        vec![coin(1_000_000, "random")],
    );
    mint_to_address(
        &mut app,
        subscriber.to_string(),
        vec![coin(1_000_000, "random")],
    );

    let stream_swap_factory_contract = Box::new(ContractWrapper::new(
        factory_execute,
        factory_instantiate,
        factory_query,
    ));
    let stream_swap_contract = Box::new(ContractWrapper::new(
        streamswap_execute,
        streamswap_instantiate,
        streamswap_query,
    ));

    let stream_swap_code_id = app.store_code(stream_swap_contract);
    let stream_swap_factory_code_id = app.store_code(stream_swap_factory_contract);

    let test_accounts = TestAccounts {
        admin: admin.clone(),
        creator: creator.clone(),
        subscriber: subscriber.clone(),
    };
    SetupResponse {
        test_accounts,
        stream_swap_factory_code_id,
        stream_swap_code_id,
        app,
    }
}

pub fn mint_to_address(app: &mut App, to_address: String, amount: Vec<Coin>) {
    app.sudo(SudoMsg::Bank(BankSudo::Mint { to_address, amount }))
        .unwrap();
}

pub struct SetupResponse {
    pub app: App,
    pub test_accounts: TestAccounts,
    pub stream_swap_factory_code_id: u64,
    pub stream_swap_code_id: u64,
}

pub struct TestAccounts {
    pub admin: Addr,
    pub creator: Addr,
    pub subscriber: Addr,
}
