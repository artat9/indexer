use candid::CandidType;
use candid::Deserialize;
use candid::Nat;
use candid::Principal;
use common::abi::ERC20_ABI;
use common::indexing::IndexingConfig;
use common::types::TransferEvent;
use ic_cdk::api::call::CallResult;
use ic_cdk::api::management_canister::http_request::{HttpResponse, TransformArgs};
use ic_cdk::update;
use ic_cdk_macros::query;
use ic_web3::types::Address;
use log_finder::{contract, http_client, LogFinder};
mod log_finder;
use ic_cdk::export::candid::{candid_method, export_service};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::ops::Add;
use std::str::FromStr;
const EVENT: &str = "Transfer";

thread_local! {
    static  SAVED_BLOCK:RefCell<u64> = RefCell::default();
    static  EVENTS_STORE: RefCell<BTreeMap<u64, Vec<TransferEvent>>> = RefCell::new(BTreeMap::new());
    static PROFILE_STORE: RefCell<Profile> = RefCell::new(Profile{config : IndexingConfig::new_dai_mainnet()});
    static SUBSCRIBER_STORE :RefCell<Vec<Principal>> = RefCell::default();
}

#[derive(Clone, Debug, CandidType, Default, Deserialize)]
struct Profile {
    config: IndexingConfig,
}

#[query]
fn subscribers() -> Vec<Principal> {
    SUBSCRIBER_STORE.with(|f| f.borrow().to_owned())
}

#[update]
fn subscribe() {
    SUBSCRIBER_STORE.with(|f| f.borrow_mut().push(ic_cdk::caller()))
}

async fn publish(events: Vec<TransferEvent>) -> bool {
    let store = SUBSCRIBER_STORE.with(|f| f.borrow().to_owned());
    for subscriber in store {
        let result: CallResult<()> =
            ic_cdk::api::call::call(subscriber, "on_update", (&events,)).await;

        match result {
            Ok(_) => {
                ic_cdk::println!("[indexer] called subscribe");
            }
            Err(e) => {
                ic_cdk::println!("error calling subscriber: {:?}", e);
            }
        }
    }
    true
}

#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    export_service!();
    __export_service()
}

#[query]
fn block_number_at_deploy() -> u64 {
    ic_cdk::println!("called block_number_at_deploy");
    PROFILE_STORE
        .with(|f| f.borrow().to_owned())
        .config
        .batch_sync_start_from
}

#[query]
fn latest_block_number() -> u64 {
    SAVED_BLOCK.with(|f| f.borrow().to_owned())
}

#[query(name = "getEventsByBlockNumber")]
fn get_events_by_block_number(block_number: u64) -> Vec<TransferEvent> {
    EVENTS_STORE
        .with(|f| {
            let map = f.borrow();
            match map.get(&block_number) {
                Some(events) => events.to_owned(),
                None => Vec::new(),
            }
        })
        .to_owned()
        .to_vec()
}
#[update]
async fn update_events(events: HashMap<u64, Vec<TransferEvent>>) {
    ic_cdk::println!("update events invoked: blocks: {}", events.len());
    let mut store = EVENTS_STORE.with(|f| f.borrow_mut().to_owned());
    let latest_saved_block = SAVED_BLOCK.with(|f| f.borrow().to_owned());
    events
        .iter()
        .filter(|(k, _)| **k > latest_saved_block)
        .for_each(|(k, v)| {
            store.insert(*k, v.to_owned());
        });
    events.keys().max().map(|max| {
        SAVED_BLOCK.with(|f| {
            let mut block = f.borrow_mut();
            *block = *max;
        });
    });
    publish(events.values().flatten().cloned().collect()).await;
}

#[query(name = "transform")]
#[candid_method(query, rename = "transform")]
fn transform(response: TransformArgs) -> HttpResponse {
    let res = response.response;
    // remove header
    HttpResponse {
        status: res.status,
        headers: Vec::default(),
        body: res.body,
    }
}
#[ic_cdk_macros::post_upgrade]
async fn post_upgrade() {
    init().await
}
#[ic_cdk::update]
async fn update() {
    init().await
}

fn sync_completed() -> bool {
    saved_block().ge(&block_number_at_deploy())
}

fn saved_block() -> u64 {
    SAVED_BLOCK.with(|f| f.borrow().to_owned())
}

fn setup() {
    let profile = profile();
    SAVED_BLOCK.with(|f| f.replace(profile.config.deployed_block()));
}

#[ic_cdk_macros::init]
async fn init() {
    setup();
    let interval = std::time::Duration::from_secs(1 * 60 * 60);
    ic_cdk_timers::set_timer_interval(interval, || {
        if !sync_completed() {
            ic_cdk::spawn(async {
                let result = save_logs().await;
                match result {
                    Ok(_) => {}
                    Err(e) => {}
                }
            })
        }
    });
}

async fn save_logs() -> Result<String, String> {
    let saved = saved_block();
    let latest = get_latest_block_number().await?;

    if saved.ge(&latest) {
        return Ok("".to_string());
    }
    if saved.lt(&block_number_at_deploy()) {
        return Ok("".to_string());
    }
    let next: u64 = saved.add(5);
    let result: Result<String, String> = save_logs_from_to(saved, next).await;
    SAVED_BLOCK.with(|f| f.replace(next));
    result
}

async fn get_latest_block_number() -> Result<u64, String> {
    let web3: ic_web3::Web3<ic_web3::transports::ICHttp> = http_client()?;
    let number = web3.eth().block_number().await.unwrap();
    Ok(number.as_u64())
}

fn profile() -> Profile {
    PROFILE_STORE.with(|f| f.borrow().to_owned())
}

async fn save_logs_from_to(from: u64, to: u64) -> Result<String, String> {
    let web3: ic_web3::Web3<ic_web3::transports::ICHttp> = http_client()?;
    let contract = contract(
        web3,
        Address::from_str(profile().config.address()).unwrap(),
        ERC20_ABI,
    )?;
    let results = LogFinder::new(http_client().unwrap(), contract, EVENT)
        .find(from, to)
        .await?;
    results.into_iter().for_each(|result| {
        let (mut from, mut to, mut value): (String, String, Nat) =
            ("".to_string(), "".to_string(), Nat::default());
        result
            .event
            .params
            .iter()
            .for_each(|param| match param.name.as_str() {
                "from" => from = "0x".to_owned() + param.value.to_string().as_str(),
                "to" => to = "0x".to_owned() + param.value.to_string().as_str(),
                "value" => {
                    value = Nat::from(param.clone().value.into_int().unwrap_or_default().as_u128())
                }
                _ => {}
            });
        let event: TransferEvent = TransferEvent {
            at: result.log.block_number.unwrap().as_u64(),
            block_number: result.log.block_number.unwrap().as_u64(),
            hash: result.log.transaction_hash.unwrap().to_string(),
            from,
            to,
            value,
        };
        ic_cdk::println!("{:?}", event.block_number);
        ic_cdk::println!("{:?}", event);
        // insert event into EVENTS_MAP
        EVENTS_STORE.with(|e| {
            e.borrow_mut()
                .entry(event.block_number)
                .or_insert(Vec::new())
                .push(event);
        });
    });

    Ok("ok".to_string())
}
