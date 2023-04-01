use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use chrono::{DateTime, Utc};
// External imports
use serde::{Deserialize, Serialize};
use sqlx::{types::BigDecimal, FromRow };
use serde_json::value::Value;

#[derive(Debug, Clone, FromRow)]
pub struct StorageAccount {
    pub id: i64,
    pub nonce: i64,
    pub address: Vec<u8>,
    pub pubkey_hash: Vec<u8>,
    pub account_type: AccountType,
    pub chain_id: i16,
    pub last_block: i64,
}

#[derive(Debug, FromRow)]
pub struct StorageAccountCreation {
    pub account_id: i64,
    pub address: Vec<u8>,
    pub block_number: i64,
    pub update_order_id: i32,
    pub tx_hash: Vec<u8>
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, FromRow)]
pub struct StorageAccountUpdate {
    pub balance_update_id: i64,
    pub account_id: i64,
    pub sub_account_id: i32,
    pub coin_id: i32,
    pub old_balance: BigDecimal,
    pub new_balance: BigDecimal,
    pub old_nonce: i64,
    pub new_nonce: i64,
    pub block_number: i64,
    pub update_order_id: i32,
    pub tx_hash: Vec<u8>
}

#[derive(Debug, FromRow)]
pub struct StorageAccountPubkeyUpdate {
    pub pubkey_update_id: i32,
    pub account_id: i64,
    pub old_pubkey_hash: Vec<u8>,
    pub new_pubkey_hash: Vec<u8>,
    pub old_nonce: i64,
    pub new_nonce: i64,
    pub block_number: i64,
    pub update_order_id: i32,
    pub tx_hash: Vec<u8>
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, FromRow)]
pub struct StorageAccountOrderUpdate {
    pub order_nonce_update_id: i64,
    pub account_id: i64,
    pub sub_account_id: i32,
    pub slot_id: i32,
    pub old_order_nonce: Value,
    pub new_order_nonce: Value,
    pub block_number: i64,
    pub update_order_id: i32,
    pub tx_hash: Vec<u8>
}

#[derive(Debug, Default)]
pub struct StorageStateUpdates {
    pub account_creates: Vec<StorageAccountCreation>,
    pub balance_updates: Vec<StorageAccountUpdate>,
    pub order_nonce_updates: Vec<StorageAccountOrderUpdate>,
    pub account_pubkey_updates: Vec<StorageAccountPubkeyUpdate>
}

impl StorageStateUpdates {
    pub fn group_by_tx_hash(self) -> HashMap<Vec<u8>, StorageStateUpdates> {
        let mut groups: HashMap<Vec<u8>, StorageStateUpdates>= HashMap::new();
        for u in self.account_creates {
            groups
                .entry(u.tx_hash.clone())
                .or_default()
                .account_creates
                .push(u);
        }
        for u in self.balance_updates {
            groups
                .entry(u.tx_hash.clone())
                .or_default()
                .balance_updates
                .push(u);
        }
        for u in self.order_nonce_updates {
            groups
                .entry(u.tx_hash.clone())
                .or_default()
                .order_nonce_updates
                .push(u);
        }
        for u in self.account_pubkey_updates {
            groups
                .entry(u.tx_hash.clone())
                .or_default()
                .account_pubkey_updates
                .push(u);
        }
        groups
    }
}

#[derive(Debug, FromRow, Clone)]
pub struct StorageOriginalBalance {
    pub account_id: i64,
    pub coin_id: Option<i32>,
    pub balance: BigDecimal,
}

#[derive(Debug, FromRow, Clone)]
pub struct StorageBalance {
    pub account_id: i64,
    pub sub_account_id: i32,
    pub coin_id: i32,
    pub balance: BigDecimal,
}


#[derive(Debug, FromRow, Clone)]
pub struct StorageOrderNonce {
    pub account_id: i64,
    pub sub_account_id: i32,
    pub slot_id: i32,
    pub order_nonce: i64,
    pub residue: BigDecimal,
}

#[derive(Debug, FromRow, Clone)]
pub struct StorageOriginalOrderNonce {
    pub account_id: i64,
    pub slot_id: Option<i32>,
    pub order_nonce: i64,
    pub residue: BigDecimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "layer1_account_type")]
pub enum AccountType {
    Unknown,
    EthOwned, // check l1 signature by ECDSA off chain
    EthCREATE2, // check l1 signature by EIP1271 on chain
    StarkContract
}

impl Display for AccountType{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct StorageAccountType {
    pub account_id: i64,
    pub account_type: AccountType,
}

#[derive(Debug)]
pub struct StorageAccountState {
    pub account: Option<StorageAccount>,
    pub balances: Vec<StorageBalance>,
    pub orders: Vec<StorageOrderNonce>
}

#[derive(Debug, FromRow)]
pub struct StorageWhiteSubmitter {
    pub id: i64,
    pub sub_account_id: i32,
    pub submitter_account_id: i64,
}

/// Wrapper for date and time of the first executed transaction
/// for the account.
#[derive(Debug, Serialize, Deserialize, FromRow, PartialEq)]
pub struct AccountCreatedAt {
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AccountSnapshot {
    pub account: Option<StorageAccount>,
    pub balances: Vec<StorageBalance>,
    pub order_slots: Vec<StorageOrderNonce>,
    pub block_number: i64,
}