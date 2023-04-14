pub use self::version::ZkLinkContractVersion;
pub use self::zklink_evm_contract::ZkLinkEvmContract;
use async_trait::async_trait;
use std::fmt::Debug;
use zklink_types::{Account, BlockNumber, ChainId, H256};

pub mod update_token_events;
pub mod utils;
pub mod v0;
pub mod version;
pub mod zklink_evm_contract;

/// Abstracts the basic information of layer1 Log.
pub trait LogInfo: Debug {
    /// Returns all topics of this log.
    fn topics(&self) -> Vec<H256>;

    /// Returns all topics of this log.
    fn data(&self) -> Vec<u8>;

    /// Returns the transaction hash of this log.
    fn transaction_hash(&self) -> H256;

    /// Returns the height of the block where the log is located
    fn block_number(&self) -> Option<u64>;
}

/// Abstracts the basic information of layer1 transaction.
pub trait TransactionInfo {
    /// Returns the input parameter when the layer1 tx calls the contract api.
    fn input_data(&self) -> anyhow::Result<Vec<u8>>;

    /// Returns the hash of layer1 transaction.
    fn transaction_hash(&self) -> H256;

    /// Returns the block height of layer1 transaction.
    fn block_number(&self) -> Option<u64>;
}

/// Abstracts the basic type and info for all Layer1s.
#[async_trait]
pub trait BlockChain {
    type Log: LogInfo;
    type Transaction: TransactionInfo;

    /// Returns chain id of layer1 blockchain.
    fn layer1_chain_id(&self) -> u32;

    /// Returns chain id allocated by layer2.
    fn layer2_chain_id(&self) -> ChainId;

    /// Return the current block height of this layer1
    async fn block_number(&self) -> anyhow::Result<u64>;
}

/// Abstracts the required api of ZkLink contract for recovering state.
#[async_trait]
pub trait ZkLinkContract: BlockChain {
    /// Returns topics(signature) of event by name;
    fn get_event_signature(&self, name: &str) -> H256;

    /// Returns the fee account of genesis block.
    fn get_genesis_account(&self, genesis_tx: Self::Transaction) -> anyhow::Result<Account>;

    /// Returns all transaction info by transaction hash.
    async fn get_transaction(&self, hash: H256) -> anyhow::Result<Option<Self::Transaction>>;

    /// Returns total number of verified blocks on Rollup contract
    async fn get_total_verified_blocks(&self) -> anyhow::Result<u32>;

    /// Returns the contract logs that occurred on the specified blocks
    ///
    /// # Arguments
    /// * `from` - Start Layer1 block number
    /// * `to` - End Layer1 block number
    async fn get_block_logs(
        &self,
        from: BlockNumber,
        to: BlockNumber,
    ) -> Result<Vec<Self::Log>, anyhow::Error>;

    /// Returns logs about complete contract upgrades.
    async fn get_gatekeeper_logs(&self) -> anyhow::Result<Vec<Self::Log>>;
}
