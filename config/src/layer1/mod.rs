use std::collections::HashMap;
use zklink_types::{ChainId, ZkLinkAddress};

use crate::evn_tools::parse_env_to_vec_if_exists;
pub use chain_config::ChainConfig;
pub use chain_type::ChainType;
pub use client_config::ClientConfig;
pub use contract_config::ContractConfig;

mod chain_config;
mod chain_type;
mod client_config;
mod contract_config;

#[derive(Clone, Debug)]
pub struct MultiChainConfigs {
    pub chain_ids: Vec<ChainId>,
    pub chain_configs: Vec<Layer1Config>,
}

impl MultiChainConfigs {
    pub fn from_env() -> Self {
        let chain_ids: Vec<ChainId> = parse_env_to_vec_if_exists("CHAIN_IDS").unwrap();
        let chain_configs = chain_ids
            .iter()
            .map(|chain_id| Layer1Config::from_env((*chain_id).into()))
            .collect::<Vec<_>>();
        Self {
            chain_ids,
            chain_configs,
        }
    }

    pub fn get_contracts(&self) -> HashMap<ChainId, ZkLinkAddress> {
        self.chain_configs
            .iter()
            .map(|c| (c.chain.chain_id, c.contract.address.clone()))
            .collect()
    }

    pub fn get_max_chain_num(&self) -> usize {
        **self.chain_ids.iter().max().unwrap() as usize
    }
}

#[derive(Debug, Clone)]
pub struct Layer1Config {
    pub chain: ChainConfig,
    pub contract: ContractConfig,
    pub client: ClientConfig,
}

impl Layer1Config {
    pub fn from_env(chain_id: u8) -> Self {
        Self {
            chain: ChainConfig::from_env(chain_id),
            contract: ContractConfig::from_env(chain_id),
            client: ClientConfig::from_env(chain_id),
        }
    }
}
