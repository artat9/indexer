use candid::candid_method;
use ic_cdk::api::management_canister::http_request::{HttpResponse, TransformArgs};
use ic_cdk_macros::{query, update};
use ic_web3::types::Address;
use log_finder::{contract, http_client, LogFinder};
mod log_finder;
use std::str::FromStr;
use std::vec;

pub struct TransferEvent {
    //from: Address,
    recipient: Address,
    amount: u128,
    at: u64,
}

static mut EVENTS: Vec<TransferEvent> = vec![];

#[query]
fn event_amount(idx: usize) -> u128 {
    unsafe { EVENTS[usize::try_from(idx).unwrap()].amount }
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

#[update(name = "testing")]
#[candid_method(update)]
async fn testing() -> Result<String, String> {
    save_logs(17022971, 17022973).await
}

const ERC20_ABI: &[u8] = include_bytes!("./abis/erc20.abi");
const EVENT: &str = "Transfer";
const TOKEN_ADDRESS: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"; // WETH

async fn save_logs(from: u64, to: u64) -> Result<String, String> {
    let web3 = http_client()?;
    let contract = contract(web3, Address::from_str(TOKEN_ADDRESS).unwrap(), ERC20_ABI)?;
    let results = LogFinder::new(http_client().unwrap(), contract, EVENT)
        .find(from, to)
        .await?;
    results.into_iter().for_each(|result| {
        let event = TransferEvent {
            amount: result.event.params[2]
                .clone()
                .value
                .into_uint()
                .unwrap()
                .as_u128(),
            at: result.log.block_number.unwrap().as_u64(),
            recipient: result.event.params[1].clone().value.into_address().unwrap(),
        };
        unsafe {
            EVENTS.push(event);
        }
    });
    Ok("ok".to_string())
}
