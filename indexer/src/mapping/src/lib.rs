use candid::CandidType;
use candid::Deserialize;
use candid::Nat;
use candid::Principal;
use ic_cdk::api::call::{self, CallResult};
use ic_cdk::query;
use ic_cdk::update;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::str::FromStr;

#[derive(candid::CandidType, Debug, Clone, Deserialize)]
pub struct Event {
    //from: Address,
    pub recipient: String,
    pub hash: String,
    pub at: u64,
    pub block_number: u64,
    pub from: String,
    pub to: String,
    pub value: Nat,
}
type Balance = Nat;
type SubscriberStore = BTreeMap<Principal, Subscriber>;

thread_local! {
    static  SAVED_BLOCK:RefCell<u64> = RefCell::new(17078925);
    static ACCOUNT_BALANCES: RefCell<BTreeMap<String, Balance>> = RefCell::new(BTreeMap::new());
    static SUBSCRIBERS: RefCell<Vec<String>> = RefCell::new(Vec::new());
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

#[derive(Clone, Debug, CandidType, Deserialize)]
struct Subscriber {
    topic: String,
}

#[ic_cdk_macros::post_upgrade]
fn post_upgrade() {
    init()
}
#[ic_cdk_macros::init]
fn init() {
    let interval = std::time::Duration::from_secs(5);
    ic_cdk_timers::set_timer_interval(interval, || {
        ic_cdk::spawn(async {
            let result = udpate_balances().await;
            match result {
                Ok(_) => {}
                Err(e) => {}
            }
        })
    });
}

async fn udpate_balances() -> Result<String, String> {
    let saved = SAVED_BLOCK.with(|f| f.borrow().to_owned());
    ic_cdk::println!("saved block: {:?}", saved);
    let next = saved + 1;
    update_account_balance(saved).await?;
    SAVED_BLOCK.with(|f| f.replace(next));
    Ok(("").to_string())
}

async fn update_account_balance(block_num: u64) -> Result<String, String> {
    let target = Principal::from_str("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();

    let result: CallResult<(Vec<Event>,)> =
        call::call(target, "getEventsByBlockNumber", (block_num,)).await;
    match result {
        Ok((events,)) => {
            events.into_iter().for_each(|event| {
                update_balance(event);
            });
            Ok(("").to_string())
        }
        Err((_, s)) => {
            ic_cdk::println!("error: {:?}", s);
            Err(s)
        }
    }
}

fn update_balance(event: Event) {
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
