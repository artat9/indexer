use candid::candid_method;
use ic_cdk::api::management_canister::http_request::{HttpResponse, TransformArgs};
use ic_cdk_macros::{query, update};
use ic_web3::ethabi::Topic;
use ic_web3::ethabi::{RawLog, TopicFilter};
use ic_web3::transports::ICHttp;
use ic_web3::types::FilterBuilder;
use ic_web3::Web3;
use ic_web3::{
    contract::Contract,
    types::{Address, BlockNumber},
};

use std::str::FromStr;
use std::vec;

pub struct TransferEvent {
    //from: Address,
    recipient: Address,
    amount: u128,
    at: u64,
}

pub struct Block {
    events: [TransferEvent],
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
    subscribe(
        BlockNumber::Number(17022971.into()),
        BlockNumber::Number(17022973.into()),
    )
    .await
}

const ERC20_ABI: &[u8] = include_bytes!("./abis/erc20.abi");
const EVENT: &str = "Transfer";
const WS_ENDPOINT: &str = "https://mainnet.infura.io/v3/TEST";
const TOKEN_ADDRESS: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"; // WETH

async fn subscribe(from: BlockNumber, to: BlockNumber) -> Result<String, String> {
    let web3 = match ICHttp::new(WS_ENDPOINT, None) {
        Ok(v) => Web3::new(v),
        Err(e) => return Err(format!("init web3 failed:{}", e)),
    };
    let contract = Contract::from_json(
        web3.eth(),
        Address::from_str(TOKEN_ADDRESS).unwrap(),
        ERC20_ABI,
    )
    .map_err(|e| format!("init contract failed:{}", e))?;
    let event_sig = contract.abi().event(EVENT).unwrap().signature();
    let log_future = web3.eth_filter().create_logs_filter(
        FilterBuilder::default()
            .from_block(from)
            .to_block(to)
            .address(vec![Address::from_str(TOKEN_ADDRESS).unwrap()])
            .topic_filter(TopicFilter {
                topic0: Topic::This(event_sig),
                topic1: Topic::Any,
                topic2: Topic::Any,
                topic3: Topic::Any,
            })
            .build(),
    );
    let log_result = log_future
        .await
        .map_err(|e| format!("create log filter failed:{}", e.to_string()))?;
    log_result.logs().await.iter().for_each(|logs| {
        logs.iter().for_each(|log| {
            let result = contract
                .abi()
                .event(EVENT)
                .unwrap()
                .parse_log(RawLog {
                    data: log.data.0.clone(),
                    topics: log.topics.clone(),
                })
                .unwrap();
            let event = TransferEvent {
                amount: result.params[2]
                    .clone()
                    .value
                    .into_uint()
                    .unwrap()
                    .as_u128(),
                at: log.block_number.unwrap().as_u64(),
                recipient: result.params[1].clone().value.into_address().unwrap(),
            };
            unsafe {
                EVENTS.push(event);
            }
        })
    });
    Ok("ok".to_string())
}
