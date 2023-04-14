use super::PubKeyHash;
use super::{Nonce, TokenId};
use crate::{SlotId, SubAccountId, ZkLinkAddress};
use num::BigUint;
use serde::{Deserialize, Serialize};

/// Atomic change in the account state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AccountUpdate {
    /// Create a new account.
    Create {
        address: ZkLinkAddress,
        nonce: Nonce,
    },
    /// Delete an existing account.
    /// Note: Currently this kind of update is not used directly in the network.
    /// However, it's used to revert made operation (e.g. to restore state back in time from the last verified block).
    Delete {
        address: ZkLinkAddress,
        nonce: Nonce,
    },
    /// Change the account balance.
    UpdateBalance {
        old_nonce: Nonce,
        new_nonce: Nonce,
        /// Tuple of (token, old_balance, new_balance)
        balance_update: (TokenId, SubAccountId, BigUint, BigUint),
    },
    /// Change the account Public Key.
    ChangePubKeyHash {
        old_pub_key_hash: PubKeyHash,
        new_pub_key_hash: PubKeyHash,
        old_nonce: Nonce,
        new_nonce: Nonce,
    },
    /// Update order nonce and residue
    UpdateTidyOrder {
        slot_id: SlotId,
        sub_account_id: SubAccountId,
        old_order: (Nonce, BigUint),
        new_order: (Nonce, BigUint),
    },
}

impl AccountUpdate {
    /// Generates an account update to revert current update.
    pub fn reversed_update(&self) -> Self {
        match self {
            AccountUpdate::Create { address, nonce } => AccountUpdate::Delete {
                address: (*address).clone(),
                nonce: *nonce,
            },
            AccountUpdate::Delete { address, nonce } => AccountUpdate::Create {
                address: (*address).clone(),
                nonce: *nonce,
            },
            AccountUpdate::UpdateBalance {
                old_nonce,
                new_nonce,
                balance_update,
            } => AccountUpdate::UpdateBalance {
                old_nonce: *new_nonce,
                new_nonce: *old_nonce,
                balance_update: (
                    balance_update.0,
                    balance_update.1,
                    balance_update.3.clone(),
                    balance_update.2.clone(),
                ),
            },
            AccountUpdate::ChangePubKeyHash {
                old_pub_key_hash,
                new_pub_key_hash,
                old_nonce,
                new_nonce,
            } => AccountUpdate::ChangePubKeyHash {
                old_pub_key_hash: *new_pub_key_hash,
                new_pub_key_hash: *old_pub_key_hash,
                old_nonce: *new_nonce,
                new_nonce: *old_nonce,
            },
            AccountUpdate::UpdateTidyOrder {
                slot_id,
                sub_account_id,
                old_order,
                new_order,
            } => AccountUpdate::UpdateTidyOrder {
                slot_id: *slot_id,
                sub_account_id: *sub_account_id,
                old_order: new_order.clone(),
                new_order: old_order.clone(),
            },
        }
    }
}
