use candid::CandidType;
use candid::Deserialize;
use candid::Nat;
use candid::Principal;
use common::types::Balance;
use common::types::TransferEvent;
use ic_cdk::api::call::CallResult;
use ic_cdk::query;
use ic_cdk::update;
use std::cell::RefCell;
use std::collections::BTreeMap;

#[derive(candid::CandidType, Debug, Clone, Deserialize)]
pub struct BalanceUpdateEvent {
    pub block_number: u64,
    pub account: String,
    pub balance_after: Balance,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct Subscriber {
    topic: String,
}

thread_local! {
    static ACCOUNT_BALANCES: RefCell<BTreeMap<String, Balance>> = RefCell::new(BTreeMap::new());
    static SUBSCRIBERS: RefCell<BTreeMap<Principal,Subscriber>> = RefCell::new(BTreeMap::new());
}

#[query]
fn get_account_balance(account: String) -> Balance {
    ACCOUNT_BALANCES.with(|f| {
        let balances: std::cell::Ref<BTreeMap<String, Balance>> = f.borrow();
        match balances.get(&account) {
            Some(balance) => balance.clone(),
            None => Nat::from(0),
        }
    })
}

#[query]
fn get_account_balances() -> BTreeMap<String, Balance> {
    ACCOUNT_BALANCES.with(|f| f.borrow().to_owned())
}

#[query]
fn balances_top_n(n: u64) -> Vec<(String, Balance)> {
    let mut balances_vec: Vec<(String, Balance)> = ACCOUNT_BALANCES
        .with(|f| f.borrow().to_owned())
        .into_iter()
        .collect();
    balances_vec.sort_by(|a, b| b.1.cmp(&a.1));
    balances_vec.truncate(n as usize);
    balances_vec
}

#[update]
async fn subscribe_transfer_event(canister_id: String) {
    let result: CallResult<()> = ic_cdk::api::call::call(
        Principal::from_text(canister_id.clone()).unwrap(),
        "subscribe",
        (),
    )
    .await;
    match result {
        Ok(_) => {
            ic_cdk::println!("subscriptioin ok");
        }
        Err(e) => {
            ic_cdk::println!("error calling subscriber: {:?}", e);
        }
    }
}

#[update]
async fn on_update(eventes: Vec<TransferEvent>) {
    eventes.iter().for_each(|event| {
        update_balance(event.clone());
    });
}

fn update_balance(event: TransferEvent) {
    ACCOUNT_BALANCES.with(|f| {
        let mut balances: std::cell::RefMut<BTreeMap<String, Balance>> = f.borrow_mut();
        let from = event.from.clone();
        let to = event.to.clone();
        ic_cdk::println!("saving transfer from: {:?}, to: {:?}", from.clone(), to);
        let value = event.value;
        let from_balance = balances.entry(from).or_insert(Nat::from(0));
        if from_balance.to_owned().ge(&value) {
            *from_balance -= value.clone();
        }
        let to_balance = balances.entry(to).or_insert(Nat::from(0));
        *to_balance += value.clone();
    });
}
