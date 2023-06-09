use std::ops::{Add, Div, Mul};

use candid::{Nat, Principal};
use common::types::Balance;
use ic_cdk::{
    api::call::{call, CallResult, RejectionCode},
    query, update,
};

const BASE: u128 = 1_000_000_000_000_u128;

async fn top_n(mapping: Principal, n: u64) -> Result<Vec<(String, Balance)>, RejectionCode> {
    let result: CallResult<(Vec<(String, Balance)>,)> =
        call(mapping, "balances_top_n", (&n,)).await;
    match result {
        Ok(result) => Ok(result.0),
        Err(e) => {
            ic_cdk::println!("error calling subscriber: {:?}", e);
            Err(e.0)
        }
    }
}

fn total_of(balances: Vec<Balance>) -> Balance {
    balances
        .iter()
        .fold(Balance::from(0), |acc, balance| acc.add(balance.clone()))
}

#[update]
async fn hhi_of_top_n(mapper: String, n: u64) -> Nat {
    let balances = top_n(Principal::from_text(mapper).unwrap(), n).await;
    let amounts: Vec<Nat> = balances.unwrap().iter().map(|v| v.clone().1).collect();
    let total_amount = total_of(amounts.clone());
    amounts
        .iter()
        .map(|balance| hhi(balance.clone(), total_amount.clone()))
        .fold(Balance::from(0), |acc, balance| acc.add(balance))
        .mul(Balance::from(100))
        .mul(Balance::from(100))
        .div(BASE)
        .div(BASE)
}

fn hhi(balance: Balance, total_amount: Balance) -> Balance {
    let dominance = balance.mul(BASE).div(total_amount);
    dominance.clone().mul(dominance)
}
