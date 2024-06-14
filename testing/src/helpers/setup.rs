use cosmwasm_std::{coin, Addr, BlockInfo, Coin, Timestamp};
use cw_multi_test::{App, BankSudo, ContractWrapper, Executor, SudoMsg};
use streamswap_factory::contract::{
    execute as factory_execute, instantiate as factory_instantiate, query as factory_query,
};
use streamswap_stream::contract::{
    execute as streamswap_execute, instantiate as streamswap_instantiate, query as streamswap_query,
};

pub fn setup() -> SetupResponse {
    let mut app = App::default();
    let accounts = create_test_accounts();
    let denoms = vec![
        "fee_denom".to_string(),
        "out_denom".to_string(),
        "in_denom".to_string(),
        "wrong_denom".to_string(),
    ];
    accounts.fund_accounts(&mut app, denoms);
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

    SetupResponse {
        test_accounts: accounts,
        stream_swap_factory_code_id,
        stream_swap_code_id,
        app,
    }
}

fn create_test_accounts() -> TestAccounts {
    let admin = Addr::unchecked("admin");
    let creator_1 = Addr::unchecked("creator_1");
    let subscriber_1 = Addr::unchecked("subscriber_1");
    let subscriber_2 = Addr::unchecked("subscriber_2");
    let wrong_user = Addr::unchecked("wrong_user");
    let creator_2 = Addr::unchecked("creator_2");

    TestAccounts {
        admin,
        creator_1,
        subscriber_1,
        subscriber_2,
        wrong_user,
        creator_2,
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
    pub creator_1: Addr,
    pub subscriber_1: Addr,
    pub subscriber_2: Addr,
    pub wrong_user: Addr,
    pub creator_2: Addr,
}
impl TestAccounts {
    pub fn all(&self) -> Vec<Addr> {
        vec![
            self.admin.clone(),
            self.creator_1.clone(),
            self.subscriber_1.clone(),
            self.subscriber_2.clone(),
            self.wrong_user.clone(),
            self.creator_2.clone(),
        ]
    }

    pub fn fund_accounts(&self, app: &mut App, denoms: Vec<String>) {
        // Collect all accounts
        let accounts = self.all();
        denoms.iter().for_each(|denom| {
            let amount = 1_000_000_000_000_000u128;
            accounts.iter().for_each(|account| {
                mint_to_address(app, account.to_string(), vec![coin(amount, denom.clone())]);
            });
        });
    }
}
