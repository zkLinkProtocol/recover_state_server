use std::collections::HashMap;
use std::convert::TryFrom;
use zklink_storage::chain::operations::records::StoredSubmitTransaction;
use zklink_storage::recover_state::records::{
    NewBlockEvent, StoredBlockEvent, StoredRollupOpsBlock,
};
use zklink_types::{
    block::Block, AccountId, AccountMap, AccountUpdate, BlockNumber, ChainId, Token, TokenId, H256,
};

use crate::contract::utils::NewToken;
use crate::{
    contract::ZkLinkContractVersion,
    data_restore_driver::StorageUpdateState,
    events::{BlockEvent, EventType},
    events_state::RollUpEvents,
    rollup_ops::RollupOpsBlock,
};

pub struct StoredTreeState {
    pub last_block_number: BlockNumber,
    pub last_serial_ids: HashMap<ChainId, i64>,
    pub account_map: AccountMap,
    pub fee_acc_id: AccountId,
}

#[async_trait::async_trait]
pub trait StorageInteractor {
    /// loads all token and supported chains.
    async fn load_tokens(&mut self) -> HashMap<TokenId, Token>;

    /// Update priority ops and tokens events to the storage
    ///
    /// # Arguments
    ///
    /// * `chain_id` - the chain id of stored tokens
    /// * `last_watched_block_number` - the block height of syncing token events
    /// * `last_serial_id` - the serial id of syncing priority op
    /// * `submit_ops` - all priority ops(etc Deposit, FullExit)
    /// * `token_events` - Token events that emitted when call addToken api of contract
    ///
    async fn update_priority_ops_and_tokens(
        &mut self,
        chain_id: ChainId,
        last_watched_block_number: u64,
        last_serial_id: i64,
        submit_ops: Vec<StoredSubmitTransaction>,
        token_events: Vec<NewToken>,
        symbols: Vec<String>,
    );

    /// Saves Rollup operations blocks in storage
    ///
    /// # Arguments
    ///
    /// * `blocks` - Rollup operations blocks
    ///
    async fn save_rollup_ops(&mut self, blocks: &[RollupOpsBlock]);

    /// stores blocks and account updates
    ///
    /// # Arguments
    ///
    /// * `blocks_updated` - blocks and account updated
    ///
    async fn store_blocks_and_updates(
        &mut self,
        blocks_and_updates: Vec<(Block, Vec<(AccountId, AccountUpdate, H256)>)>,
    );

    /// Init the progress of syncing token events.
    /// # Arguments
    ///
    /// * `chain_id` - the chain id of syncing token events
    /// * `last_watched_block_number` - the original block height of syncing token events
    ///
    async fn init_token_event_progress(
        &mut self,
        chain_id: ChainId,
        last_block_number: BlockNumber,
    );

    async fn init_block_events_state(&mut self, chain_id: ChainId, last_watched_block_number: u64);

    /// Update Rollup contract events in storage (includes block events, new tokens and last watched block number)
    ///
    /// # Arguments
    ///
    /// * `eveblock_eventsnts` - Rollup contract block events descriptions
    /// * `tokens` - Tokens that had been added to system
    /// * `last_watched_block_number` - Last watched layer1 block
    ///
    async fn update_block_events_state(
        &mut self,
        chain_id: ChainId,
        block_events: &[BlockEvent],
        last_watched_block_number: u64,
    ) -> anyhow::Result<()>;

    /// Saves genesis accounts state in storage
    ///
    /// # Arguments
    ///
    /// * `genesis_updates` - Genesis account updates
    ///
    async fn save_genesis_tree_state(
        &mut self,
        genesis_updates: &[(AccountId, AccountUpdate, H256)],
    );

    /// Returns Rollup contract events state from storage
    async fn get_block_events_state_from_storage(&mut self, chain_id: ChainId) -> RollUpEvents;

    /// Returns the current Rollup block, tree accounts map, unprocessed priority ops and the last fee acc from storage
    async fn get_tree_state(&mut self, chain_ids: Vec<ChainId>) -> StoredTreeState;

    /// Returns Rollup operations blocks from storage
    async fn get_ops_blocks_from_storage(&mut self) -> Vec<RollupOpsBlock>;

    /// Returns last recovery state update step from storage
    async fn get_storage_state(&mut self) -> StorageUpdateState;
}

/// Returns Rollup contract event from its stored representation
///
/// # Arguments
///
/// * `block` - Stored representation of ZkLink Contract event
///
pub fn stored_block_event_into_block_event(block: StoredBlockEvent) -> BlockEvent {
    BlockEvent {
        block_num: BlockNumber(
            u32::try_from(block.block_num).expect("Wrong block number - cant convert into u32"),
        ),
        transaction_hash: H256::from_slice(block.transaction_hash.as_slice()),
        block_type: match &block.block_type {
            c if c == "Committed" => EventType::Committed,
            v if v == "Verified" => EventType::Verified,
            _ => panic!("Wrong block type"),
        },
        contract_version: ZkLinkContractVersion::try_from(block.contract_version as u32)
            .unwrap_or(ZkLinkContractVersion::V0),
    }
}

/// Get new stored representation of the Rollup contract event from itself
///
/// # Arguments
///
/// * `event` - Rollup contract event description
///
pub fn block_event_into_stored_block_event(event: &BlockEvent) -> NewBlockEvent {
    NewBlockEvent {
        block_type: match event.block_type {
            EventType::Committed => "Committed".to_string(),
            EventType::Verified => "Verified".to_string(),
        },
        transaction_hash: event.transaction_hash.as_bytes().to_vec(),
        block_num: i64::from(*event.block_num),
        contract_version: event.contract_version.into(),
    }
}

/// Returns Rollup operations block from its stored representation
///
/// # Arguments
///
/// * `op_block` - Stored ZkLink operations block description
///
pub fn stored_ops_block_into_ops_block(op_block: StoredRollupOpsBlock) -> RollupOpsBlock {
    let ops = serde_json::from_value(op_block.operation).unwrap();
    RollupOpsBlock {
        block_num: BlockNumber::from(op_block.block_num as u32),
        ops,
        fee_account: AccountId::from(op_block.fee_account as u32),
        timestamp: op_block.created_at.map(|t| t.timestamp_millis() as u64),
        previous_block_root_hash: H256::from_slice(&op_block.previous_block_root_hash),
        contract_version: Some(
            ZkLinkContractVersion::try_from(op_block.contract_version as u32)
                .expect("invalid contract version in the database"),
        ),
    }
}
