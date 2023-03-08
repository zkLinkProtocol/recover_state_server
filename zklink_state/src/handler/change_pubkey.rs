use anyhow::{ensure, format_err};
use zklink_types::{operations::ChangePubKeyOp, tx::ChangePubKey, AccountUpdate, AccountUpdates};
use crate::{
    handler::TxHandler,
    state::ZkLinkState,
};

impl TxHandler<ChangePubKey> for ZkLinkState {
    type Op = ChangePubKeyOp;

    fn create_op(&self, tx: ChangePubKey) -> Result<Self::Op, anyhow::Error> {
        // All stateless checking(tx format, l1 signature) should have been done by rpc
        // and we still redo stateful(which could be changed, eg. balance, nonce, pub_key_hash) checking when execute l2 transaction

        // Check fee token
        self.ensure_token_supported(&tx.fee_token)?;

        let account = self
            .get_account(tx.account_id)
            .ok_or_else(|| format_err!("ChangePubKey account does not exist"))?;
        let change_pk_op = ChangePubKeyOp {
            account_id: tx.account_id,
            tx,
            address: account.address
        };
        Ok(change_pk_op)
    }

    fn apply_op(
        &mut self,
        op: &mut Self::Op,
    ) -> Result<AccountUpdates, anyhow::Error> {
        let mut updates = Vec::new();
        let mut account = self.get_account(op.account_id).unwrap();
        let old_pub_key_hash = account.pub_key_hash;
        let actual_fee_token  = Self::get_actual_token_by_sub_account(op.tx.sub_account_id, op.tx.fee_token);
        let old_balance = account.get_balance(actual_fee_token);
        let old_nonce = account.nonce;

        // Update nonce.
        ensure!(op.tx.nonce == account.nonce, "Nonce mismatch");
        *account.nonce += 1;

        // Update pubkey hash.
        account.pub_key_hash = op.tx.new_pk_hash;

        // Subtract fees.
        ensure!(old_balance >= op.tx.fee, "Not enough balance");
        account.sub_balance(actual_fee_token, &op.tx.fee);

        let new_pub_key_hash = account.pub_key_hash;
        let new_nonce = account.nonce;
        let new_balance = account.get_balance(actual_fee_token);

        self.insert_account(op.account_id, account);

        updates.push((
            op.account_id,
            AccountUpdate::ChangePubKeyHash {
                old_pub_key_hash,
                old_nonce,
                new_pub_key_hash,
                new_nonce,
            },
        ));

        updates.push((
            op.account_id,
            AccountUpdate::UpdateBalance {
                balance_update: (op.tx.fee_token, op.tx.sub_account_id, old_balance, new_balance),
                old_nonce: new_nonce,
                new_nonce,
            },
        ));

        self.collect_fee(op.tx.fee_token, &op.tx.fee, &mut updates);

        Ok(updates)
    }
}
