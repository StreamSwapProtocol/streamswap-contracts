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
    let accounts = create_test_accounts();
    let denoms = vec![
        "fee_token".to_string(),
        "out_denom".to_string(),
        "in_denom".to_string(),
        "wrong_denom".to_string(),
    ];
    fund_accounts(&mut app, accounts.clone(), denoms.clone());

    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_seconds(1_000),
    });

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
        admin: accounts[0].clone(),
        creator: accounts[1].clone(),
        subscriber: accounts[2].clone(),
        subscriber_2: accounts[3].clone(),
        wrong_user: accounts[4].clone(),
    };

    SetupResponse {
        test_accounts,
        stream_swap_factory_code_id,
        stream_swap_code_id,
        app,
    }
}

fn create_test_accounts() -> Vec<Addr> {
    vec![
        Addr::unchecked("admin"),
        Addr::unchecked("stream_creator"),
        Addr::unchecked("subscriber"),
        Addr::unchecked("subscriber_2"),
        Addr::unchecked("wrong_user"),
    ]
}

fn fund_accounts(app: &mut App, accounts: Vec<Addr>, denoms: Vec<String>) {
    for account in accounts {
        for denom in &denoms {
            mint_to_address(app, account.to_string(), vec![coin(1_000_000, denom)]);
        }
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
    pub subscriber_2: Addr,
    pub wrong_user: Addr,
}
