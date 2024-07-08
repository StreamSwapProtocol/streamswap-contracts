use std::str::FromStr;

use cosmwasm_std::{Coin, Uint128};
use cw_multi_test::AppResponse;

#[allow(dead_code)]

pub fn get_contract_address_from_res(res: AppResponse) -> String {
    res.events
        .iter()
        .find(|e| e.ty == "instantiate")
        .unwrap()
        .attributes
        .iter()
        .find(|a| a.key == "_contract_address")
        .unwrap()
        .value
        .clone()
}

#[allow(dead_code)]
pub fn get_funds_from_res(res: AppResponse) -> Vec<(String, Coin)> {
    let mut funds = Vec::new();

    for event in res.events.iter() {
        if event.ty == "transfer" {
            let recipient = event
                .attributes
                .iter()
                .find(|a| a.key == "recipient")
                .map(|a| a.value.clone());

            let amount = event
                .attributes
                .iter()
                .find(|a| a.key == "amount")
                .map(|a| a.value.clone());

            if let (Some(recipient), Some(amount)) = (recipient, amount) {
                let (amount_value, denom) =
                    amount.chars().partition::<String, _>(|c| c.is_numeric());

                if let Ok(parsed_amount) = Uint128::from_str(&amount_value) {
                    let coin = Coin {
                        amount: parsed_amount,
                        denom,
                    };
                    funds.push((recipient, coin));
                }
            }
        }
    }
    funds
}

#[cfg(test)]
pub fn get_wasm_attribute_with_key(res: AppResponse, key: String) -> String {
    if let Some(_non_empty_key) = key.chars().next() {
        res.events
            .iter()
            .find(|e| e.ty == "wasm")
            .and_then(|event| event.attributes.iter().find(|a| a.key == key))
            .map(|attr| attr.value.clone())
            .unwrap_or_else(|| "".to_string())
    } else {
        "".to_string()
    }
}
