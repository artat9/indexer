use candid::Deserialize;
use candid::Principal;
use ic_cdk::api::call::{self, CallResult};
use ic_cdk::query;
use ic_cdk::update;
use std::{collections::HashMap, str::FromStr};
#[derive(candid::CandidType, Debug, Clone, Deserialize)]
pub struct Event {
    //from: Address,
    pub recipient: String,
    pub hash: String,
    pub at: u64,
    pub block_number: u64,
    pub params: HashMap<String, String>,
}

#[update]
async fn greet(block_num: u64) {
    let target = Principal::from_str("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();

    let result: CallResult<(Vec<Event>,)> =
        call::call(target, "getEventsByBlockNumber", (block_num,)).await;
    match result {
        Ok((events,)) => {
            ic_cdk::println!("events: {:?}", events);
        }
        Err(e) => {
            ic_cdk::println!("error: {:?}", e);
        }
    }
}
