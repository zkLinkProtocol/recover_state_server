// Built-in deps
use std::cmp::Ordering;
// External imports
use num::bigint::ToBigInt;
use zklink_basic_types::H256;
// Workspace imports
use zklink_types::{PubKeyHash, SlotId, SubAccountId, TokenId, AccountId, AccountUpdate, Nonce, ZkLinkAddress};
// Local imports
use crate::chain::account::records::*;
use sqlx::types::BigDecimal;
/// `StorageAccoundDiff` is a enum that combines all the possible
/// changes that can be applied to account, which includes:
///
/// - Creation of the new account.
/// - Removing of the existing account.
/// - Changing balance of the account.
/// - Changing the public key of the account.
///
/// This enum allows one to process account updates in a generic way.
#[derive(Debug)]
pub enum StorageAccountDiff {
    BalanceUpdate(StorageAccountUpdate),
    Create(StorageAccountCreation),
    ChangePubKey(StorageAccountPubkeyUpdate),
    ChangeOrderNonce(StorageAccountOrderUpdate),
}

impl From<StorageAccountUpdate> for StorageAccountDiff {
    fn from(update: StorageAccountUpdate) -> Self {
        StorageAccountDiff::BalanceUpdate(update)
    }
}

impl From<StorageAccountCreation> for StorageAccountDiff {
    fn from(create: StorageAccountCreation) -> Self {
        StorageAccountDiff::Create(create)
    }
}

impl From<StorageAccountPubkeyUpdate> for StorageAccountDiff {
    fn from(update: StorageAccountPubkeyUpdate) -> Self {
        StorageAccountDiff::ChangePubKey(update)
    }
}

impl From<StorageAccountOrderUpdate> for StorageAccountDiff {
    fn from (update: StorageAccountOrderUpdate) -> Self {
        StorageAccountDiff::ChangeOrderNonce(update)
    }
}

impl Into<(AccountId, AccountUpdate)> for StorageAccountDiff {
    fn into(self) -> (AccountId, AccountUpdate) {
        match self {
            StorageAccountDiff::BalanceUpdate(upd) => {
                let old_balance = upd.old_balance.to_bigint().unwrap();
                let old_balance = old_balance.to_biguint().unwrap();

                let new_balance = upd.new_balance.to_bigint().unwrap();
                let new_balance = new_balance.to_biguint().unwrap();

                (
                    AccountId(upd.account_id as u32),
                    AccountUpdate::UpdateBalance {
                        old_nonce: Nonce(upd.old_nonce as u32),
                        new_nonce: Nonce(upd.new_nonce as u32),
                        balance_update: (
                            TokenId(upd.coin_id as u32),
                            SubAccountId(upd.sub_account_id as u8),
                            old_balance,
                            new_balance),
                    },
                )
            },
            StorageAccountDiff::Create(upd) => (
                AccountId(upd.account_id as u32),
                AccountUpdate::Create {
                    nonce: Nonce(0u32),
                    address: ZkLinkAddress::from_slice(&upd.address.as_slice()).unwrap(),
                },
            ),
            StorageAccountDiff::ChangePubKey(upd) => (
                AccountId(upd.account_id as u32),
                AccountUpdate::ChangePubKeyHash {
                    old_nonce: Nonce(upd.old_nonce as u32),
                    new_nonce: Nonce(upd.new_nonce as u32),
                    old_pub_key_hash: PubKeyHash::from_bytes(&upd.old_pubkey_hash)
                        .expect("PubkeyHash update from db deserialize"),
                    new_pub_key_hash: PubKeyHash::from_bytes(&upd.new_pubkey_hash)
                        .expect("PubkeyHash update from db deserialize"),
                },
            ),
            StorageAccountDiff::ChangeOrderNonce(upd) => {
                let order_info_old: (i64, BigDecimal, H256) = serde_json::from_str(upd.old_order_nonce.as_str().unwrap()).unwrap();
                let order_info_new: (i64, BigDecimal, H256) = serde_json::from_str(upd.new_order_nonce.as_str().unwrap()).unwrap();
                (
                    AccountId(upd.account_id as u32),
                    AccountUpdate::UpdateTidyOrder {
                        slot_id: SlotId(upd.slot_id as u32),
                        sub_account_id: SubAccountId(upd.sub_account_id as u8),
                        old_order: (Nonce(order_info_old.0 as u32), order_info_old.1.to_bigint().unwrap().to_biguint().unwrap()),
                        new_order: (Nonce(order_info_new.0 as u32), order_info_new.1.to_bigint().unwrap().to_biguint().unwrap()),
                    }
                )

            }
        }
    }
}

    impl StorageAccountDiff {
        /// Compares updates by `block number` then by `update_order_id` (which is number within block).
        pub fn cmp_order(&self, other: &Self) -> Ordering {
            self.block_number()
                .cmp(&other.block_number())
                .then(self.update_order_id().cmp(&other.update_order_id()))
        }

        /// Returns the index of the operation within block.
        pub fn update_order_id(&self) -> i32 {
            match self {
                StorageAccountDiff::BalanceUpdate(StorageAccountUpdate {
                                                      update_order_id, ..
                                                  }) => *update_order_id,
                StorageAccountDiff::Create(StorageAccountCreation {
                                               update_order_id, ..
                                           }) => *update_order_id,
                StorageAccountDiff::ChangePubKey(StorageAccountPubkeyUpdate {
                                                     update_order_id,
                                                     ..
                                                 }) => *update_order_id,
                StorageAccountDiff::ChangeOrderNonce(StorageAccountOrderUpdate {
                                                     update_order_id,
                                                     ..
                                                 }) => *update_order_id,
            }
        }

        /// Returns the block index to which the operation belongs.
        pub fn block_number(&self) -> i64 {
            *match self {
                StorageAccountDiff::BalanceUpdate(StorageAccountUpdate { block_number, .. }) => {
                    block_number
                }
                StorageAccountDiff::Create(StorageAccountCreation { block_number, .. }) => block_number,
                StorageAccountDiff::ChangePubKey(StorageAccountPubkeyUpdate {
                                                     block_number, ..
                                                 }) => block_number,
                StorageAccountDiff::ChangeOrderNonce(StorageAccountOrderUpdate {
                                                     block_number, ..
                                                 }) => block_number,
            }
        }
    }
