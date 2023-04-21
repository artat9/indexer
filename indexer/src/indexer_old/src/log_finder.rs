use ic_web3::ethabi::Log;
use ic_web3::ethabi::RawLog;
use ic_web3::ethabi::Topic;
use ic_web3::ethabi::TopicFilter;
use ic_web3::transports::ICHttp;
use ic_web3::types::BlockNumber;
use ic_web3::types::FilterBuilder;
use ic_web3::types::Log as EthLog;
use ic_web3::types::H256;
use ic_web3::Transport;
use ic_web3::Web3;
use ic_web3::{contract::Contract, types::Address};

const HTTP_ENDPOINT: &str = "https://mainnet.infura.io/v3/TEST";

pub struct LogFinder {
    web3: Web3<ICHttp>,
    contract: Contract<ICHttp>,
    event_sig: H256,
    event_name: String,
}

pub struct EventLog {
    pub event: Log,
    pub log: EthLog,
}

impl LogFinder {
    pub fn new(web3: Web3<ICHttp>, contract: Contract<ICHttp>, event_name: &str) -> Self {
        let event_sig = event_sig(&contract, event_name).unwrap();
        Self {
            web3,
            contract,
            event_sig,
            event_name: event_name.to_string(),
        }
    }
    pub async fn find(&self, from: u64, to: u64) -> Result<Vec<EventLog>, String> {
        assert!(from <= to);
        let filter = self
            .web3
            .eth_filter()
            .create_logs_filter(
                FilterBuilder::default()
                    .from_block(BlockNumber::Number(from.into()))
                    .to_block(BlockNumber::Number(to.into()))
                    .address(vec![self.contract.address()])
                    .topic_filter(TopicFilter {
                        topic0: Topic::This(self.event_sig),
                        topic1: Topic::Any,
                        topic2: Topic::Any,
                        topic3: Topic::Any,
                    })
                    .build(),
            )
            .await
            .map_err(|e| format!("create log filter failed:{}", e.to_string()))?;
        let parser = self.contract.abi().event(&self.event_name).unwrap();
        let logs = filter
            .logs()
            .await
            .unwrap()
            .into_iter()
            .map(|log| EventLog {
                event: parser
                    .parse_log(RawLog {
                        data: log.data.0.clone(),
                        topics: log.topics.clone(),
                    })
                    .unwrap(),
                log,
            })
            .collect::<Vec<_>>();
        Ok(logs)
    }
}

pub fn http_client() -> Result<Web3<ICHttp>, String> {
    match ICHttp::new(HTTP_ENDPOINT, None) {
        Ok(v) => Ok(Web3::new(v)),
        Err(e) => Err(format!("init web3 failed:{}", e)),
    }
}

pub fn contract<T: Transport>(
    client: Web3<T>,
    address: Address,
    abi: &[u8],
) -> Result<Contract<T>, String> {
    Contract::from_json(client.eth(), address, abi)
        .map_err(|e| format!("init contract failed:{}", e))
}

pub fn event_sig<T: Transport>(contract: &Contract<T>, name: &str) -> Result<H256, String> {
    contract
        .abi()
        .event(name)
        .map(|r| r.signature())
        .map_err(|e| (format!("get event signature failed:{}", e)))
}
