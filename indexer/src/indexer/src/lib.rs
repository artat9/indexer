use candid::CandidType;
use candid::Deserialize;
use ic_cdk::api::management_canister::http_request::{HttpResponse, TransformArgs};
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

#[derive(CandidType, Clone, Deserialize, Debug)]
pub struct Event {
    //from: Address,
    pub recipient: String,
    pub hash: String,
    pub at: u64,
    pub block_number: u64,
    pub params: HashMap<String, String>,
}
thread_local! {
    static  SAVED_BLOCK:RefCell<u64> = RefCell::new(17078925);
    static  EVENTS_MAP: RefCell<BTreeMap<u64, Vec<Event>>> = RefCell::new(BTreeMap::new());
}

#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    export_service!();
    __export_service()
}

#[query]
fn latest_block_number() -> u64 {
    SAVED_BLOCK.with(|f| f.borrow().to_owned())
}

#[query(name = "getEventsByBlockNumber")]
fn get_events_by_block_number(block_number: u64) -> Vec<Event> {
    EVENTS_MAP
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
fn post_upgrade() {
    init()
}
#[ic_cdk::update]
fn update() {
    init()
}

#[ic_cdk_macros::init]
fn init() {
    let interval = std::time::Duration::from_secs(10);
    ic_cdk_timers::set_timer_interval(interval, || {
        ic_cdk::spawn(async {
            let result = save_logs().await;
            match result {
                Ok(_) => {}
                Err(e) => {}
            }
        })
    });
}

const ERC20_ABI: &[u8] = include_bytes!("./abis/erc20.abi");
const EVENT: &str = "Transfer";
const TOKEN_ADDRESS: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"; // WETH
async fn save_logs() -> Result<String, String> {
    let saved = SAVED_BLOCK.with(|f| f.borrow().to_owned());
    let latest = get_latest_block_number().await?;
    if saved.ge(&latest) {
        return Ok("".to_string());
    }
    let next = saved.add(5);
    let result: Result<String, String> = save_logs_from_to(saved, next).await;
    SAVED_BLOCK.with(|f| f.replace(next));
    result
}

async fn get_latest_block_number() -> Result<u64, String> {
    let web3: ic_web3::Web3<ic_web3::transports::ICHttp> = http_client()?;
    let number = web3.eth().block_number().await.unwrap();
    Ok(number.as_u64())
}

async fn save_logs_from_to(from: u64, to: u64) -> Result<String, String> {
    let web3: ic_web3::Web3<ic_web3::transports::ICHttp> = http_client()?;
    let contract = contract(web3, Address::from_str(TOKEN_ADDRESS).unwrap(), ERC20_ABI)?;
    let results = LogFinder::new(http_client().unwrap(), contract, EVENT)
        .find(from, to)
        .await?;
    results.into_iter().for_each(|result| {
        let event: Event = Event {
            at: result.log.block_number.unwrap().as_u64(),
            recipient: result.event.params[1].clone().value.to_string(),
            block_number: result.log.block_number.unwrap().as_u64(),
            hash: result.log.transaction_hash.unwrap().to_string(),
            params: result
                .event
                .params
                .iter()
                .map(|param| (param.name.clone(), param.value.to_string()))
                .collect(),
        };
        ic_cdk::println!("{:?}", event.block_number);
        ic_cdk::println!("{:?}", event);
        // insert event into EVENTS_MAP
        EVENTS_MAP.with(|e| {
            e.borrow_mut()
                .entry(event.block_number)
                .or_insert(Vec::new())
                .push(event);
        });
    });
    Ok("ok".to_string())
}
