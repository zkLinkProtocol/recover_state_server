// External uses
use serde::Deserialize;
// Workspace uses
use zklink_types::{ZkLinkAddress, H256};
// Local uses
use crate::envy_load;

/// Data about deployed contracts.
#[derive(Default, Debug, Deserialize, Clone, PartialEq)]
pub struct ContractConfig {
    /// The block number of contracts deployed.
    pub deployment_block: u64,
    /// The zkLink main contract address
    pub address: ZkLinkAddress,
    /// The zkLink contract deployed tx hash, used for recover data
    pub genesis_tx_hash: H256,
}

impl ContractConfig {
    pub fn from_env(chain_id: u8) -> Self {
        envy_load!("contract", format!("CHAIN_{}_CONTRACT_", chain_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configs::test_utils::{addr, hash, set_env};

    fn expected_config() -> ContractConfig {
        ContractConfig {
            deployment_block: 10000,
            address: "0x70a0F165d6f8054d0d0CF8dFd4DD2005f0AF6B55"
                .parse()
                .unwrap(),
            genesis_tx_hash: "0xb99ebfea46cbe05a21cd80fe5597d97b204befc52a16303f579c607dc1ac2e2e"
                .parse()
                .unwrap(),
        }
    }

    #[test]
    fn from_env() {
        let config = r#"
            CHAIN_1_CONTRACT_DEPLOYMENT_BLOCK="10000"
            CHAIN_1_CONTRACT_CONTRACT_ADDR="0x70a0F165d6f8054d0d0CF8dFd4DD2005f0AF6B55"
            CHAIN_1_CONTRACT_GENESIS_TX_HASH="0xb99ebfea46cbe05a21cd80fe5597d97b204befc52a16303f579c607dc1ac2e2e"
        "#;
        set_env(config);

        let actual = ContractConfig::from_env(1);
        assert_eq!(actual, expected_config());
    }
}
