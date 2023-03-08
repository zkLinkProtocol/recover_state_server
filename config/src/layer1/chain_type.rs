use std::str::FromStr;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub enum ChainType{
    EVM,
    STARKNET,
}

impl Default for ChainType{
    fn default() -> Self { ChainType::EVM }
}

impl FromStr for ChainType{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chain_type = match s {
            "ETH" => ChainType::EVM,
            "STARKNET" => ChainType::STARKNET,
            _ => return Err("Unsupported chain type".to_string())
        };
        Ok(chain_type)
    }
}
