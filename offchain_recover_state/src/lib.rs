use recover_state_config::{ChainType, RecoverStateConfig};
use crate::contract::{ZkLinkContract, ZkLinkEvmContract};

pub mod contract;
pub mod data_restore_driver;
pub mod database_storage_interactor;
pub mod events;
pub mod events_state;
pub mod inmemory_storage_interactor;
pub mod rollup_ops;
pub mod storage_interactor;
pub mod tree_state;
pub mod aggregated_commit_op;
pub mod log;

#[cfg(test)]
mod tests;

// How many blocks we will process at once.
pub const VIEW_BLOCKS_STEP: u64 = 2_000;
pub const END_BLOCK_OFFSET: u64 = 40;

pub fn get_fully_on_chain_zklink_contract(config: &RecoverStateConfig) -> (u64, impl ZkLinkContract){
    let uncompress_chain_config = config.layer1
        .chain_configs
        .iter()
        .find(|chain| !chain.chain.is_commit_compressed_blocks)
        .unwrap();
    let deploy_block_number = uncompress_chain_config.contracts.deployment_block;
    let zklink_contract: impl ZkLinkContract = match uncompress_chain_config.chain.chain_type{
        ChainType::EVM => ZkLinkEvmContract::new(uncompress_chain_config.clone()),
        ChainType::STARKNET => panic!("Not currently supported!")
    };
    (deploy_block_number, zklink_contract)
}