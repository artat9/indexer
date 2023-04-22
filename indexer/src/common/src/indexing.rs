use candid::{CandidType, Deserialize};

use crate::config::{SupportedNetwork, Token};

#[derive(Clone, Debug, CandidType, Default, Deserialize)]
pub struct IndexingConfig {
    pub network: SupportedNetwork,
    pub token: Token,
    pub batch_sync_start_from: u64,
}

impl IndexingConfig {
    pub fn new(network: SupportedNetwork, token: Token, start_from: u64) -> Self {
        Self {
            network,
            token,
            batch_sync_start_from: start_from,
        }
    }
    pub fn new_dai_mainnet() -> Self {
        Self::new(SupportedNetwork::Mainnet, Token::DAI, 17099971)
    }
    pub fn new_dai_optimism() -> Self {
        Self::new(SupportedNetwork::Optimism, Token::DAI, 727037)
    }
    pub fn indexing_start_from(&self) -> u64 {
        self.batch_sync_start_from
    }
    pub fn address(&self) -> &str {
        self.token.address(&self.network)
    }

    pub fn deployed_block(&self) -> u64 {
        self.token.deployed_block(&self.network)
    }
}
