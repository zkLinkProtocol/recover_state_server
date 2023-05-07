// Built-in deps
use std::time::{Duration, Instant, UNIX_EPOCH};
// External imports
// Workspace imports
use zklink_crypto::convert::FeConvert;
use zklink_types::{
    block::{Block, ExecutedTx},
    AccountId, BlockNumber, Fr, H256, U256,
};
// Local imports
use self::records::{StorageBlock, StorageBlockState};
use crate::chain::account::records::{
    StorageAccountCreation, StorageAccountOrderUpdate, StorageAccountPubkeyUpdate,
    StorageAccountUpdate, StorageStateUpdates,
};
use crate::chain::block::records::{NewZkLinkTx, StorageBlockOnChainState};
use crate::chain::operations::records::{AggType, StorageTxHash, StorageTxHashData};
use crate::chain::operations::OperationsSchema;
use crate::{
    chain::operations::records::{NewExecutedTransaction, StoredExecutedTransaction},
    QueryResult, StorageProcessor,
};
use chrono::{DateTime, Utc};
use parity_crypto::Keccak256;
use zklink_types::block::FailedExecutedTx;

mod conversion;
pub mod records;

/// Block schema is a primary sidechain storage controller.
///
/// Besides block getters/setters, it provides an `execute_operation` method,
/// which is essential for the sidechain logic, as it causes the state updates in the chain.
#[derive(Debug)]
pub struct BlockSchema<'a, 'c>(pub &'a mut StorageProcessor<'c>);

impl<'a, 'c> BlockSchema<'a, 'c> {
    /// Given a block, stores its transactions in the database.
    pub async fn save_block_transactions(
        &mut self,
        block_number: BlockNumber,
        operations: Vec<ExecutedTx>,
    ) -> QueryResult<()> {
        for tx in operations {
            // Store the executed operation in the corresponding schema.
            let new_tx = NewExecutedTransaction::prepare_stored_tx(tx, block_number);
            OperationsSchema(self.0).store_executed_tx(new_tx).await?;
        }
        Ok(())
    }

    /// Given a block, update its priority transactions in the database.
    pub async fn update_block_transactions(
        &mut self,
        block_number: BlockNumber,
        operations: Vec<ExecutedTx>,
    ) -> QueryResult<()> {
        for tx in operations {
            if tx.get_executed_op().is_priority_operation(){
                // Store the executed priority operation in the corresponding schema.
                let new_tx = NewExecutedTransaction::prepare_stored_priority_tx(tx, block_number);
                OperationsSchema(self.0).update_executed_tx(new_tx).await?;
            } else {
                // Store the executed operation in the corresponding schema.
                let new_tx = NewExecutedTransaction::prepare_stored_tx(tx, block_number);
                OperationsSchema(self.0).store_executed_tx(new_tx).await?;
            }
        }
        Ok(())
    }

    pub async fn save_block_failed_transactions(
        &mut self,
        block_number: BlockNumber,
        operations: Vec<FailedExecutedTx>,
    ) -> QueryResult<()> {
        for tx in operations {
            // Store the executed operation in the corresponding schema.
            let new_tx = NewExecutedTransaction::prepare_stored_failed_tx(tx, block_number);
            OperationsSchema(self.0).store_executed_tx(new_tx).await?;
        }
        Ok(())
    }

    // Helper method for retrieving blocks from the database.
    pub async fn get_storage_block(&mut self, block: i64) -> QueryResult<Option<StorageBlock>> {
        let start = Instant::now();
        let block = sqlx::query_as!(
            StorageBlock,
            "SELECT * FROM blocks WHERE number = $1",
            block
        )
        .fetch_optional(self.0.conn())
        .await?;

        metrics::histogram!("sql.chain.block.get_storage_block", start.elapsed());

        Ok(block)
    }

    // Helper method for retrieving blocks from the database.
    pub async fn get_last_block_number(&mut self) -> QueryResult<i64> {
        // we use fetch_one because there should be at least one block(the genesis block) stored in table blocks
        let block_number = sqlx::query!("SELECT max(number) from blocks",)
            .fetch_one(self.0.conn())
            .await?
            .max
            .unwrap_or_default();
        Ok(block_number)
    }

    // Helper method for retrieving block state from the database
    pub async fn get_block_state(&mut self) -> QueryResult<StorageBlockState> {
        let mut transaction = self.0.start_transaction().await?;

        // we use fetch_one because there should be at least one block(the genesis block) stored in table blocks
        let last_block = sqlx::query_as!(
            StorageBlock,
            "SELECT * from blocks order by number desc limit 1",
        )
        .fetch_one(transaction.conn())
        .await?;

        let last_committed_block = OperationsSchema(&mut transaction)
            .get_last_block_by_aggregated_action(AggType::CommitBlocks, true)
            .await
            .unwrap();

        let last_verified_block = OperationsSchema(&mut transaction)
            .get_last_block_by_aggregated_action(AggType::ExecuteBlocks, true)
            .await
            .unwrap();

        transaction.commit().await?;

        Ok(StorageBlockState {
            last_block_number: last_block.number,
            created_at: last_block.created_at,
            committed: last_committed_block,
            verified: last_verified_block,
        })
    }

    // Helper method for retrieving block onchain state from the database
    pub async fn get_block_onchain_state(
        &mut self,
        block_number: i64,
    ) -> QueryResult<StorageBlockOnChainState> {
        let mut transaction = self.0.start_transaction().await?;

        let committed = OperationsSchema(&mut transaction)
            .get_block_onchain(block_number, AggType::CommitBlocks)
            .await?;

        let verified = OperationsSchema(&mut transaction)
            .get_block_onchain(block_number, AggType::ExecuteBlocks)
            .await?;

        transaction.commit().await?;

        Ok(StorageBlockOnChainState {
            committed,
            verified,
        })
    }

    /// Given the block number, attempts to retrieve it from the database.
    /// Returns `None` if the block with provided number does not exist yet.
    pub async fn get_block(&mut self, block: i64) -> QueryResult<Option<Block>> {
        let start = Instant::now();
        // Load block header.
        let Some(stored_block) = self.get_storage_block(block).await? else {
            return Ok(None);
        };

        // Load transactions for this block.
        let block_transactions = self.get_block_executed_ops(block).await?;

        // Encode the root hash as `0xFF..FF`.
        let new_root_hash =
            FeConvert::from_bytes(&stored_block.root_hash).expect("Unparsable root hash");

        let commitment = H256::from_slice(&stored_block.commitment);
        let sync_hash = H256::from_slice(&stored_block.sync_hash);
        // Return the obtained block in the expected format.

        let mut timestamp = stored_block.created_at.timestamp();
        if block == 0 {
            timestamp = 0;
        }
        let result = Block::new(
            BlockNumber(block as u32),
            new_root_hash,
            AccountId(stored_block.fee_account_id as u32),
            block_transactions,
            stored_block.block_size as usize,
            stored_block.ops_composition_number as usize,
            U256::from(stored_block.commit_gas_limit as u64),
            U256::from(stored_block.verify_gas_limit as u64),
            commitment,
            sync_hash,
            timestamp as u64,
        );

        metrics::histogram!("sql.chain.block.get_block", start.elapsed());

        Ok(Some(result))
    }

    /// Given the block number, loads all the operations that were executed in that block.
    pub async fn get_block_executed_ops(&mut self, block: i64) -> QueryResult<Vec<ExecutedTx>> {
        let start = Instant::now();
        let mut executed_operations = Vec::new();

        // Load both executed transactions from the database.
        let executed_ops = sqlx::query_as!(
                StoredExecutedTransaction,
                "SELECT block_number, block_index, tx_data, operation, tx_hash, from_account,\
                  to_account, success, fail_reason, nonce, created_at FROM submit_txs WHERE block_number = $1 AND success = true",
                block
            )
            .fetch_all(self.0.conn())
            .await?;

        // Transform executed operations to be `ExecutedOperations`.
        let executed_ops = executed_ops
            .into_iter()
            .filter_map(|stored_exec| stored_exec.into_executed_tx().ok());
        executed_operations.extend(executed_ops);

        // Sort the operations, so all the failed operations will be at the very end
        // of the list.
        executed_operations.sort_by_key(|exec_op| {
            if let Some(idx) = exec_op.block_index {
                idx
            } else {
                // failed operations are at the end. all failed_tx.block_index are set to 0
                // and will not contain in the query result
                u32::MAX
            }
        });

        metrics::histogram!("sql.chain.block.get_block_executed_ops", start.elapsed());
        Ok(executed_operations)
    }

    /// Returns the number of last block for which proof has been confirmed on Ethereum.
    /// Essentially, it's number of last block for which updates were applied to the chain state.
    pub async fn get_last_verified_confirmed_block(&mut self) -> QueryResult<i64> {
        let start = Instant::now();
        let result = OperationsSchema(self.0)
            .get_last_block_by_aggregated_action(AggType::ExecuteBlocks, true)
            .await;
        metrics::histogram!(
            "sql.chain.block.get_last_verified_confirmed_block",
            start.elapsed()
        );
        result
    }

    pub async fn save_block(&mut self, block: Block) -> QueryResult<()> {
        let mut transaction = self.0.start_transaction().await?;

        let number = i64::from(*block.block_number);
        let root_hash = block.new_root_hash.to_bytes();
        let fee_account_id = i64::from(*block.fee_account);

        let block_size = block.block_chunks_size as i64;
        let ops_composition_number = block.ops_composition_number as i64;
        let commit_gas_limit = block.commit_gas_limit.as_u64() as i64;
        let verify_gas_limit = block.verify_gas_limit.as_u64() as i64;
        let commitment = block.block_commitment.as_bytes().to_vec();
        let sync_hash = block.sync_hash.as_bytes().to_vec();
        // Creates a new SystemTime from the specified number of whole seconds
        let d = UNIX_EPOCH + Duration::from_secs(block.timestamp);
        // Create DateTime from SystemTime
        let created_at = DateTime::<Utc>::from(d);
        BlockSchema(&mut transaction)
            .update_block_transactions(block.block_number, block.block_transactions)
            .await?;

        let new_block = StorageBlock {
            number,
            root_hash,
            fee_account_id,
            block_size,
            ops_composition_number,
            commit_gas_limit,
            verify_gas_limit,
            commitment,
            sync_hash,
            created_at,
        };

        // Save new completed block.
        sqlx::query!("
            INSERT INTO blocks (number, root_hash, fee_account_id, block_size, ops_composition_number, commit_gas_limit, verify_gas_limit, commitment, sync_hash, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ",
            new_block.number, new_block.root_hash, new_block.fee_account_id,
            new_block.block_size, new_block.ops_composition_number, new_block.commit_gas_limit, new_block.verify_gas_limit,
            new_block.commitment, new_block.sync_hash, new_block.created_at,
        ).execute(transaction.conn())
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    pub async fn save_genesis_block(&mut self, root_hash: Fr) -> QueryResult<()> {
        let block = Block {
            block_number: BlockNumber(0),
            new_root_hash: root_hash,
            fee_account: AccountId(0),
            block_transactions: Vec::new(),
            block_chunks_size: 0,
            ops_composition_number: 0,
            commit_gas_limit: 0u32.into(),
            verify_gas_limit: 0u32.into(),
            block_commitment: H256::zero(),
            sync_hash: H256::from(Vec::new().keccak256()),
            timestamp: 0u64,
        };

        self.save_block(block).await
    }

    pub async fn get_last_tx_id(&mut self, op_types: &[i16]) -> QueryResult<i64> {
        let last_tx_id = sqlx::query!(
            r#"
            SELECT max(id) FROM submit_txs WHERE executed = true AND op_type = ANY($1)
            "#,
            op_types,
        )
        .fetch_one(self.0.conn())
        .await?;

        // if no tx executed, return 0 is safe, because tx id start from 1
        let id = last_tx_id.max.unwrap_or(0);
        Ok(id)
    }

    /// Load a batch of un_executed txs
    pub async fn get_new_block_txs(
        &mut self,
        last_tx_id: i64,
        op_types: &[i16],
        limit: i64,
    ) -> QueryResult<Vec<NewZkLinkTx>> {
        // select where id > last_tx_id without equal
        // order id by ASC, first in first out
        let new_txs = sqlx::query_as!(
            NewZkLinkTx,
            r#"
            SELECT id, tx_data FROM submit_txs WHERE id > $1 AND executed = false AND op_type = ANY($2) ORDER BY id ASC LIMIT $3
            "#,
            last_tx_id,
            op_types,
            limit,
        )
            .fetch_all(self.0.conn())
            .await?;

        Ok(new_txs)
    }

    pub async fn get_block_tx_hash_list(
        &mut self,
        block_number: i64,
    ) -> QueryResult<Vec<StorageTxHash>> {
        let list = sqlx::query_as!(
            StorageTxHash,
            r#"
            SELECT tx_hash FROM submit_txs WHERE block_number = $1 and success = true
            ORDER BY block_index ASC
            "#,
            block_number,
        )
        .fetch_all(self.0.conn())
        .await?;
        Ok(list)
    }

    pub async fn get_block_tx_hash_data_list(
        &mut self,
        block_number: i64,
    ) -> QueryResult<Vec<StorageTxHashData>> {
        let list = sqlx::query_as!(
            StorageTxHashData,
            r#"
            SELECT tx_hash, tx_data FROM submit_txs WHERE block_number = $1 and success = true
            ORDER BY block_index ASC
            "#,
            block_number,
        )
        .fetch_all(self.0.conn())
        .await?;
        Ok(list)
    }

    pub async fn get_block_state_updates(
        &mut self,
        block_number: i64,
    ) -> QueryResult<StorageStateUpdates> {
        // no need to do update query in a transaction, because all updates
        // will be write to database in a transaction
        let account_creates = self.get_account_creates_by_block(block_number).await?;
        let balance_updates = self
            .get_account_balance_updates_by_block(block_number)
            .await?;
        let order_nonce_updates = self
            .get_account_order_updates_by_block(block_number)
            .await?;
        let account_pubkey_updates = self
            .get_account_pubkey_updates_by_block(block_number)
            .await?;
        Ok(StorageStateUpdates {
            account_creates,
            balance_updates,
            order_nonce_updates,
            account_pubkey_updates,
        })
    }

    pub async fn get_account_creates_by_block(
        &mut self,
        block_number: i64,
    ) -> QueryResult<Vec<StorageAccountCreation>> {
        let updates = sqlx::query_as!(
            StorageAccountCreation,
            r#"
                SELECT * FROM account_creates
                WHERE block_number=$1
                ORDER BY account_id ASC
            "#,
            block_number,
        )
        .fetch_all(self.0.conn())
        .await?;

        Ok(updates)
    }

    pub async fn get_account_balance_updates_by_block(
        &mut self,
        block_number: i64,
    ) -> QueryResult<Vec<StorageAccountUpdate>> {
        let updates = sqlx::query_as!(
            StorageAccountUpdate,
            r#"
                SELECT * FROM account_balance_updates
                WHERE block_number=$1
                ORDER BY balance_update_id ASC
            "#,
            block_number,
        )
        .fetch_all(self.0.conn())
        .await?;

        Ok(updates)
    }

    pub async fn get_account_order_updates_by_block(
        &mut self,
        block_number: i64,
    ) -> QueryResult<Vec<StorageAccountOrderUpdate>> {
        let updates = sqlx::query_as!(
            StorageAccountOrderUpdate,
            r#"
                SELECT * FROM account_order_updates
                WHERE block_number=$1
                ORDER BY order_nonce_update_id ASC
            "#,
            block_number,
        )
        .fetch_all(self.0.conn())
        .await?;

        Ok(updates)
    }

    pub async fn get_account_pubkey_updates_by_block(
        &mut self,
        block_number: i64,
    ) -> QueryResult<Vec<StorageAccountPubkeyUpdate>> {
        let updates = sqlx::query_as!(
            StorageAccountPubkeyUpdate,
            r#"
                SELECT * FROM account_pubkey_updates
                WHERE block_number=$1
                ORDER BY pubkey_update_id ASC
            "#,
            block_number,
        )
        .fetch_all(self.0.conn())
        .await?;

        Ok(updates)
    }
}
