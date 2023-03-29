use serde::Deserialize;
use zklink_types::ChainId;
use crate::envy_load;
use crate::layer1::ChainType;

#[derive(Default, Debug, Deserialize, Clone, PartialEq)]
pub struct ChainConfig {
    /// chain id defined by zkLink
    pub chain_id: ChainId,
    /// Layer one chain type, for example, the chain type of Ethereum is EVM
    pub chain_type: ChainType,
    /// Gas token symbol
    pub gas_token: String,
    /// Whether sender should commit compressed block
    pub is_commit_compressed_blocks: bool,
}

impl ChainConfig {
    pub fn from_env(chain_id: u8) -> Self {
        envy_load!("chain", format!("CHAIN_{}_", chain_id))
    }
}