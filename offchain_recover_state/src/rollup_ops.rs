use zklink_types::operations::ZkLinkOp;

use crate::contract::{TransactionInfo, ZkLinkContract, ZkLinkContractVersion};
use crate::events::BlockEvent;
use zklink_types::{AccountId, BlockNumber, H256};

/// Description of a Rollup operations block
#[derive(Debug, Clone)]
pub struct RollupOpsBlock {
    /// Rollup block number
    pub block_num: BlockNumber,
    /// Rollup operations in block
    pub ops: Vec<ZkLinkOp>,
    /// Fee account
    pub fee_account: AccountId,
    /// Timestamp
    pub timestamp: Option<u64>,
    /// Previous block root hash.
    pub previous_block_root_hash: H256,
    /// zkLink contract version for the given block.
    /// Used to obtain block chunk sizes. Stored in the database
    /// in the corresponding block event.
    pub contract_version: Option<ZkLinkContractVersion>,
}

impl RollupOpsBlock {
    /// Returns a Rollup operations block description
    ///
    /// # Arguments
    ///
    /// * `zklink_contract` - the contract provider of zklink
    /// * `event_data` - Rollup contract event description
    ///
    ///
    pub async fn get_rollup_ops_blocks<T: ZkLinkContract>(
        zklink_contract: &T,
        event: &BlockEvent,
    ) -> anyhow::Result<Vec<Self>> {
        let transaction = zklink_contract
            .get_transaction(event.transaction_hash)
            .await?
            .expect("The transaction must exist");
        let input_data = transaction.input_data()?;
        let blocks: Vec<RollupOpsBlock> = event
            .contract_version
            .rollup_ops_blocks_from_bytes(input_data)?;
        Ok(blocks)
    }
}
