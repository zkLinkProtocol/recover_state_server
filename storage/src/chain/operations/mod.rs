use std::collections::HashMap;
// Built-in deps
use std::time::Instant;
use serde_json::Value;
use zklink_basic_types::ChainId;
// Workspace imports
use zklink_types::{BlockNumber, DepositOp, FullExitOp, TransferOp, TransferToNewOp, WithdrawOp, ZkLinkAddress, ZkLinkTx, ZkLinkTxType};
// Local imports
use self::records::{
    NewExecutedTransaction, StoredAggregatedOperation, StoredSubmitTransaction
};
use crate::chain::operations::records::{AggType, StoredOnChainTx};
use crate::{QueryResult, StorageProcessor};
use crate::chain::account::records::{StorageAccountCreation, StorageAccountOrderUpdate, StorageAccountPubkeyUpdate, StorageAccountUpdate, StorageStateUpdates};

pub mod records;

/// Operations schema is capable of storing and loading the transactions.
/// Every kind of transaction (non-executed, executed, and executed priority tx)
/// can be either saved or loaded from the database.
#[derive(Debug)]
pub struct OperationsSchema<'a, 'c>(pub &'a mut StorageProcessor<'c>);

impl<'a, 'c> OperationsSchema<'a, 'c> {
    /// batch submit priority transactions and record the latest block number of the priority tX
    pub async fn submit_priority_txs(
        &mut self,
        txs: Vec<StoredSubmitTransaction>,
    ) -> QueryResult<()>{
        let mut transaction = self.0.start_transaction().await?;

        for tx in txs.into_iter() {
            transaction.chain()
                .operations_schema()
                .add_new_submit_tx(tx)
                .await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    /// Return the greatest block number with the given `action_type` and `confirmed` status.
    pub async fn get_last_block_by_aggregated_action(
        &mut self,
        aggregated_action_type: AggType,
        confirmed: bool,
    ) -> QueryResult<i64> {
        let start = Instant::now();
        let max_block = sqlx::query!(
            r#"SELECT max(to_block) FROM aggregate_operations WHERE action_type = $1 AND confirmed = $2"#,
            aggregated_action_type as AggType,
            confirmed
        )
        .fetch_one(self.0.conn())
        .await?
        .max
        .unwrap_or(0);

        metrics::histogram!(
            "sql.chain.operations.get_last_block_by_aggregated_action",
            start.elapsed()
        );
        Ok(max_block)
    }

    /// Return the block onchain info(chain_id, tx_hash) of all chains.
    pub async fn get_block_onchain(
        &mut self,
        block_number: i64,
        aggregated_action_type: AggType,
    ) -> QueryResult<Vec<StoredOnChainTx>> {
        // a confirmed agg op will have final onchain txs
        let op_id = sqlx::query!(
            r#"SELECT id FROM aggregate_operations
            WHERE action_type = $1 AND $2 BETWEEN from_block AND to_block AND confirmed = true"#,
            aggregated_action_type as AggType,
            block_number
        )
            .fetch_optional(self.0.conn())
            .await?;

        let on_chain_txs = match op_id {
            Some(r) => {
                sqlx::query_as!(
                    StoredOnChainTx,
                    r#"SELECT e.chain_id, e.final_hash FROM eth_operations as e
                    INNER JOIN eth_aggregated_ops_binding as b
                    ON e.id = b.eth_op_id
                    WHERE b.op_id = $1"#,
                    r.id
                )
                    .fetch_all(self.0.conn())
                    .await?
            },
            None => {
                vec![]
            }
        };

        Ok(on_chain_txs)
    }

    /// Add a new submit tx in the database.
    pub async fn add_new_submit_tx(
        &mut self,
        tx: StoredSubmitTransaction,
    ) -> QueryResult<()> {
        let start = Instant::now();

        sqlx::query!(
            r#"INSERT INTO submit_txs
            (chain_id, op_type, from_account, to_account, nonce, amount, tx_data, eth_signature, tx_hash, created_at, executed, success, block_number, block_index)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)"#,
            tx.chain_id,
            tx.op_type,
            tx.from_account,
            tx.to_account,
            tx.nonce,
            tx.amount,
            tx.tx_data,
            tx.eth_signature,
            tx.tx_hash,
            tx.created_at,
            false,
            false,
            0,
            0,
        )
            .execute(self.0.conn())
            .await?;

        metrics::histogram!("sql.chain.operations.add_new_submit_tx", start.elapsed());

        Ok(())
    }

    /// Retrieves transaction from the database given its hash.
    pub async fn get_submit_tx_by_hash(
        &mut self,
        op_hash: &[u8],
    ) -> QueryResult<Option<StoredSubmitTransaction>> {
        let start = Instant::now();

        let op = sqlx::query_as!(
            StoredSubmitTransaction,
            "SELECT * FROM submit_txs WHERE tx_hash = $1",
            op_hash
        )
        .fetch_optional(self.0.conn())
        .await?;

        metrics::histogram!(
            "sql.chain.operations.get_submit_tx_by_hash",
            start.elapsed()
        );
        Ok(op)
    }

    /// Retrieves priority transaction from the database given priority transaction serial id.
    pub async fn get_priority_tx_by_serial_id(
        &mut self,
        serial_id: i64,
    ) -> QueryResult<Option<ZkLinkTx>> {
        let start = Instant::now();

        let tx_data = sqlx::query!(
            "SELECT tx_data FROM submit_txs WHERE nonce = $1 and (op_type = $2 or op_type = $3)",
            serial_id,
            DepositOp::OP_CODE as i16,
            FullExitOp::OP_CODE as i16
        )
            .fetch_optional(self.0.conn())
            .await?
            .map(|tx_data|serde_json::from_value(tx_data.tx_data).unwrap());

        metrics::histogram!(
            "sql.chain.operations.get_tx_by_serial_id",
            start.elapsed()
        );
        Ok(tx_data)
    }

    /// Retrieves unprocessed priority transaction from the database given address.
    pub async fn get_unprocessed_priority_tx_by_address(
        &mut self,
        address: &[u8],
    ) -> QueryResult<HashMap<u64, ZkLinkTx>> {
        let start = Instant::now();

        let tx_data = sqlx::query!(
            "SELECT tx_data, nonce FROM submit_txs WHERE to_account = $1 and (op_type = $2 or op_type = $3) and executed=false",
            address, DepositOp::OP_CODE as i16, FullExitOp::OP_CODE as i16
        )
            .fetch_all(self.0.conn())
            .await?
            .into_iter()
            .map(|record|(record.nonce as u64, serde_json::from_value(record.tx_data).unwrap()))
            .collect::<HashMap<u64, ZkLinkTx>>();

        metrics::histogram!(
            "sql.chain.operations.get_unprocessed_priority_tx_by_address",
            start.elapsed()
        );
        Ok(tx_data)
    }

    /// Retrieves all unprocessed priority transactions from the database.
    pub async fn get_unprocessed_priority_txs(&mut self, chain_id: i16) -> QueryResult<Vec<(u64, ZkLinkTx)>> {
        let start = Instant::now();

        let tx_data = sqlx::query!(
            "SELECT tx_data, nonce FROM submit_txs
            WHERE executed = false AND (op_type = $1 OR op_type = $2) AND chain_id = $3
            ORDER BY nonce ASC;
            ",
            DepositOp::OP_CODE as i16, FullExitOp::OP_CODE as i16, chain_id
        )
            .fetch_all(self.0.conn())
            .await?
            .into_iter()
            .map(|record|(record.nonce as u64, serde_json::from_value(record.tx_data).unwrap()))
            .collect();

        metrics::histogram!(
            "sql.chain.operations.get_unprocessed_priority_txs",
            start.elapsed()
        );
        Ok(tx_data)
    }

    /// Retrieves priority transaction from the database given priority transaction serial id.
    pub async fn get_last_serial_id(&mut self, chain_id: i16) -> QueryResult<i64> {
        let start = Instant::now();

        let tx_data = sqlx::query!(
            "SELECT max(nonce) FROM submit_txs WHERE chain_id = $1 and (op_type = $2 or op_type = $3)",
            chain_id,
            DepositOp::OP_CODE as i16,
            FullExitOp::OP_CODE as i16
        )
            .fetch_one(self.0.conn())
            .await?
            .max
            .unwrap_or(-1);

        metrics::histogram!(
            "sql.chain.operations.get_tx_by_serial_id",
            start.elapsed()
        );
        Ok(tx_data)
    }

    /// Retrieves transaction from the database given tx_type
    pub async fn get_tx_history(
        &mut self,
        tx_type: ZkLinkTxType,
        address: &ZkLinkAddress,
        page_index: i64,
        count: i64,
    ) -> QueryResult<(i64, Vec<StoredSubmitTransaction>)> {
        let address = address.as_bytes();
        let total_num = match tx_type {
            ZkLinkTxType::Deposit => sqlx::query!(
                    r#"SELECT count(*) FROM submit_txs
                     WHERE op_type = $1 AND to_account = $2"#,
                    DepositOp::OP_CODE as i16,
                    address,
                )
                .fetch_one(self.0.conn())
                .await?
                .count
                .unwrap_or(0),
            ZkLinkTxType::Withdraw => sqlx::query!(
                    r#"SELECT count(*) FROM submit_txs
                     WHERE op_type = $1 AND from_account = $2"#,
                    WithdrawOp::OP_CODE as i16,
                    address,
                )
                .fetch_one(self.0.conn())
                .await?
                .count
                .unwrap_or(0),
            ZkLinkTxType::Transfer => sqlx::query!(
                    r#"SELECT count(*) FROM submit_txs
                     WHERE op_type = ANY($1) AND (from_account = $2 OR to_account = $2)"#,
                    &[TransferOp::OP_CODE as i16, TransferToNewOp::OP_CODE as i16],
                    address,
                )
                .fetch_one(self.0.conn())
                .await?
                .count
                .unwrap_or(0),
            _ => 0
        };
        // no result
        if total_num == 0 {
            return Ok((0, vec![]));
        }
        // total page num, for example:
        // total_num = 5, page_size = 3, total_page_num = 2
        // total_num = 6, page_zie = 3, total_page_num = 2
        let mut total_page_num = total_num / count;
        if total_num % count != 0 {
            total_page_num += 1;
        }
        // page_index should be [0, total_page_num - 1]
        if page_index >= total_page_num {
            return Ok((total_page_num, vec![]));
        }
        let offset = page_index * count;
        let ops = match tx_type {
            ZkLinkTxType::Deposit => sqlx::query_as!(
                    StoredSubmitTransaction,
                    r#"SELECT a.* FROM submit_txs a INNER JOIN
                    (SELECT id FROM submit_txs
                     WHERE op_type = $1 AND to_account = $2
                     ORDER BY id DESC OFFSET $3 LIMIT $4) b
                     ON a.id = b.id"#,
                    DepositOp::OP_CODE as i16,
                    address,
                    offset,
                    count,
                )
                .fetch_all(self.0.conn())
                .await?,
            ZkLinkTxType::Withdraw => sqlx::query_as!(
                    StoredSubmitTransaction,
                    r#"SELECT a.* FROM submit_txs a INNER JOIN
                     (SELECT id FROM submit_txs
                     WHERE op_type = $1 AND from_account = $2
                     ORDER BY id DESC OFFSET $3 LIMIT $4) b
                     ON a.id = b.id"#,
                    WithdrawOp::OP_CODE as i16,
                    address,
                    offset,
                    count,
                )
                .fetch_all(self.0.conn())
                .await?,
            ZkLinkTxType::Transfer => sqlx::query_as!(
                    StoredSubmitTransaction,
                    r#"SELECT a.* FROM submit_txs a INNER JOIN
                     (SELECT id FROM submit_txs
                     WHERE op_type = ANY($1) AND (from_account = $2 OR to_account = $2)
                     ORDER BY id DESC OFFSET $3 LIMIT $4) b
                     ON a.id = b.id"#,
                    &[TransferOp::OP_CODE as i16, TransferToNewOp::OP_CODE as i16],
                    address,
                    offset,
                    count,
                )
                .fetch_all(self.0.conn())
                .await?,
            _ => vec![]
        };

        Ok((total_page_num, ops))
    }

    pub async fn confirm_aggregated_operations(
        &mut self,
        op_ids: Vec<i64>,
    ) -> QueryResult<()> {
        sqlx::query!(
            "UPDATE aggregate_operations SET confirmed = true WHERE id = ANY($1)",
            &op_ids
        )
            .execute(self.0.conn())
            .await?;

        Ok(())
    }

    /// Stores the executed transaction in the database.
    pub(crate) async fn store_executed_tx(
        &mut self,
        operation: NewExecutedTransaction,
    ) -> QueryResult<()> {
        let start = Instant::now();
        sqlx::query!(
            r#"UPDATE submit_txs SET block_number = $1, block_index = $2, operation = $3, executed = true,
            executed_timestamp = current_timestamp, success = $4, fail_reason = $5, nonce = $6, amount=$7
            WHERE tx_hash = $8"#,
            operation.block_number,
            operation.block_index,
            operation.operation,
            operation.success,
            operation.fail_reason,
            operation.nonce,
            operation.amount,
            operation.tx_hash,
        )
            .execute(self.0.conn())
            .await?;
        metrics::histogram!("sql.chain.operations.store_executed_tx", start.elapsed());
        Ok(())
    }

    /// Update the priority executed transaction in the database.
    pub(crate) async fn update_executed_tx(
        &mut self,
        operation: NewExecutedTransaction,
    ) -> QueryResult<()> {
        let start = Instant::now();
        sqlx::query!(
            r#"UPDATE submit_txs SET block_number = $1, block_index = $2, operation = $3,
            executed = true, executed_timestamp = current_timestamp, success = true
            WHERE chain_id = $4 AND op_type=$5 AND nonce=$6"#,
            operation.block_number,
            operation.block_index,
            operation.operation,
            operation.chain_id,
            operation.op_type,
            operation.nonce,
        )
            .execute(self.0.conn())
            .await?;
        metrics::histogram!("sql.chain.operations.store_executed_tx", start.elapsed());
        Ok(())
    }

    pub async fn store_aggregated_action(
        &mut self,
        operation: &StoredAggregatedOperation,
    ) -> QueryResult<i64> {
        let op_id = sqlx::query!(
            "INSERT INTO aggregate_operations (action_type, from_block, to_block, created_at, confirmed)
            VALUES ($1, $2, $3, $4, $5) RETURNING id",
            operation.action_type as AggType,
            operation.from_block,
            operation.to_block,
            operation.created_at,
            operation.confirmed
        )
            .fetch_one(self.0.conn())
            .await?
            .id;

        Ok(op_id as i64)
    }

    pub async fn store_aggregate_op_and_eth_op(
        &mut self,
        agg_op: StoredAggregatedOperation,
        raw_txs_with_chains: HashMap<ChainId, Value>,
        gas_limit: i32,
    ) -> QueryResult<()> {
        let mut transaction = self.0.start_transaction().await?;

        let op_id= transaction.chain()
            .operations_schema()
            .store_aggregated_action(&agg_op)
            .await?;

        // Generate eth_operations in db
        for (chain_id, raw_tx) in raw_txs_with_chains.into_iter() {
            // Obtain the operation ID for the follow-up queried.
            let eth_op_id = sqlx::query!(
                "INSERT INTO eth_operations
                (op_type, chain_id, sent, confirmed,last_deadline_block, last_used_gas_price, raw_tx, gas_limit)
                VALUES ($1, $2, false, false, 0, 0, $3, $4)
                RETURNING id
                ",
                agg_op.action_type as AggType, *chain_id as i64, raw_tx, gas_limit
            )
                .fetch_one(transaction.conn())
                .await?
                .id;

            sqlx::query!(
                "INSERT INTO eth_aggregated_ops_binding (op_id, eth_op_id) VALUES ($1, $2)",
                op_id,
                eth_op_id
            )
                .execute(transaction.conn())
                .await?;

        }

        transaction.commit().await?;

        Ok(())
    }

    pub async fn get_last_affected_block_by_aggregated_action(
        &mut self,
        aggregated_action: AggType,
    ) -> QueryResult<BlockNumber> {
        let block_number = sqlx::query!(
            "SELECT max(to_block) from aggregate_operations where action_type = $1",
            aggregated_action as AggType,
        )
        .fetch_one(self.0.conn())
        .await?
        .max
        .map(|b| BlockNumber(b as u32))
        .unwrap_or_default();
        Ok(block_number)
    }

    pub async fn get_last_confirmed_block_by_aggregated_action(
        &mut self,
        aggregated_action: AggType,
    ) -> QueryResult<BlockNumber> {
        let block_number = sqlx::query!(
            "SELECT max(to_block) from aggregate_operations where action_type = $1 and confirmed = true",
            aggregated_action as AggType,
        )
            .fetch_one(self.0.conn())
            .await?
            .max
            .map(|b| BlockNumber(b as u32))
            .unwrap_or_default();
        Ok(block_number)
    }

    pub async fn get_aggregated_op_that_affects_block(
        &mut self,
        aggregated_action: AggType,
        block_number: BlockNumber,
    ) -> QueryResult<Option<(i64,i64)>> {
        let aggregated_op = sqlx::query_as!(
            StoredAggregatedOperation,
            r#"SELECT id,action_type as "action_type:AggType",from_block,to_block,created_at,confirmed
             FROM aggregate_operations WHERE action_type = $1 AND from_block <= $2
             AND $2 <= to_block"#,
            aggregated_action as AggType,
            i64::from(*block_number)
        )
        .fetch_optional(self.0.conn())
        .await?
        .map(|op| {
            (op.from_block,op.to_block)
        });
        Ok(aggregated_op)
    }

    pub async fn get_not_confirmed_aggregated_op(
        &mut self,
        agg_type: AggType,
        id_offset: i64,
    ) -> QueryResult<Option<StoredAggregatedOperation>> {
        let aggregated_op = sqlx::query_as!(
            StoredAggregatedOperation,
            r#"SELECT id,action_type as "action_type:AggType",from_block,to_block,created_at,confirmed
             FROM aggregate_operations WHERE action_type = $1 AND confirmed = false
             AND id > $2 ORDER BY id ASC LIMIT 1"#,
            agg_type as AggType,
            id_offset
        )
            .fetch_optional(self.0.conn())
            .await?;
        Ok(aggregated_op)
    }

    pub async fn count_sent_unconfirmed_eth_ops(
        &mut self,
        chain_id: i16,
        op_type: AggType
    ) -> QueryResult<i64> {
        let record = sqlx::query!(
            r#"SELECT count(*)
               FROM eth_operations
               WHERE chain_id = $1 AND sent = true AND confirmed = false AND op_type = $2"#,
            chain_id,
            op_type as AggType
        )
            .fetch_one(self.0.conn())
            .await?;
        Ok(record.count.unwrap_or_default())
    }

    pub async fn update_nonce_and_raw_tx_of_eth_op(
        &mut self,
        eth_op_id: i64,
        nonce: i64,
        raw_tx: Value,
    ) -> QueryResult<()> {
        sqlx::query!(
            r#"UPDATE eth_operations SET nonce = $1, raw_tx = $2
               WHERE id = $3"#,
            nonce,
            raw_tx,
            eth_op_id
        )
            .execute(self.0.conn())
            .await?;
        Ok(())
    }

    pub async fn get_tx_state_updates(
        &mut self,
        block_number:i64,
        tx_hash: &[u8],
    ) -> QueryResult<StorageStateUpdates> {
        // no need to do update query in a transaction, because all updates
        // will be write to database in a transaction
        let account_creates = self
            .get_account_creates_by_block_tx(block_number, tx_hash)
            .await?;
        let balance_updates = self
            .get_account_balance_updates_by_block_tx(block_number, tx_hash)
            .await?;
        let order_nonce_updates = self
            .get_account_order_updates_by_block_tx(block_number, tx_hash)
            .await?;
        let account_pubkey_updates = self
            .get_account_pubkey_updates_by_block_tx(block_number, tx_hash)
            .await?;
        Ok(StorageStateUpdates{
            account_creates,
            balance_updates,
            order_nonce_updates,
            account_pubkey_updates
        })
    }

    pub async fn get_account_creates_by_block_tx(
        &mut self,
        block_number:i64,
        tx_hash: &[u8],
    ) -> QueryResult<Vec<StorageAccountCreation>> {
        let updates = sqlx::query_as!(
            StorageAccountCreation,
            r#"
                SELECT * FROM account_creates
                WHERE block_number=$1 AND tx_hash=$2
                ORDER BY account_id ASC
            "#,
            block_number,
            tx_hash,
        )
        .fetch_all(self.0.conn())
        .await?;

        Ok(updates)
    }

    pub async fn get_account_balance_updates_by_block_tx(
        &mut self,
        block_number:i64,
        tx_hash: &[u8],
    ) -> QueryResult<Vec<StorageAccountUpdate>> {
        let updates = sqlx::query_as!(
            StorageAccountUpdate,
            r#"
                SELECT * FROM account_balance_updates
                WHERE block_number=$1 AND tx_hash=$2
                ORDER BY balance_update_id ASC
            "#,
            block_number,
            tx_hash,
        )
        .fetch_all(self.0.conn())
        .await?;

        Ok(updates)
    }

    pub async fn get_account_order_updates_by_block_tx(
        &mut self,
        block_number:i64,
        tx_hash: &[u8],
    ) -> QueryResult<Vec<StorageAccountOrderUpdate>> {
        let updates = sqlx::query_as!(
            StorageAccountOrderUpdate,
            r#"
                SELECT * FROM account_order_updates
                WHERE block_number=$1 AND tx_hash=$2
                ORDER BY order_nonce_update_id ASC
            "#,
            block_number,
            tx_hash,
        )
        .fetch_all(self.0.conn())
        .await?;

        Ok(updates)
    }

    pub async fn get_account_pubkey_updates_by_block_tx(
        &mut self,
        block_number:i64,
        tx_hash: &[u8],
    ) -> QueryResult<Vec<StorageAccountPubkeyUpdate>> {
        let updates = sqlx::query_as!(
            StorageAccountPubkeyUpdate,
            r#"
                SELECT * FROM account_pubkey_updates
                WHERE block_number=$1 AND tx_hash=$2
                ORDER BY pubkey_update_id ASC
            "#,
            block_number,
            tx_hash,
        )
        .fetch_all(self.0.conn())
        .await?;

        Ok(updates)
    }
}
