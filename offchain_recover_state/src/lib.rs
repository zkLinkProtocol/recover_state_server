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
pub mod log;

#[cfg(test)]
mod tests;

// How many blocks we will process at once.
pub const VIEW_BLOCKS_STEP: u64 = 2_000;
pub const END_BLOCK_OFFSET: u64 = 40;

// An error returned by the rpc server because the number of requests was too frequent.
// It is configured according to the documentation of the rpc service.
// The first error comes from the Infura docs(https://docs.infura.io/infura/networks/ethereum/how-to/avoid-rate-limiting).
pub const PRC_REQUEST_FREQUENT_ERROR_SETS:[&str; 1] = ["429 Too Many Requests"];

pub fn get_fully_on_chain_zklink_contract(config: &RecoverStateConfig) -> (u64, impl ZkLinkContract){
    let uncompress_chain_config = config.layer1
        .chain_configs
        .iter()
        .find(|chain| !chain.chain.is_commit_compressed_blocks)
        .unwrap();
    let deploy_block_number = uncompress_chain_config.contract.deployment_block;
    (
        deploy_block_number,
        match uncompress_chain_config.chain.chain_type{
            ChainType::EVM => ZkLinkEvmContract::new(uncompress_chain_config.clone()),
            ChainType::STARKNET => panic!("Not currently supported!")
        }
    )
}