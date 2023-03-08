use zklink_types::ChainId;

pub use chain_config::ChainConfig;
pub use contract_config::ContractConfig;
pub use client_config::ClientConfig;
pub use chain_type::ChainType;

mod chain_config;
mod contract_config;
mod client_config;
mod chain_type;

pub struct MultiChainConfigs {
    pub chain_ids: Vec<ChainId>,
    pub chain_configs: Vec<Layer1Config>,
}

impl MultiChainConfigs {
    pub fn from_env() -> Self {
        let chain_ids: Vec<ChainId> = zklink_utils::parse_env_to_vec_if_exists("CHAIN_IDS").unwrap();
        let chain_configs = chain_ids.iter()
            .map(|chain_id| Layer1Config::from_env((*chain_id).into()))
            .collect::<Vec<_>>();
        Self{
            chain_ids,
            chain_configs,
        }
    }
}

pub struct Layer1Config{
    pub chain: ChainConfig,
    pub contracts: ContractConfig,
    pub client: ClientConfig,
}

impl Layer1Config {
    pub fn from_env(chain_id: u8) -> Self{
        Self{
            chain: ChainConfig::from_env(chain_id),
            contracts: ContractConfig::from_env(chain_id),
            client: ClientConfig::from_env(chain_id),
        }
    }
}