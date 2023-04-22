use candid::CandidType;
use candid::Deserialize;
use candid::Nat;
use candid::Principal;
use common::types::TransferEvent;
use ic_cdk::api::call::RejectionCode;
use ic_cdk::api::call::{self, CallResult};
use ic_cdk::query;
use ic_cdk::update;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::str::FromStr;

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

type Balance = Nat;

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

#[update]
async fn subscribe_transfer_event(canister_id: String) {
    let result: Result<(), (RejectionCode, String)> =
        ic_cdk::api::call::call(Principal::from_text(canister_id).unwrap(), "subscribe", ()).await;
    match result {
        Ok(_) => {}
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

#[update]
fn subscribe(subscriber: Subscriber) -> bool {
    let subscriber_principal_id = ic_cdk::caller();
    SUBSCRIBERS.with(|f| {
        let mut store = f.borrow_mut();
        if !store.contains_key(&subscriber_principal_id) {
            store.insert(subscriber_principal_id, subscriber);
        }
    });
    true
}

async fn pub_task(
    principal: Principal,
    event: BalanceUpdateEvent,
) -> Result<(), (RejectionCode, String)> {
    let res: CallResult<(BalanceUpdateEvent,)> =
        call::call(principal, "on_update", (&event,)).await;
    match res {
        Ok(_) => Ok(()),
        Err((c, s)) => Err((c, s)),
    }
}

async fn publish(events: Vec<BalanceUpdateEvent>) -> bool {
    let mut tasks = Vec::new();
    SUBSCRIBERS.with(|f| {
        for (k, v) in f.borrow().iter() {
            for event in events.clone() {
                let future = pub_task(k.to_owned(), event);
                tasks.push(future)
            }
        }
    });
    futures::future::join_all(tasks).await;
    true
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
