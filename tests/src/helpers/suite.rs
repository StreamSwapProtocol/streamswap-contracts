use crate::helpers::stargate::MyStargateKeeper;
use cosmwasm_std::testing::MockStorage;
use cosmwasm_std::Empty;
use cosmwasm_std::{coin, Addr, BlockInfo, Coin, Timestamp};
use cw_multi_test::addons::{MockAddressGenerator, MockApiBech32};
use cw_multi_test::{App, AppBuilder, BankKeeper, ContractWrapper, WasmKeeper};
use cw_multi_test::{
    DistributionKeeper, FailingModule, GovFailingModule, IbcFailingModule, StakeKeeper,
};

#[allow(dead_code)]

pub const PREFIX: &str = "cosmwasm";

#[allow(dead_code)]

type AppType = App<
    BankKeeper,
    MockApiBech32,
    MockStorage,
    FailingModule<Empty, Empty, Empty>,
    WasmKeeper<Empty, Empty>,
    StakeKeeper,
    DistributionKeeper,
    IbcFailingModule,
    GovFailingModule,
    MyStargateKeeper,
>;

#[allow(dead_code)]

pub(crate) struct Suite {
    pub app: AppType,
    pub test_accounts: TestAccounts,
    pub stream_swap_controller_code_id: u64,
    pub stream_swap_code_id: u64,
    pub vesting_code_id: u64,
}

#[derive(Default)]
pub(crate) struct SuiteBuilder {}

impl SuiteBuilder {
    #[allow(dead_code)]

    pub fn build(self) -> Suite {
        let denoms = [
            "fee_denom".to_string(),
            "out_denom".to_string(),
            "in_denom".to_string(),
            "wrong_denom".to_string(),
        ];
        let amount = 1_000_000_000_000_000u128;

        let api = MockApiBech32::new(PREFIX);
        let accounts = TestAccounts::new(&api);
        let mut app = AppBuilder::default()
            .with_api(api)
            .with_wasm(WasmKeeper::default().with_address_generator(MockAddressGenerator))
            .with_stargate(MyStargateKeeper {})
            .build(|router, _api, storage| {
                accounts.all().iter().for_each(|account| {
                    let coins: Vec<Coin> = denoms.iter().map(|d| coin(amount, d.clone())).collect();
                    router.bank.init_balance(storage, account, coins).unwrap();
                });
            });

        app.set_block(BlockInfo {
            chain_id: "test_1".to_string(),
            height: 1_000,
            time: Timestamp::from_seconds(1_000),
        });

        let stream_swap_controller_contract = Box::new(ContractWrapper::new(
            streamswap_controller::contract::execute,
            streamswap_controller::contract::instantiate,
            streamswap_controller::contract::query,
        ));
        let stream_swap_contract = Box::new(ContractWrapper::new(
            streamswap_stream::contract::execute,
            streamswap_stream::contract::instantiate,
            streamswap_stream::contract::query,
        ));
        let vesting_contract = Box::new(ContractWrapper::new(
            cw_vesting::contract::execute,
            cw_vesting::contract::instantiate,
            cw_vesting::contract::query,
        ));

        let stream_swap_code_id = app.store_code(stream_swap_contract);
        let stream_swap_controller_code_id = app.store_code(stream_swap_controller_contract);
        let vesting_code_id = app.store_code(vesting_contract);

        Suite {
            test_accounts: accounts,
            stream_swap_controller_code_id,
            stream_swap_code_id,
            vesting_code_id,
            app,
        }
    }
}

#[allow(dead_code)]

pub struct TestAccounts {
    pub admin: Addr,
    pub admin_2: Addr,
    pub creator_1: Addr,
    pub subscriber_1: Addr,
    pub subscriber_2: Addr,
    pub wrong_user: Addr,
    pub creator_2: Addr,
}

impl TestAccounts {
    #[allow(dead_code)]
    pub fn new(api: &MockApiBech32) -> TestAccounts {
        let admin = api.addr_make("admin");
        let admin_2 = api.addr_make("admin_2");
        let creator_1 = api.addr_make("creator_1");
        let creator_2 = api.addr_make("creator_2");
        let subscriber_1 = api.addr_make("subscriber_1");
        let subscriber_2 = api.addr_make("subscriber_2");
        let wrong_user = api.addr_make("wrong_user");

        TestAccounts {
            admin,
            admin_2,
            creator_1,
            subscriber_1,
            subscriber_2,
            wrong_user,
            creator_2,
        }
    }
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
}
