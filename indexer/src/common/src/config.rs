use candid::{CandidType, Deserialize};

#[derive(Clone, Debug, Default, CandidType, Deserialize)]
pub enum SupportedNetwork {
    #[default]
    Mainnet,
    Optimism,
}

#[derive(Clone, Debug, Default, CandidType, Deserialize)]
pub enum Token {
    #[default]
    DAI,
}

impl SupportedNetwork {
    pub fn rpc_url(&self) -> &str {
        match self {
            SupportedNetwork::Mainnet => "https://mainnet.infura.io/v3/TEST",
            SupportedNetwork::Optimism => "https://mainnet.optimism.io",
        }
    }
}

impl Token {
    pub fn address(&self, network: &SupportedNetwork) -> &str {
        match (self, network) {
            (Token::DAI, SupportedNetwork::Mainnet) => "0x6b175474e89094c44da98b954eedeac495271d0f",
            (Token::DAI, SupportedNetwork::Optimism) => {
                "0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1"
            }
        }
    }
    pub fn contract_craeted_block_number(&self, network: &SupportedNetwork) -> u64 {
        match (self, network) {
            (Token::DAI, SupportedNetwork::Mainnet) => 8928158,
            (Token::DAI, SupportedNetwork::Optimism) => 0,
        }
    }
}
