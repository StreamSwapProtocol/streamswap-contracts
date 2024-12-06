use std::collections::{BTreeMap};
use cosmwasm_std::{Addr, BankMsg, Coin, CosmosMsg};

/// Payments is a simple wrapper around a map of address to a map of denom to amount
/// Prevents the case where multiple same denom coins are sent to the same address
pub struct Payments {
    // address -> denom -> amount
    payments: BTreeMap<String, BTreeMap<String, Coin>>,
}

impl Payments {
    pub fn to_cosmos_msgs(self) -> Vec<CosmosMsg> {
        self.payments
            .into_iter()
            .map(|(addr, coins)| {
                let coins: Vec<Coin> = coins.into_iter().map(|(_, coin)| coin).collect();
                CosmosMsg::Bank(BankMsg::Send {
                    to_address: addr,
                    amount: coins,
                })
            }).collect()

    }

    pub fn add_payment(&mut self, address: Addr, coin: Coin) {
        let coins = self.payments.entry(address.to_string()).or_insert(BTreeMap::new());
        coins.entry(coin.denom.clone())
            .and_modify(|c| c.amount += coin.amount)
            .or_insert(Coin::new(coin.amount.u128(), coin.denom.clone()));
    }

    pub fn add_payments(&mut self, address: Addr, coins: Vec<Coin>) {
        coins.into_iter().for_each(|coin| {
            self.add_payment(address.clone(), coin);
        });
    }

    pub fn new() -> Self {
        Payments {
            payments: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_payment() {
        let mut payments = Payments::new();
        let sender = Addr::unchecked("addr1");
        let sender2 = Addr::unchecked("addr2");
        payments.add_payment(sender.clone(), Coin::new(100, "ucosm"));
        payments.add_payment(sender.clone(), Coin::new(200, "ucosm"));
        payments.add_payment(sender.clone(), Coin::new(300, "ustake"));
        payments.add_payment(sender.clone(), Coin::new(400, "ustake"));
        payments.add_payment(sender.clone(), Coin::new(1000, "utoken"));
        payments.add_payment(sender2.clone(), Coin::new(3000, "utoken"));
        payments.add_payment(sender2.clone(), Coin::new(4000, "utoken"));

        let msgs = payments.to_cosmos_msgs();
        if let CosmosMsg::Bank(BankMsg::Send { to_address, amount }) = &msgs[0] {
            assert_eq!(to_address, sender.as_str());
            assert_eq!(amount.len(), 3);
            assert_eq!(amount[0], Coin::new(300, "ucosm"));
            assert_eq!(amount[1], Coin::new(700, "ustake"));
            assert_eq!(amount[2], Coin::new(1000, "utoken"));
        } else {
            panic!("unexpected message type");
        }

        if let CosmosMsg::Bank(BankMsg::Send { to_address, amount }) = &msgs[1] {
            assert_eq!(to_address, sender2.as_str());
            assert_eq!(amount.len(), 1);
            assert_eq!(amount[0], Coin::new(7000, "utoken"));
        } else {
            panic!("unexpected message type");
        }
    }

    #[test]
fn test_add_payments() {
    let mut payments = Payments::new();
    let sender = Addr::unchecked("addr1");
    let coins = vec![
        Coin::new(100, "ucosm"),
        Coin::new(400, "ucosm"),
        Coin::new(200, "ustake"),
        Coin::new(500, "ustake"),
        Coin::new(300, "utoken"),
    ];

        payments.add_payments(sender.clone(), coins);
        let sender2 = Addr::unchecked("addr2");
        let coins2 = vec![
            Coin::new(100, "ucosm"),
            Coin::new(400, "ucosm"),
            Coin::new(200, "ustake"),
            Coin::new(500, "ustake"),
            Coin::new(300, "utoken"),
        ];

        payments.add_payments(sender2.clone(), coins2);

    let msgs = payments.to_cosmos_msgs();
    if let CosmosMsg::Bank(BankMsg::Send { to_address, amount }) = &msgs[0] {
        assert_eq!(to_address, sender.as_str());
        assert_eq!(amount.len(), 3);
        assert_eq!(amount[0], Coin::new(500, "ucosm"));
        assert_eq!(amount[1], Coin::new(700, "ustake"));
        assert_eq!(amount[2], Coin::new(300, "utoken"));
    } else {
        panic!("unexpected message type");
    }

        if let CosmosMsg::Bank(BankMsg::Send { to_address, amount }) = &msgs[1] {
            assert_eq!(to_address, sender2.as_str());
            assert_eq!(amount.len(), 3);
            assert_eq!(amount[0], Coin::new(500, "ucosm"));
            assert_eq!(amount[1], Coin::new(700, "ustake"));
            assert_eq!(amount[2], Coin::new(300, "utoken"));
        } else {
            panic!("unexpected message type");
        }
}
}
