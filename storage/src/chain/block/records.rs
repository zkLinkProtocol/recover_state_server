// External imports
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::value::Value;
use sqlx::FromRow;
// Workspace imports
use crate::chain::operations::records::StoredOnChainTx;
use zklink_utils::{BytesToHexSerde, OptionBytesToHexSerde, SyncBlockPrefix, ZeroxPrefix};
// Local imports

#[derive(Debug, FromRow)]
pub struct StorageBlock {
    pub number: i64,
    pub root_hash: Vec<u8>,
    pub fee_account_id: i64,
    pub block_size: i64,
    pub ops_composition_number: i64,
    pub created_at: DateTime<Utc>,
    pub commitment: Vec<u8>,
    pub sync_hash: Vec<u8>,
    pub commit_gas_limit: i64,
    pub verify_gas_limit: i64,
}

#[derive(Clone, Debug)]
pub struct StorageBlockState {
    pub last_block_number: i64,
    pub created_at: DateTime<Utc>,
    pub committed: i64,
    pub verified: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow, PartialEq, Clone)]
pub struct BlockDetails {
    pub block_number: i64,

    #[serde(with = "BytesToHexSerde::<SyncBlockPrefix>")]
    pub new_state_root: Vec<u8>,

    pub block_size: i64,

    #[serde(with = "OptionBytesToHexSerde::<ZeroxPrefix>")]
    pub commit_tx_hash: Option<Vec<u8>>,

    #[serde(with = "OptionBytesToHexSerde::<ZeroxPrefix>")]
    pub verify_tx_hash: Option<Vec<u8>>,

    pub committed_at: DateTime<Utc>,

    pub verified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, PartialEq)]
pub struct BlockTransactionItem {
    pub tx_hash: String,
    pub block_number: i64,
    pub op: Value,
    pub success: Option<bool>,
    pub fail_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountTreeCache {
    pub block: i64,
    pub tree_cache: String,
}

#[derive(Debug, FromRow)]
pub struct NewZkLinkTx {
    pub id: i64,
    pub tx_data: Value,
}

impl BlockDetails {
    /// Checks if block is finalized, meaning that
    /// both Verify operation is performed for it, and this
    /// operation is anchored on the Ethereum blockchain.
    pub fn is_verified(&self) -> bool {
        // We assume that it's not possible to have block that is
        // verified and not committed.
        self.verified_at.is_some() && self.verify_tx_hash.is_some()
    }
}

#[derive(Clone, Debug)]
pub struct StorageBlockOnChainState {
    pub committed: Vec<StoredOnChainTx>,
    pub verified: Vec<StoredOnChainTx>,
}
