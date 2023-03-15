// Built-in deps
use std::time::Instant;
use tracing::info;
use zklink_basic_types::{AccountId, BlockNumber, ChainId};
use zklink_types::{AccountUpdate, H256};
// External imports
// Workspace imports
// Local imports
use self::records::{
    NewBlockEvent, NewRollupOpsBlock, NewStorageState, StoredBlockEvent,
    StoredRollupOpsBlock, StoredStorageState,
};
use crate::chain::operations::OperationsSchema;
use crate::{chain::state::StateSchema};
use crate::{QueryResult, StorageProcessor};
use crate::chain::operations::records::StoredAggregatedOperation;

pub mod records;

/// Data restore schema provides a convenient interface to restore the
/// sidechain state from the Ethereum contract.
///
/// This schema is used exclusively by the `recover_state` crate.
#[derive(Debug)]
pub struct RecoverSchema<'a, 'c>(pub &'a mut StorageProcessor<'c>);

impl<'a, 'c> RecoverSchema<'a, 'c> {
    pub async fn save_block_operations(
        &mut self,
        commit_op: &StoredAggregatedOperation,
        execute_op: &StoredAggregatedOperation,
    ) -> QueryResult<()> {
        let start = Instant::now();
        let new_state = self.new_storage_state("None");
        let mut transaction = self.0.start_transaction().await?;

        OperationsSchema(&mut transaction)
            .store_aggregated_action(commit_op)
            .await?;
        OperationsSchema(&mut transaction)
            .store_aggregated_action(execute_op)
            .await?;
        // The state is expected to be updated, so it's necessary
        // to do it here.
        for block_number in commit_op.from_block..commit_op.to_block + 1 {
            StateSchema(&mut transaction)
                .apply_state_update(block_number.into())
                .await?;
        }

        RecoverSchema(&mut transaction)
            .update_storage_state(new_state)
            .await?;
        transaction.commit().await?;
        metrics::histogram!("sql.recover_state.save_block_operations", start.elapsed());
        Ok(())
    }

    pub async fn save_genesis_state(
        &mut self,
        genesis_updates: &[(AccountId, AccountUpdate, H256)],
    ) -> QueryResult<()> {
        let start = Instant::now();
        let mut transaction = self.0.start_transaction().await?;
        StateSchema(&mut transaction)
            .commit_state_update(BlockNumber(0), genesis_updates)
            .await?;
        StateSchema(&mut transaction)
            .apply_state_update(BlockNumber(0))
            .await?;
        transaction.commit().await?;
        metrics::histogram!("sql.recover_state.save_genesis_state", start.elapsed());
        Ok(())
    }

    pub async fn load_rollup_ops_blocks(&mut self) -> QueryResult<Vec<StoredRollupOpsBlock>> {
        let start = Instant::now();
        // For each block aggregate its operations from the
        // `recover_state_rollup_block_ops` table into array and
        // match it by the block number from `recover_state_rollup_blocks`.
        // The contract version is obtained from block events.
        let stored_blocks = sqlx::query_as!(
            StoredRollupOpsBlock,
            "SELECT block_num, operation, fee_account, created_at, previous_block_root_hash, contract_version \
            FROM recover_state_rollup_ops ORDER BY block_num ASC"
        )
            .fetch_all(self.0.conn())
            .await?;
        metrics::histogram!("sql.recover_state.load_rollup_ops_blocks", start.elapsed());
        Ok(stored_blocks)
    }

    /// update the last seen layer1 block number.
    pub async fn update_last_watched_block_number(
        &mut self,
        chain_id: i16,
        event_type: &str,
        block_number: i64,
    ) -> QueryResult<()> {
        let start = Instant::now();

        sqlx::query!(
            "UPDATE recover_state_last_watched_block SET block_number=$1 WHERE chain_id=$2 AND event_type=$3",
            block_number,
            chain_id,
            event_type
        )
            .execute(self.0.conn())
            .await?;

        metrics::histogram!(
            "sql.recover_state.update_last_watched_block_number",
            start.elapsed()
        );
        Ok(())
    }

    /// Loads the last seen layer1 block number of updating events(e.g block, token).
    pub async fn last_watched_block_number(
        &mut self,
        chain_id: i16,
        event_type: &str
    ) -> QueryResult<Option<i64>> {
        let start = Instant::now();
        let stored = sqlx::query!(
            "SELECT block_number FROM recover_state_last_watched_block WHERE chain_id=$1 and event_type=$2",
            chain_id,
            event_type
        )
            .fetch_optional(self.0.conn())
            .await?
            .map(|num|num.block_number);

        metrics::histogram!(
            "sql.recover_state.last_watched_block_number",
            start.elapsed()
        );
        Ok(stored)
    }

    /// store the last seen layer1 block number.
    pub async fn insert_last_watched_block_number(
        &mut self,
        chain_id: i16,
        event_type: &str,
        block_number: i64,
    ) -> QueryResult<()> {
        let start = Instant::now();

        sqlx::query!(
            "INSERT INTO recover_state_last_watched_block (chain_id, event_type, block_number) VALUES ($1, $2, $3)\
            ON CONFLICT (chain_id, event_type) DO NOTHING",
            chain_id,
            event_type,
            block_number,
        )
            .execute(self.0.conn())
            .await?;

        metrics::histogram!(
            "sql.recover_state.insert_last_watched_block_number",
            start.elapsed()
        );
        Ok(())
    }

    fn new_storage_state(&self, state: impl ToString) -> NewStorageState {
        info!("Enter {:?} storage state", state.to_string());
        NewStorageState {
            storage_state: state.to_string(),
        }
    }

    pub async fn insert_block_events_state(
        &mut self,
        chain_id: ChainId,
        last_watched_block_number: u64,
    ) -> QueryResult<()> {
        let start = Instant::now();
        let new_state = self.new_storage_state("Events");
        let mut transaction = self.0.start_transaction().await?;

        RecoverSchema(&mut transaction)
            .insert_last_watched_block_number(*chain_id as i16, "block", last_watched_block_number as i64)
            .await?;
        RecoverSchema(&mut transaction)
            .update_storage_state(new_state)
            .await?;

        transaction.commit().await?;

        metrics::histogram!("sql.recover_state.save_events_state", start.elapsed());
        Ok(())
    }

    pub async fn update_block_events_state(
        &mut self,
        chain_id: ChainId,
        block_events: &[NewBlockEvent],
        last_watched_block_number: u64,
    ) -> QueryResult<()> {
        let start = Instant::now();
        let new_state = self.new_storage_state("Events");
        let mut transaction = self.0.start_transaction().await?;

        RecoverSchema(&mut transaction)
            .update_block_events(block_events)
            .await?;

        RecoverSchema(&mut transaction)
            .update_last_watched_block_number(*chain_id as i16, "block", last_watched_block_number as i64)
            .await?;
        RecoverSchema(&mut transaction)
            .update_storage_state(new_state)
            .await?;

        transaction.commit().await?;

        metrics::histogram!("sql.recover_state.save_events_state", start.elapsed());
        Ok(())
    }

    pub async fn save_rollup_ops(
        &mut self,
        rollup_blocks: &[NewRollupOpsBlock<'_>],
    ) -> QueryResult<()> {
        let start = Instant::now();
        let new_state = self.new_storage_state("Operations");
        let mut transaction = self.0.start_transaction().await?;
        // Clean up the blocks table after applying last batch blocks.
        sqlx::query!("DELETE FROM recover_state_rollup_ops")
            .execute(transaction.conn())
            .await?;

        for block in rollup_blocks {
            let operations= serde_json::to_value(block.ops).unwrap();
            sqlx::query!(
                "INSERT INTO recover_state_rollup_ops (block_num, operation, fee_account, created_at, previous_block_root_hash, contract_version)
                VALUES ($1, $2, $3, $4, $5, $6)",
                i64::from(*block.block_num),
                operations,
                i64::from(*block.fee_account),
                block.timestamp,
                block.previous_block_root_hash.as_bytes(),
                block.contract_version
            )
            .execute(transaction.conn())
            .await?;
        }
        RecoverSchema(&mut transaction)
            .update_storage_state(new_state)
            .await?;
        transaction.commit().await?;
        metrics::histogram!("sql.recover_state.save_rollup_ops", start.elapsed());
        Ok(())
    }

    async fn load_events_state(&mut self, state: &str) -> QueryResult<Vec<StoredBlockEvent>> {
        let start = Instant::now();
        let events = sqlx::query_as!(
            StoredBlockEvent,
            "SELECT * FROM recover_state_events_state
            WHERE block_type = $1
            ORDER BY block_num ASC",
            state,
        )
        .fetch_all(self.0.conn())
        .await?;

        metrics::histogram!("sql.recover_state.load_events_state", start.elapsed());
        Ok(events)
    }

    pub async fn load_committed_events_state(&mut self) -> QueryResult<Vec<StoredBlockEvent>> {
        self.load_events_state("Committed").await
    }

    pub async fn load_verified_events_state(&mut self) -> QueryResult<Vec<StoredBlockEvent>> {
        self.load_events_state("Verified").await
    }

    pub async fn load_storage_state(&mut self) -> QueryResult<StoredStorageState> {
        let start = Instant::now();
        let state = sqlx::query_as!(
            StoredStorageState,
            "SELECT * FROM recover_state_storage_state_update
            LIMIT 1",
        )
        .fetch_one(self.0.conn())
        .await?;

        metrics::histogram!("sql.recover_state.load_storage_state", start.elapsed());
        Ok(state)
    }

    pub(crate) async fn update_storage_state(&mut self, state: NewStorageState) -> QueryResult<()> {
        let start = Instant::now();
        let mut transaction = self.0.start_transaction().await?;
        sqlx::query!("DELETE FROM recover_state_storage_state_update")
            .execute(transaction.conn())
            .await?;

        sqlx::query!(
            "INSERT INTO recover_state_storage_state_update (storage_state) VALUES ($1)",
            state.storage_state,
        )
            .execute(transaction.conn())
            .await?;
        transaction.commit().await?;

        metrics::histogram!("sql.recover_state.update_storage_state", start.elapsed());
        Ok(())
    }

    pub(crate) async fn update_block_events(
        &mut self,
        events: &[NewBlockEvent],
    ) -> QueryResult<()> {
        let start = Instant::now();
        let mut transaction = self.0.start_transaction().await?;
        sqlx::query!("DELETE FROM recover_state_events_state")
            .execute(transaction.conn())
            .await?;

        for event in events.iter() {
            sqlx::query!(
                "INSERT INTO recover_state_events_state (block_type, transaction_hash, block_num, contract_version) \
                VALUES ($1, $2, $3, $4)",
                event.block_type, event.transaction_hash, event.block_num, event.contract_version
            )
            .execute(transaction.conn())
            .await?;
        }
        transaction.commit().await?;
        metrics::histogram!("sql.recover_state.update_block_events", start.elapsed());
        Ok(())
    }
}
