#[cfg(test)]
mod tests {
    use crate::contract::{
        execute_create_stream, execute_exit_stream, execute_finalize_stream, execute_subscribe,
        execute_trigger_purchase, execute_update_dist_index, execute_withdraw, instantiate,
        query_position, query_stream, update_dist_index,
    };
    use crate::state::Stream;
    use crate::ContractError;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{Addr, BankMsg, Coin, CosmosMsg, Decimal, Timestamp, Uint128, Uint64};
    use cw_utils::PaymentError;
    use std::str::FromStr;

    #[test]
    fn test_create_stream() {
        let mut deps = mock_dependencies();
        let msg = crate::msg::InstantiateMsg {
            min_stream_duration: Uint64::new(1000),
            min_duration_until_start_time: Uint64::new(1000),
            stream_creation_denom: "fee".to_string(),
            stream_creation_fee: Uint128::new(100),
            fee_collector: "collector".to_string(),
        };
        instantiate(deps.as_mut(), mock_env(), mock_info("creator", &[]), msg).unwrap();

        // end < start case
        let treasury = "treasury";
        let name = "name";
        let url = "url";
        let start_time = Timestamp::from_seconds(1000);
        let end_time = Timestamp::from_seconds(10);
        let out_supply = Uint128::new(50_000_000);
        let out_denom = "out_denom";
        let in_denom = "in_denom";

        let info = mock_info("creator", &[]);
        let env = mock_env();
        let res = execute_create_stream(
            deps.as_mut(),
            env,
            info,
            treasury.to_string(),
            name.to_string(),
            url.to_string(),
            in_denom.to_string(),
            out_denom.to_string(),
            out_supply,
            start_time,
            end_time,
        );
        assert_eq!(res, Err(ContractError::StreamInvalidEndTime {}));

        // min_stream_duration is not sufficient
        let end_time = Timestamp::from_seconds(1000);
        let start_time = Timestamp::from_seconds(500);
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(0);
        let info = mock_info("creator1", &[]);
        let res = execute_create_stream(
            deps.as_mut(),
            env,
            info,
            treasury.to_string(),
            name.to_string(),
            url.to_string(),
            in_denom.to_string(),
            out_denom.to_string(),
            out_supply,
            start_time,
            end_time,
        );
        assert_eq!(res, Err(ContractError::StreamDurationTooShort {}));

        // start cannot be before current time
        let end_time = Timestamp::from_seconds(1000);
        let start_time = Timestamp::from_seconds(500);
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(600);
        let info = mock_info("creator1", &[]);
        let res = execute_create_stream(
            deps.as_mut(),
            env,
            info,
            treasury.to_string(),
            name.to_string(),
            url.to_string(),
            in_denom.to_string(),
            out_denom.to_string(),
            out_supply,
            start_time,
            end_time,
        );
        assert_eq!(res, Err(ContractError::StreamInvalidStartTime {}));

        // stream starts too soon case
        let end_time = Timestamp::from_seconds(100000);
        let start_time = Timestamp::from_seconds(1400);
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(401);
        let info = mock_info("creator1", &[]);
        let res = execute_create_stream(
            deps.as_mut(),
            env,
            info,
            treasury.to_string(),
            name.to_string(),
            url.to_string(),
            in_denom.to_string(),
            out_denom.to_string(),
            out_supply,
            start_time,
            end_time,
        );
        assert_eq!(res, Err(ContractError::StreamStartsTooSoon {}));

        // no funds fee case
        let end_time = Timestamp::from_seconds(100000);
        let start_time = Timestamp::from_seconds(3000);
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1);
        let info = mock_info("creator1", &[]);
        let res = execute_create_stream(
            deps.as_mut(),
            env,
            info,
            treasury.to_string(),
            name.to_string(),
            url.to_string(),
            in_denom.to_string(),
            out_denom.to_string(),
            out_supply,
            start_time,
            end_time,
        );
        assert_eq!(res, Err(ContractError::NoFundsSent {}));

        // wrong supply amount case
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1);
        let info = mock_info("creator1", &[Coin::new(1_000_000, "out_denom")]);
        let res = execute_create_stream(
            deps.as_mut(),
            env,
            info,
            treasury.to_string(),
            name.to_string(),
            url.to_string(),
            in_denom.to_string(),
            out_denom.to_string(),
            out_supply,
            start_time,
            end_time,
        );
        assert_eq!(res, Err(ContractError::StreamOutSupplyFundsRequired {}));

        // no creation fee case
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1);
        let info = mock_info("creator1", &[Coin::new(out_supply.u128(), "out_denom")]);
        let res = execute_create_stream(
            deps.as_mut(),
            env,
            info,
            treasury.to_string(),
            name.to_string(),
            url.to_string(),
            in_denom.to_string(),
            out_denom.to_string(),
            out_supply,
            start_time,
            end_time,
        );
        assert_eq!(res, Err(ContractError::NoFundsSent {}));

        // mismatch creation fee case
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1);
        let info = mock_info("creator1", &[Coin::new(out_supply.u128(), "out_denom")]);
        let res = execute_create_stream(
            deps.as_mut(),
            env,
            info,
            treasury.to_string(),
            name.to_string(),
            url.to_string(),
            in_denom.to_string(),
            out_denom.to_string(),
            out_supply,
            start_time,
            end_time,
        );
        assert_eq!(res, Err(ContractError::NoFundsSent {}));

        // happy path
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1);
        let info = mock_info(
            "creator1",
            &[
                Coin::new(out_supply.u128(), "out_denom"),
                Coin::new(100, "fee"),
            ],
        );
        execute_create_stream(
            deps.as_mut(),
            env,
            info,
            treasury.to_string(),
            name.to_string(),
            url.to_string(),
            in_denom.to_string(),
            out_denom.to_string(),
            out_supply,
            start_time,
            end_time,
        )
        .unwrap();

        // query stream with id
        let env = mock_env();
        let stream = query_stream(deps.as_ref(), env, 1).unwrap();
        assert_eq!(stream.id, 1);
    }

    #[test]
    fn test_subscribe() {
        let treasury = Addr::unchecked("treasury");
        let start = Timestamp::from_seconds(2000);
        let end = Timestamp::from_seconds(1_000_000);
        let out_supply = Uint128::new(1_000_000);
        let out_denom = "out_denom";

        // instantiate
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100);
        let msg = crate::msg::InstantiateMsg {
            min_stream_duration: Uint64::new(1000),
            min_duration_until_start_time: Uint64::new(1000),
            stream_creation_denom: "fee".to_string(),
            stream_creation_fee: Uint128::new(100),
            fee_collector: "collector".to_string(),
        };
        instantiate(deps.as_mut(), mock_env(), mock_info("creator", &[]), msg).unwrap();

        // create stream
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1);
        let info = mock_info(
            "creator1",
            &[
                Coin::new(out_supply.u128(), out_denom),
                Coin::new(100, "fee"),
            ],
        );
        execute_create_stream(
            deps.as_mut(),
            env,
            info,
            treasury.to_string(),
            "test".to_string(),
            "test".to_string(),
            "in".to_string(),
            out_denom.to_string(),
            out_supply,
            start,
            end,
        )
        .unwrap();

        // stream did not begin yet
        let mut env = mock_env();
        env.block.time = start.minus_seconds(100);
        let info = mock_info("creator1", &[]);
        let res = execute_subscribe(deps.as_mut(), env, info, 1).unwrap_err();
        assert_eq!(res, ContractError::StreamNotStarted {});

        // stream ended
        let mut env = mock_env();
        env.block.time = end.plus_seconds(1000000);
        let info = mock_info("creator1", &[]);
        let res = execute_subscribe(deps.as_mut(), env, info, 1).unwrap_err();
        assert_eq!(res, ContractError::StreamEnded {});

        // no funds
        let mut env = mock_env();
        env.block.time = start.plus_seconds(100);
        let info = mock_info("creator1", &[]);
        let res = execute_subscribe(deps.as_mut(), env, info, 1).unwrap_err();
        assert_eq!(res, PaymentError::NoFunds {}.into());

        // incorrect denom
        let mut env = mock_env();
        env.block.time = start.plus_seconds(100);
        let info = mock_info("creator1", &[Coin::new(100, "wrong_denom")]);
        let res = execute_subscribe(deps.as_mut(), env, info, 1).unwrap_err();
        assert_eq!(
            res,
            PaymentError::MissingDenom {
                0: "in".to_string()
            }
            .into()
        );

        // first subscribe
        let mut env = mock_env();
        env.block.time = start.plus_seconds(100);
        let info = mock_info("creator1", &[Coin::new(1_000_000, "in")]);
        execute_subscribe(deps.as_mut(), env, info, 1).unwrap();

        // dist index updated
        let env = mock_env();
        let stream = query_stream(deps.as_ref(), env.clone(), 1).unwrap();
        // position index not updated, stage updated, in_supply updated
        assert_eq!(stream.dist_index, Decimal::zero());
        assert_eq!(
            stream.current_stage,
            Decimal::from_atomics(Uint128::new(100200400801603), 18).unwrap()
        );
        assert_eq!(stream.total_in_supply, Uint128::new(1000000));
        let position = query_position(deps.as_ref(), env, 1, "creator1".to_string()).unwrap();
        assert_eq!(position.index, Decimal::zero());
        assert_eq!(
            position.current_stage,
            Decimal::from_atomics(Uint128::new(100200400801603), 18).unwrap()
        );
        assert_eq!(position.in_balance, Uint128::new(1000000));

        // subscription increase
        let mut env = mock_env();
        env.block.time = start.plus_seconds(200);
        let info = mock_info("creator1", &[Coin::new(1_000_000, "in")]);
        execute_subscribe(deps.as_mut(), env.clone(), info, 1).unwrap();
        // dist index updated and stage
        let stream = query_stream(deps.as_ref(), env.clone(), 1).unwrap();
        assert_eq!(
            stream.dist_index,
            Decimal::from_atomics(Uint128::new(100010001000100), 18).unwrap()
        );
        assert_eq!(
            stream.current_stage,
            Decimal::from_atomics(Uint128::new(200400801603206), 18).unwrap()
        );
        // dist index updated and stage, position reduced and increased
        let position = query_position(deps.as_ref(), env, 1, "creator1".to_string()).unwrap();
        assert_eq!(
            position.index,
            Decimal::from_atomics(Uint128::new(100010001000100), 18).unwrap()
        );
        assert_eq!(
            position.current_stage,
            Decimal::from_atomics(Uint128::new(200400801603206), 18).unwrap()
        );
        assert_eq!(position.in_balance, Uint128::new(1999900));
    }

    #[test]
    fn test_update_index() {
        let treasury = Addr::unchecked("treasury");
        let start = Timestamp::from_seconds(0);
        let end = Timestamp::from_seconds(1_000_000);
        let out_supply = Uint128::new(1_000_000);
        let _cumulative_out = Uint128::zero();

        let mut stream = Stream::new(
            "test".to_string(),
            treasury,
            "test_url".to_string(),
            "out".to_string(),
            out_supply,
            "in".to_string(),
            start,
            end,
        );
        let now = Timestamp::from_seconds(100);

        // current_stage = 100 / 1_000_000 = 0.0001
        // new_distribution = 0.0001 * 1_000_000 = 100
        update_dist_index(now, &mut stream).unwrap();
        assert_eq!(stream.current_stage, Decimal::from_str("0.0001").unwrap());

        // no in supply, should be 0
        assert_eq!(stream.current_out, Uint128::new(0));
        assert_eq!(stream.dist_index, Decimal::zero());

        // out supply not changed
        assert_eq!(stream.out_supply, out_supply);

        /*
        user1 subscribes 100_000 at %1
        current_stage = %1
        */
        let now = Timestamp::from_seconds(10_000);
        update_dist_index(now, &mut stream).unwrap();
        // still no supply
        assert_eq!(stream.current_stage, Decimal::from_str("0.01").unwrap());
        assert_eq!(stream.current_out, Uint128::new(0));
        assert_eq!(stream.dist_index, Decimal::zero());
        assert_eq!(stream.out_supply, out_supply);
        stream.in_supply += Uint128::new(100_000);

        /*
        update_dist_index triggers at %2
        */

        // stage_diff is %1
        // spent_in = 100_000 * %1 = 1_000
        // in_supply = in_supply - spent_in = 100_000 - 10_000 = 99_000
        // new_distribution =  1_000_000 * 1 / 100 = 10_000
        // current_out = 0 + new_distribution = 100_000
        // new_dist_index = 0 + 10_000 / 99_000 = 0.1010101...
        let now = Timestamp::from_seconds(20_000);
        update_dist_index(now, &mut stream).unwrap();
        assert_eq!(stream.current_stage, Decimal::from_str("0.02").unwrap());
        assert_eq!(stream.current_out, Uint128::new(10_000));
        assert_eq!(
            stream.dist_index,
            Decimal::from_str("0.10101010101010101").unwrap()
        );
        assert_eq!(stream.in_supply, Uint128::new(99_000));

        /*
        user2 subscribes 100_000 at %4
        */
        let now = Timestamp::from_seconds(40_000);
        update_dist_index(now, &mut stream).unwrap();
        // TODO: to be cont
    }

    #[test]
    fn test_trigger_purchase() {
        let treasury = Addr::unchecked("treasury");
        let start = Timestamp::from_seconds(1_000_000);
        let end = Timestamp::from_seconds(5_000_000);
        let out_supply = Uint128::new(1_000_000);
        let out_denom = "out_denom";

        // instantiate
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100);
        let msg = crate::msg::InstantiateMsg {
            min_stream_duration: Uint64::new(1000),
            min_duration_until_start_time: Uint64::new(1000),
            stream_creation_denom: "fee".to_string(),
            stream_creation_fee: Uint128::new(100),
            fee_collector: "collector".to_string(),
        };
        instantiate(deps.as_mut(), mock_env(), mock_info("creator", &[]), msg).unwrap();

        // create stream
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1);
        let info = mock_info(
            "creator",
            &[
                Coin::new(out_supply.u128(), out_denom),
                Coin::new(100, "fee"),
            ],
        );
        execute_create_stream(
            deps.as_mut(),
            env,
            info,
            treasury.to_string(),
            "test".to_string(),
            "test".to_string(),
            "in".to_string(),
            out_denom.to_string(),
            out_supply,
            start,
            end,
        )
        .unwrap();

        // first subscription
        let mut env = mock_env();
        env.block.time = start.plus_seconds(100);
        let info = mock_info("creator1", &[Coin::new(1_000_000, "in")]);
        execute_subscribe(deps.as_mut(), env, info, 1).unwrap();

        // trigger purchase
        let mut env = mock_env();
        env.block.time = start.plus_seconds(3_000_000);
        let info = mock_info("creator1", &[]);
        execute_trigger_purchase(deps.as_mut(), env.clone(), info, 1).unwrap();

        let position =
            query_position(deps.as_ref(), env.clone(), 1, "creator1".to_string()).unwrap();
        assert_eq!(position.current_stage, Decimal::from_str("0.75").unwrap());
        assert_eq!(
            position.index,
            Decimal::from_str("2.999600039996000399").unwrap()
        );
        assert_eq!(position.purchased, Uint128::new(749974));
        assert_eq!(position.spent, Uint128::new(749975));
        assert_eq!(position.in_balance, Uint128::new(250025));
        let stream = query_stream(deps.as_ref(), env, 1).unwrap();
        assert_eq!(stream.current_stage, Decimal::from_str("0.75").unwrap());
        assert_eq!(
            stream.dist_index,
            Decimal::from_str("2.999600039996000399").unwrap()
        );

        // can trigger purchase after stream ends
        let mut env = mock_env();
        env.block.time = end.plus_seconds(1);
        let info = mock_info("creator1", &[]);
        execute_trigger_purchase(deps.as_mut(), env.clone(), info, 1).unwrap();
        let stream = query_stream(deps.as_ref(), env.clone(), 1).unwrap();
        assert_eq!(stream.current_stage, Decimal::one());
        assert_eq!(
            stream.dist_index,
            Decimal::from_str("4.33279827590809464").unwrap()
        );
        assert_eq!(stream.total_in_supply, Uint128::new(187519));
        let position = query_position(deps.as_ref(), env, 1, "creator1".to_string()).unwrap();
        assert_eq!(position.current_stage, Decimal::one());
        assert_eq!(
            position.index,
            Decimal::from_str("4.33279827590809464").unwrap()
        );
        assert_eq!(position.spent, Uint128::new(812481));
        assert_eq!(position.in_balance, Uint128::new(187519));

        // TODO: 999975 -999973 = 2? calculation leftover, gotta test it with bigger values
        assert_eq!(stream.total_out_sold, Uint128::new(999975));
        assert_eq!(position.purchased, Uint128::new(999973));
    }

    // this is for testing the leftover amount with bigger values
    #[test]
    fn test_rounding_leftover() {
        let treasury = Addr::unchecked("treasury");
        let start = Timestamp::from_seconds(1_000_000);
        let end = Timestamp::from_seconds(5_000_000);
        let out_supply = Uint128::new(1_000_000_000_000);
        let out_denom = "out_denom";

        // instantiate
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100);
        let msg = crate::msg::InstantiateMsg {
            min_stream_duration: Uint64::new(1000),
            min_duration_until_start_time: Uint64::new(1000),
            stream_creation_denom: "fee".to_string(),
            stream_creation_fee: Uint128::new(100),
            fee_collector: "collector".to_string(),
        };
        instantiate(deps.as_mut(), mock_env(), mock_info("creator", &[]), msg).unwrap();

        // create stream
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1);
        let info = mock_info(
            "creator",
            &[
                Coin::new(out_supply.u128(), out_denom),
                Coin::new(100, "fee"),
            ],
        );
        execute_create_stream(
            deps.as_mut(),
            env,
            info,
            treasury.to_string(),
            "test".to_string(),
            "test".to_string(),
            "in".to_string(),
            out_denom.to_string(),
            out_supply,
            start,
            end,
        )
        .unwrap();

        // first subscription
        let mut env = mock_env();
        env.block.time = start.plus_seconds(100);
        let info = mock_info("creator1", &[Coin::new(1_000_000_000, "in")]);
        execute_subscribe(deps.as_mut(), env, info, 1).unwrap();

        // second subscription
        let mut env = mock_env();
        env.block.time = start.plus_seconds(100_000);
        let info = mock_info("creator2", &[Coin::new(3_000_000_000, "in")]);
        execute_subscribe(deps.as_mut(), env, info, 1).unwrap();

        // trigger purchase creator1
        let mut env = mock_env();
        env.block.time = start.plus_seconds(3_000_000);
        let info = mock_info("creator1", &[]);
        execute_trigger_purchase(deps.as_mut(), env.clone(), info, 1).unwrap();

        let position =
            query_position(deps.as_ref(), env.clone(), 1, "creator1".to_string()).unwrap();
        assert_eq!(position.current_stage, Decimal::from_str("0.75").unwrap());
        assert_eq!(
            position.index,
            Decimal::from_str("688.846691491528021074").unwrap()
        );
        assert_eq!(position.purchased, Uint128::new(172_228_894_040));
        assert_eq!(position.spent, Uint128::new(749_975_000));
        assert_eq!(position.in_balance, Uint128::new(250_025_000));
        let stream = query_stream(deps.as_ref(), env, 1).unwrap();
        assert_eq!(stream.current_stage, Decimal::from_str("0.75").unwrap());
        assert_eq!(
            stream.dist_index,
            Decimal::from_str("688.846691491528021074").unwrap()
        );

        // trigger purchase creator2
        let mut env = mock_env();
        env.block.time = start.plus_seconds(3_575_000);
        let info = mock_info("creator2", &[]);
        execute_trigger_purchase(deps.as_mut(), env.clone(), info, 1).unwrap();


        let position =
            query_position(deps.as_ref(), env.clone(), 1, "creator2".to_string()).unwrap();
        assert_eq!(position.current_stage, Decimal::from_str("0.89375").unwrap());
        println!("{}", position.index.to_string());
        assert_eq!(
            position.index,
            Decimal::from_str("842.426708242230681435").unwrap()
        );
        assert_eq!(position.purchased, Uint128::new(321_619_717_288));
        assert_eq!(position.spent, Uint128::new(2_606_250_000));
        assert_eq!(position.in_balance, Uint128::new(393_750_000));
        let stream = query_stream(deps.as_ref(), env, 1).unwrap();
        assert_eq!(stream.current_stage, Decimal::from_str("0.89375").unwrap());
        println!("{}", stream.dist_index.to_string());
        assert_eq!(
            stream.dist_index,
            Decimal::from_str("842.426708242230681435").unwrap()
        );



        // trigger purchase after stream ends
        let mut env = mock_env();
        env.block.time = end.plus_seconds(1);
        let info = mock_info("creator1", &[]);
        execute_trigger_purchase(deps.as_mut(), env.clone(), info, 1).unwrap();
        let stream = query_stream(deps.as_ref(), env.clone(), 1).unwrap();
        assert_eq!(stream.current_stage, Decimal::one());
        println!("{}", stream.dist_index.to_string());
        assert_eq!(
            stream.dist_index,
            Decimal::from_str("969.437241956774606129").unwrap()
        );
        assert_eq!(stream.total_in_supply, Uint128::new(836_544_788));
        let position1 = query_position(deps.as_ref(), env, 1, "creator1".to_string()).unwrap();
        assert_eq!(position1.current_stage, Decimal::one());
        assert_eq!(
            position1.index,
            Decimal::from_str("969.437241956774606129").unwrap()
        );
        assert_eq!(position1.spent, Uint128::new(812_481_250));
        assert_eq!(position1.in_balance, Uint128::new(187_518_750));

        // trigger purchase after stream ends
        let mut env = mock_env();
        env.block.time = end.plus_seconds(1);
        let info = mock_info("creator2", &[]);
        execute_trigger_purchase(deps.as_mut(), env.clone(), info, 1).unwrap();
        let stream = query_stream(deps.as_ref(), env.clone(), 1).unwrap();
        assert_eq!(stream.current_stage, Decimal::one());
        println!("{}", stream.dist_index.to_string());
        assert_eq!(
            stream.dist_index,
            Decimal::from_str("969.437241956774606129").unwrap()
        );
        assert_eq!(stream.total_in_supply, Uint128::new(836_544_788));
        let position2 = query_position(deps.as_ref(), env, 1, "creator2".to_string()).unwrap();
        assert_eq!(position2.current_stage, Decimal::one());
        assert_eq!(
            position2.index,
            Decimal::from_str("969.437241956774606129").unwrap()
        );
        assert_eq!(position2.spent, Uint128::new(2_648_085_937));
        assert_eq!(position2.in_balance, Uint128::new(351_914_063));

        // TODO: 999975000000 - 999974999998 = 2
        // so always around 1-2 tokens
        assert_eq!(stream.total_out_sold, Uint128::new(999_975_000_000));
        println!("{}", position1.purchased);
        println!("{}", position2.purchased);
        println!("spent1 {}", position1.spent);
        assert_eq!(position1.purchased.checked_add(position2.purchased).unwrap(), Uint128::new(591_161_393_576))
    }

    #[test]
    fn test_withdraw() {
        let treasury = Addr::unchecked("treasury");
        let start = Timestamp::from_seconds(1_000_000);
        let end = Timestamp::from_seconds(5_000_000);
        let out_supply = Uint128::new(1_000_000_000_000);
        let out_denom = "out_denom";

        // instantiate
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(0);
        let msg = crate::msg::InstantiateMsg {
            min_stream_duration: Uint64::new(1000),
            min_duration_until_start_time: Uint64::new(0),
            stream_creation_denom: "fee".to_string(),
            stream_creation_fee: Uint128::new(100),
            fee_collector: "collector".to_string(),
        };
        instantiate(deps.as_mut(), mock_env(), mock_info("creator", &[]), msg).unwrap();

        // create stream
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(0);
        let info = mock_info(
            "creator1",
            &[
                Coin::new(out_supply.u128(), out_denom),
                Coin::new(100, "fee"),
            ],
        );
        execute_create_stream(
            deps.as_mut(),
            env,
            info,
            treasury.to_string(),
            "test".to_string(),
            "test".to_string(),
            "in".to_string(),
            out_denom.to_string(),
            out_supply,
            start,
            end,
        )
        .unwrap();

        // first subscription
        let mut env = mock_env();
        env.block.time = start.plus_seconds(0);
        let funds = Coin::new(2_000_000_000_000, "in");
        let info = mock_info("creator1", &[funds.clone()]);
        execute_subscribe(deps.as_mut(), env, info, 1).unwrap();

        // withdraw with cap
        let mut env = mock_env();
        env.block.time = start.plus_seconds(5000);
        let info = mock_info("creator1", &[]);
        let cap = Uint128::new(25_000_000);
        execute_withdraw(deps.as_mut(), env, info, 1, None, Some(cap)).unwrap();
        let position =
            query_position(deps.as_ref(), mock_env(), 1, "creator1".to_string()).unwrap();
        assert_eq!(position.in_balance, Uint128::new(1_997_475_000_000));
        assert_eq!(position.spent, Uint128::new(2_500_000_000));
        assert_eq!(position.purchased, Uint128::new(1_249_999_999));
        // first fund amount should be equal to in_balance + spent + cap
        assert_eq!(position.in_balance + position.spent + cap, funds.amount);

        // withdraw with cap to recipient
        let mut env = mock_env();
        env.block.time = start.plus_seconds(250_000);
        let info = mock_info("creator1", &[]);
        let cap = Uint128::new(25_000_000);
        let res = execute_withdraw(
            deps.as_mut(),
            env,
            info,
            1,
            Some("random".to_string()),
            Some(cap),
        )
        .unwrap();
        let position =
            query_position(deps.as_ref(), mock_env(), 1, "creator1".to_string()).unwrap();
        assert_eq!(position.in_balance, Uint128::new(1_875_104_656_250));
        assert_eq!(position.spent, Uint128::new(124_845_343_750));
        assert_eq!(position.purchased, Uint128::new(62_499_999_998));
        let msg = res.messages.get(0).unwrap();
        assert_eq!(
            msg.msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "random".to_string(),
                amount: vec![Coin::new(25_000_000, "in")]
            })
        );

        // can't withdraw after stream ends
        let mut env = mock_env();
        env.block.time = end.plus_seconds(1);
        let info = mock_info("creator1", &[]);
        let res = execute_withdraw(deps.as_mut(), env, info, 1, None, None).unwrap_err();
        assert_eq!(res, ContractError::StreamEnded {});

        // withdraw without cap
        let mut env = mock_env();
        env.block.time = start.plus_seconds(1_000_000);
        let info = mock_info("creator1", &[]);
        let res = execute_withdraw(deps.as_mut(), env, info, 1, None, None).unwrap();
        let position =
            query_position(deps.as_ref(), mock_env(), 1, "creator1".to_string()).unwrap();
        assert_eq!(position.in_balance, Uint128::zero());
        assert_eq!(position.spent, Uint128::new(476_427_466_796));
        assert_eq!(position.purchased, Uint128::new(249_999_999_997));
        let msg = res.messages.get(0).unwrap();
        assert_eq!(
            msg.msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "creator1".to_string(),
                amount: vec![Coin::new(1_523_522_533_204, "in")]
            })
        );
    }

    #[test]
    fn test_finalize_stream() {
        let treasury = Addr::unchecked("treasury");
        let start = Timestamp::from_seconds(1_000_000);
        let end = Timestamp::from_seconds(5_000_000);
        let out_supply = Uint128::new(1_000_000_000_000);
        let out_denom = "out_denom";

        // instantiate
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(0);
        let msg = crate::msg::InstantiateMsg {
            min_stream_duration: Uint64::new(1000),
            min_duration_until_start_time: Uint64::new(0),
            stream_creation_denom: "fee".to_string(),
            stream_creation_fee: Uint128::new(100),
            fee_collector: "collector".to_string(),
        };
        instantiate(deps.as_mut(), mock_env(), mock_info("creator", &[]), msg).unwrap();

        // create stream
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(0);
        let info = mock_info(
            "creator1",
            &[
                Coin::new(out_supply.u128(), out_denom),
                Coin::new(100, "fee"),
            ],
        );
        execute_create_stream(
            deps.as_mut(),
            env,
            info,
            treasury.to_string(),
            "test".to_string(),
            "test".to_string(),
            "in".to_string(),
            out_denom.to_string(),
            out_supply,
            start,
            end,
        )
        .unwrap();

        // first subscription
        let mut env = mock_env();
        env.block.time = start.plus_seconds(1_000_000);
        let funds = Coin::new(2_000_000_000_000, "in");
        let info = mock_info("creator1", &[funds.clone()]);
        execute_subscribe(deps.as_mut(), env, info, 1).unwrap();

        // only treasury can finalize
        let mut env = mock_env();
        env.block.time = end.plus_seconds(1);
        let info = mock_info("random", &[]);
        let res = execute_finalize_stream(deps.as_mut(), env, info, 1, None).unwrap_err();
        assert_eq!(res, ContractError::Unauthorized {});

        // can't finalize before stream ends
        let mut env = mock_env();
        env.block.time = start.plus_seconds(1);
        let info = mock_info(treasury.as_str(), &[]);
        let res = execute_finalize_stream(deps.as_mut(), env, info, 1, None).unwrap_err();
        assert_eq!(res, ContractError::StreamNotEnded {});

        // can't finalize without update distribution
        let mut env = mock_env();
        env.block.time = end.plus_seconds(1);
        let info = mock_info(treasury.as_str(), &[]);
        let res = execute_finalize_stream(deps.as_mut(), env, info, 1, None).unwrap_err();
        assert_eq!(res, ContractError::UpdateDistIndex {});

        // happy path
        let mut env = mock_env();
        env.block.time = end.plus_seconds(1);
        let info = mock_info(treasury.as_str(), &[]);
        execute_update_dist_index(deps.as_mut(), env.clone(), 1).unwrap();

        let res = execute_finalize_stream(deps.as_mut(), env, info, 1, None).unwrap();
        let fee_msg = res.messages.get(0).unwrap();
        assert_eq!(
            fee_msg.msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "collector".to_string(),
                amount: vec![Coin::new(100, "fee")]
            })
        );

        let leftover_msg = res.messages.get(1).unwrap();
        assert_eq!(
            leftover_msg.msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: treasury.to_string(),
                amount: vec![Coin::new(1_500_000_000_000, "in")]
            })
        );
        let send_msg = res.messages.get(2).unwrap();
        assert_eq!(
            send_msg.msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: treasury.to_string(),
                amount: vec![Coin::new(250_000_000_000, "out_denom")]
            })
        );
    }

    #[test]
    fn test_exit_stream() {
        let treasury = Addr::unchecked("treasury");
        let start = Timestamp::from_seconds(1_000_000);
        let end = Timestamp::from_seconds(5_000_000);
        let out_supply = Uint128::new(1_000_000_000_000);
        let out_denom = "out_denom";

        // instantiate
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(0);
        let msg = crate::msg::InstantiateMsg {
            min_stream_duration: Uint64::new(1000),
            min_duration_until_start_time: Uint64::new(0),
            stream_creation_denom: "fee".to_string(),
            stream_creation_fee: Uint128::new(100),
            fee_collector: "collector".to_string(),
        };
        instantiate(deps.as_mut(), mock_env(), mock_info("creator", &[]), msg).unwrap();

        // create stream
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(0);
        let info = mock_info(
            "creator1",
            &[
                Coin::new(out_supply.u128(), out_denom),
                Coin::new(100, "fee"),
            ],
        );
        execute_create_stream(
            deps.as_mut(),
            env,
            info,
            treasury.to_string(),
            "test".to_string(),
            "test".to_string(),
            "in".to_string(),
            out_denom.to_string(),
            out_supply,
            start,
            end,
        )
        .unwrap();

        // first subscription
        let mut env = mock_env();
        env.block.time = start.plus_seconds(1_000_000);
        let funds = Coin::new(2_000_000_000_000, "in");
        let info = mock_info("creator1", &[funds.clone()]);
        execute_subscribe(deps.as_mut(), env, info, 1).unwrap();

        // can't exit before stream ends
        let mut env = mock_env();
        env.block.time = start.plus_seconds(2_000_000);
        let info = mock_info("creator1", &[]);
        let res = execute_exit_stream(deps.as_mut(), env, info, 1, None).unwrap_err();
        assert_eq!(res, ContractError::StreamNotEnded {});

        // can't exit without update distribution
        let mut env = mock_env();
        env.block.time = end.plus_seconds(2_000_000);
        let info = mock_info("creator1", &[]);
        let res = execute_exit_stream(deps.as_mut(), env, info, 1, None).unwrap_err();
        assert_eq!(res, ContractError::UpdateDistIndex {});

        // update dist
        let mut env = mock_env();
        env.block.time = end.plus_seconds(2_000_000);
        execute_update_dist_index(deps.as_mut(), env.clone(), 1).unwrap();

        // can exit
        let mut env = mock_env();
        env.block.time = end.plus_seconds(3_000_000);
        let info = mock_info("creator1", &[]);
        let res = execute_exit_stream(deps.as_mut(), env, info, 1, None).unwrap();

        let send_msg = res.messages.get(0).unwrap();
        assert_eq!(
            send_msg.msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "creator1".to_string(),
                amount: vec![Coin::new(750_000_000_000, "out_denom")]
            })
        );

        let leftover_msg = res.messages.get(1).unwrap();
        assert_eq!(
            leftover_msg.msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "creator1".to_string(),
                amount: vec![Coin::new(500_000_000_000, "in")]
            })
        );

        // can't exit twice
        let mut env = mock_env();
        env.block.time = end.plus_seconds(4_000_000);
        let info = mock_info("creator1", &[]);
        let res = execute_exit_stream(deps.as_mut(), env, info, 1, None).unwrap_err();
        assert_eq!(res, ContractError::PositionAlreadyExited {});
    }
}
