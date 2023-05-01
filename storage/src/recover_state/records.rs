use chrono::{DateTime, Utc};
// External imports
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
// Workspace imports
use zklink_types::{AccountId, BlockNumber, ZkLinkOp, H256};
// Workspace imports
// Local imports

#[derive(Debug)]
pub struct NewRollupOpsBlock<'a> {
    pub block_num: BlockNumber,
    pub ops: &'a [ZkLinkOp],
    pub fee_account: AccountId,
    pub timestamp: Option<DateTime<Utc>>,
    pub previous_block_root_hash: H256,
    pub contract_version: i16,
}

#[derive(Debug, Clone, FromRow)]
pub struct StoredRollupOpsBlock {
    pub block_num: i64,
    pub operation: Value,
    pub fee_account: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub previous_block_root_hash: Vec<u8>,
    pub contract_version: i16,
}

#[derive(Debug)]
pub struct NewStorageState {
    pub storage_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StoredStorageState {
    pub id: i32,
    pub storage_state: String,
}

#[derive(Debug)]
pub struct NewBlockEvent {
    pub block_type: String, // 'Committed', 'Verified'
    pub transaction_hash: Vec<u8>,
    pub start_block_num: i64,
    pub end_block_num: i64,
    pub contract_version: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StoredBlockEvent {
    pub id: i32,
    pub block_type: String, // 'Committed', 'Verified'
    pub transaction_hash: Vec<u8>,
    pub start_block_num: i64,
    pub end_block_num: i64,
    pub contract_version: i16,
}
