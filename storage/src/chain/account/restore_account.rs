// Built-in deps
// External imports
use num::bigint::ToBigInt;
// Workspace imports
use zklink_types::{Account, AccountId, Nonce, TokenId};
use zklink_types::{PubKeyHash, SlotId, SubAccountId, ZkLinkAddress};
// Local imports
use super::records::*;
use zklink_types::account::TidyOrder;
use zklink_types::utils::{calculate_actual_slot, calculate_actual_token};

pub(crate) fn restore_account(
    stored_account: &StorageAccount,
    stored_balances: Vec<StorageBalance>,
    stored_orders: Vec<StorageOrderNonce>,
) -> (AccountId, Account) {
    let mut account = Account::default();
    for b in stored_balances.into_iter() {
        assert_eq!(b.account_id, stored_account.id);
        let balance_bigint = b.balance.to_bigint().unwrap();
        let balance = balance_bigint.to_biguint().unwrap();
        let coin_id = calculate_actual_token(
            SubAccountId(b.sub_account_id as u8),
            TokenId(b.coin_id as u32),
        );
        account.set_balance(coin_id, balance);
    }

    for o in stored_orders.into_iter() {
        assert_eq!(o.account_id, stored_account.id);
        let slot_id = calculate_actual_slot(
            SubAccountId(o.sub_account_id as u8),
            SlotId(o.slot_id as u32),
        );
        let residue = o.residue.to_bigint().unwrap();
        let order = TidyOrder {
            nonce: Nonce(o.order_nonce as u32),
            residue: residue.into(),
        };
        account.order_slots.insert(slot_id, order);
    }
    account.nonce = Nonce(stored_account.nonce as u32);
    account.address = ZkLinkAddress::from_slice(&stored_account.address).unwrap();
    account.pub_key_hash = PubKeyHash::from_bytes(&stored_account.pubkey_hash)
        .expect("db stored pubkey hash deserialize");
    (AccountId(stored_account.id as u32), account)
}
