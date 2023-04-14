use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Default, Deserialize, Clone, PartialEq)]
pub enum ChainType {
    #[default]
    EVM,
    STARKNET,
}

impl FromStr for ChainType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chain_type = match s {
            "ETH" => ChainType::EVM,
            "STARKNET" => ChainType::STARKNET,
            _ => return Err("Unsupported chain type".to_string()),
        };
        Ok(chain_type)
    }
}
