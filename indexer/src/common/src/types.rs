use candid::{Deserialize, Nat};

#[derive(candid::CandidType, Debug, Clone, Deserialize)]
pub struct TransferEvent {
    //from: Address,
    pub hash: String,
    pub at: u64,
    pub block_number: u64,
    pub from: String,
    pub to: String,
    pub value: Nat,
}
