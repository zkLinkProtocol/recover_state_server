// External imports
use chrono::prelude::*;
use num::bigint::ToBigInt;
use serde_json::value::Value;
use sqlx::FromRow;
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use zklink_basic_types::ChainId;
use zklink_types::{Deposit, FullExit, PriorityDeposit, PriorityFullExit, ZkLinkTx};
// Local imports
use crate::StorageActionType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "agg_type")]
#[derive(Default)]
pub enum AggType {
    #[default]
    CommitBlocks,
    CreateProofBlocks,
    PublishProofBlocksOnchain,
    BridgeBlocks,
    SyncBlocks,
    ExecuteBlocks,
}



#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone, FromRow)]
#[derive(Default)]
pub struct StoredSubmitTransaction {
    pub id: i64,
    pub chain_id: i16,
    pub op_type: i16,
    pub from_account: Vec<u8>,
    pub to_account: Vec<u8>,
    pub nonce: i64,
    pub amount: BigDecimal,
    pub tx_data: Value,
    pub eth_signature: Option<Value>,
    pub tx_hash: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub executed: bool,
    pub executed_timestamp: Option<DateTime<Utc>>,
    pub success: bool,
    pub fail_reason: Option<String>,
    pub block_number: i64,
    pub block_index: i32,
    pub operation: Option<Value>,
}

impl From<&PriorityDeposit> for StoredSubmitTransaction{
    fn from(deposit: &PriorityDeposit) -> Self {
        let tx = Deposit::new(
            ChainId(deposit.chain_id),
            deposit.from.clone(),
            deposit.sub_account_id,
            deposit.to.clone(),
            deposit.l2_target_token,
            deposit.l1_source_token,
            deposit.amount.clone(),
            deposit.serial_id,
            deposit.tx_hash,
        );
        let amount = tx.amount.to_bigint().unwrap().into();
        let zklink_tx = ZkLinkTx::from(tx);
        let hash = zklink_tx.hash().as_ref().to_vec();
        let tx_value = serde_json::to_value(zklink_tx).unwrap();
        let created_at = Utc::now();

        StoredSubmitTransaction{
            chain_id: deposit.chain_id as i16,
            op_type: Deposit::TX_TYPE as i16,
            from_account: deposit.from.as_bytes().to_vec(),
            to_account: deposit.to.as_bytes().to_vec(),
            nonce: deposit.serial_id as i64,
            amount,
            tx_data: tx_value,
            tx_hash: hash,
            created_at,
            ..Default::default()
        }
    }
}

impl From<&PriorityFullExit> for StoredSubmitTransaction{
    fn from(full_exit: &PriorityFullExit) -> Self {
        let tx = FullExit::new(
            full_exit.chain_id,
            full_exit.account_id,
            full_exit.sub_account_id,
            full_exit.exit_address.clone(),
            full_exit.l2_source_token,
            full_exit.l1_target_token,
            full_exit.serial_id,
            full_exit.tx_hash,
        );
        let zklink_tx = ZkLinkTx::from(tx);
        let hash = zklink_tx.hash().as_ref().to_vec();
        let tx_value = serde_json::to_value(zklink_tx).unwrap();
        let created_at = Utc::now();

        StoredSubmitTransaction{
            chain_id: full_exit.chain_id as i16,
            op_type: FullExit::TX_TYPE as i16,
            from_account: full_exit.initiator.as_bytes().to_vec(),
            to_account: full_exit.exit_address.as_bytes().to_vec(),
            nonce: full_exit.serial_id as i64,
            tx_data: tx_value,
            tx_hash: hash,
            created_at,
            ..Default::default()
        }
    }
}



#[derive(Debug, Clone, FromRow)]
pub struct StoredOperation {
    pub id: i64,
    pub block_number: i64,
    pub action_type: StorageActionType,
    pub created_at: DateTime<Utc>,
    pub confirmed: bool,
}

#[derive(Debug, Clone, FromRow)]
pub struct StoredExecutedPriorityOperation {
    pub block_number: i64,
    pub block_index: i32,
    pub operation: Value,
    pub from_account: Vec<u8>,
    pub to_account: Vec<u8>,
    pub priority_op_serialid: i64,
    pub deadline_block: i64,
    pub eth_hash: Vec<u8>,
    pub eth_block: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone, FromRow)]
pub struct StoredExecutedTransaction {
    pub block_number: i64,
    pub block_index: i32,
    pub tx_data: Value,
    pub operation: Option<Value>,
    pub tx_hash: Vec<u8>,
    pub from_account: Vec<u8>,
    pub to_account: Vec<u8>,
    pub success: bool,
    pub fail_reason: Option<String>,
    pub nonce: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewOperation {
    pub block_number: i64,
    pub action_type: StorageActionType,
}

#[derive(Debug, Clone)]
pub struct NewExecutedPriorityOperation {
    pub block_number: i64,
    pub block_index: i32,
    pub operation: Value,
    pub from_account: Vec<u8>,
    pub to_account: Vec<u8>,
    pub priority_op_serialid: i64,
    pub deadline_block: i64,
    pub eth_hash: Vec<u8>,
    pub eth_block: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewExecutedTransaction {
    pub op_type: i16,
    pub chain_id: i16,
    pub block_number: i64,
    pub block_index: Option<i32>,
    pub operation: Value,
    pub tx_hash: Vec<u8>,
    pub success: bool,
    pub fail_reason: Option<String>,
    pub amount: BigDecimal,
    pub nonce: i64,
}

#[derive(Debug, Clone)]
pub struct StoredPendingWithdrawal {
    pub id: i64,
    pub withdrawal_hash: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct StoredCompleteWithdrawalsTransaction {
    pub tx_hash: Vec<u8>,
    pub pending_withdrawals_queue_start_index: i64,
    pub pending_withdrawals_queue_end_index: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct StoredAggregatedOperation {
    pub id: i64,
    pub action_type: AggType,
    pub from_block: i64,
    pub to_block: i64,
    pub created_at: DateTime<Utc>,
    pub confirmed: bool,
}

#[derive(Debug, Clone, FromRow)]
pub struct StoredAggregatedOperationId {
    pub id: i64,
}
#[derive(Debug, Clone, FromRow)]
pub struct StoredAggregatedOperationConfirmed {
    pub confirmed: bool,
}


#[derive(Debug, Clone, FromRow)]
pub struct StoredConfirmMask {
    pub id: bool,
    pub mask: Vec<bool>
}

#[derive(Debug, FromRow)]
pub struct StorageTxHash {
    pub tx_hash: Vec<u8>,
}

#[derive(Debug, FromRow)]
pub struct StorageTxHashData {
    pub tx_hash: Vec<u8>,
    pub tx_data: Value
}

#[derive(Debug, Clone, FromRow)]
pub struct StoredOnChainTx {
    pub chain_id: i16,
    pub final_hash: Option<Vec<u8>>,
}