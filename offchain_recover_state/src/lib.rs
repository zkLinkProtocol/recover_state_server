use crate::contract::{ZkLinkContract, ZkLinkEvmContract};
use recover_state_config::{ChainType, RecoverStateConfig};

pub mod contract;
pub mod driver;
pub mod events;
pub mod log;
pub mod rollup_ops;
pub mod storage_interactor;
pub mod tree_state;

#[cfg(test)]
mod tests;

// Delta between last block and last watched block
pub const END_BLOCK_OFFSET: u64 = 40;

// An error returned by the rpc server because the number of requests was too frequent.
// It is configured according to the documentation of the rpc service.
// The first error comes from the Infura docs(https://docs.infura.io/infura/networks/ethereum/how-to/avoid-rate-limiting).
pub const PRC_REQUEST_FREQUENT_ERROR_SETS: [&str; 1] = ["429 Too Many Requests"];

pub fn get_fully_on_chain_zklink_contract(
    config: &RecoverStateConfig,
) -> (u64, u64, impl ZkLinkContract) {
    let uncompress_chain_config = config
        .layer1
        .chain_configs
        .iter()
        .find(|chain| !chain.chain.is_commit_compressed_blocks)
        .unwrap();
    let deploy_block_number = uncompress_chain_config.contract.deployment_block;
    let view_block_step = uncompress_chain_config.client.view_block_step;
    (
        view_block_step,
        deploy_block_number,
        match uncompress_chain_config.chain.chain_type {
            ChainType::EVM => ZkLinkEvmContract::new(uncompress_chain_config.clone()),
            ChainType::STARKNET => panic!("Not currently supported!"),
        },
    )
}
