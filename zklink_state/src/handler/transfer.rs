use anyhow::{ensure, format_err};
use zklink_types::{Account, AccountUpdate, AccountUpdates, Transfer, TransferOp, TransferToNewOp};

use crate::{
    handler::TxHandler,
    state::{TransferOutcome, ZkLinkState},
};

impl TxHandler<Transfer> for ZkLinkState {
    type Op = TransferOutcome;

    fn create_op(&self, tx: Transfer) -> Result<Self::Op, anyhow::Error> {
        // All stateless checking(tx format, l1 signature) should have been done by rpc
        // and we still redo stateful(which could be changed, eg. balance, nonce, pub_key_hash) checking when execute l2 transaction

        // Check transfer token
        self.ensure_token_supported(&tx.token)?;

        // Check from account
        let pk = tx
            .verify_signature()
            .ok_or(format_err!("Invalid l2 signature"))?;
        self.ensure_account_active_and_tx_pk_correct(tx.account_id, pk)?;

        let outcome = if let Some((to, _)) = self.get_account_by_address(&tx.to) {
            let transfer_op = TransferOp {
                from: tx.account_id,
                tx,
                to,
            };

            TransferOutcome::Transfer(transfer_op)
        } else {
            let to = self.get_free_account_id();
            let transfer_to_new_op = TransferToNewOp {
                from: tx.account_id,
                tx,
                to,
            };

            TransferOutcome::TransferToNew(transfer_to_new_op)
        };

        Ok(outcome)
    }

    fn apply_op(&mut self, op: &mut Self::Op) -> Result<AccountUpdates, anyhow::Error> {
        match op {
            TransferOutcome::Transfer(transfer_op) => self.apply_transfer_op(transfer_op),
            TransferOutcome::TransferToNew(transfer_to_new_op) => {
                self.apply_transfer_to_new_op(transfer_to_new_op)
            }
        }
    }
}

impl ZkLinkState {
    pub fn apply_transfer_op(&mut self, op: &TransferOp) -> Result<AccountUpdates, anyhow::Error> {
        if op.from == op.to {
            return self.apply_transfer_op_to_self(op);
        }

        let mut updates = Vec::new();
        let mut from_account = self.get_account(op.from).unwrap();
        let mut to_account = self.get_account(op.to).unwrap();
        let from_real_token =
            Self::get_actual_token_by_sub_account(op.tx.from_sub_account_id, op.tx.token);
        let to_real_token =
            Self::get_actual_token_by_sub_account(op.tx.to_sub_account_id, op.tx.token);

        let from_old_balance = from_account.get_balance(from_real_token);
        let from_old_nonce = from_account.nonce;
        ensure!(op.tx.nonce == from_old_nonce, "Nonce does not match");
        ensure!(
            from_old_balance >= &op.tx.amount + &op.tx.fee,
            "Insufficient balance"
        );

        from_account.sub_balance(from_real_token, &(&op.tx.amount + &op.tx.fee));
        *from_account.nonce += 1;

        let from_new_balance = from_account.get_balance(from_real_token);
        let from_new_nonce = from_account.nonce;

        updates.push((
            op.from,
            AccountUpdate::UpdateBalance {
                balance_update: (
                    op.tx.token,
                    op.tx.from_sub_account_id,
                    from_old_balance,
                    from_new_balance,
                ),
                old_nonce: from_old_nonce,
                new_nonce: from_new_nonce,
            },
        ));

        let to_old_balance = to_account.get_balance(to_real_token);
        let to_account_nonce = to_account.nonce;

        to_account.add_balance(to_real_token, &op.tx.amount);

        let to_new_balance = to_account.get_balance(to_real_token);

        updates.push((
            op.to,
            AccountUpdate::UpdateBalance {
                balance_update: (
                    op.tx.token,
                    op.tx.to_sub_account_id,
                    to_old_balance,
                    to_new_balance,
                ),
                old_nonce: to_account_nonce,
                new_nonce: to_account_nonce,
            },
        ));

        self.insert_account(op.from, from_account);
        self.insert_account(op.to, to_account);
        self.collect_fee(op.tx.token, &op.tx.fee, &mut updates);

        Ok(updates)
    }

    fn apply_transfer_op_to_self(
        &mut self,
        op: &TransferOp,
    ) -> Result<AccountUpdates, anyhow::Error> {
        // We ensure from_sub_account_id and to_sub_account_id will be different after rpc check
        let mut updates = Vec::new();
        let mut account = self.get_account(op.from).unwrap();

        let from_real_token =
            Self::get_actual_token_by_sub_account(op.tx.from_sub_account_id, op.tx.token);
        let to_real_token =
            Self::get_actual_token_by_sub_account(op.tx.to_sub_account_id, op.tx.token);
        let old_balance = account.get_balance(from_real_token);
        let old_nonce = account.nonce;
        ensure!(op.tx.nonce == old_nonce, "Nonce mismatch");
        ensure!(
            old_balance >= &op.tx.amount + &op.tx.fee,
            "Transfer account is not enough balance"
        );

        account.sub_balance(from_real_token, &(&op.tx.amount + &op.tx.fee));
        *account.nonce += 1;
        let new_balance = account.get_balance(from_real_token);
        let new_nonce = account.nonce;

        updates.push((
            op.from,
            AccountUpdate::UpdateBalance {
                balance_update: (
                    op.tx.token,
                    op.tx.from_sub_account_id,
                    old_balance,
                    new_balance,
                ),
                old_nonce,
                new_nonce,
            },
        ));

        let old_balance = account.get_balance(to_real_token);
        account.add_balance(to_real_token, &op.tx.amount);
        let new_balance = account.get_balance(to_real_token);
        let new_nonce = account.nonce;
        updates.push((
            op.to,
            AccountUpdate::UpdateBalance {
                balance_update: (
                    op.tx.token,
                    op.tx.to_sub_account_id,
                    old_balance,
                    new_balance,
                ),
                old_nonce,
                new_nonce,
            },
        ));

        self.insert_account(op.from, account);
        self.collect_fee(op.tx.token, &op.tx.fee, &mut updates);

        Ok(updates)
    }

    fn apply_transfer_to_new_op(
        &mut self,
        op: &TransferToNewOp,
    ) -> Result<AccountUpdates, anyhow::Error> {
        let mut updates = Vec::new();

        let mut from_account = self.get_account(op.from).unwrap();
        let mut to_account = {
            let (acc, upd) = Account::create_account(op.to, op.tx.to.clone());
            updates.extend(upd.into_iter());
            acc
        };
        let from_real_token =
            Self::get_actual_token_by_sub_account(op.tx.from_sub_account_id, op.tx.token);
        let to_real_token =
            Self::get_actual_token_by_sub_account(op.tx.to_sub_account_id, op.tx.token);

        let from_old_balance = from_account.get_balance(from_real_token);
        let from_old_nonce = from_account.nonce;
        ensure!(op.tx.nonce == from_old_nonce, "Nonce does not match");
        ensure!(
            from_old_balance >= &op.tx.amount + &op.tx.fee,
            "Insufficient balance"
        );
        from_account.sub_balance(from_real_token, &(&op.tx.amount + &op.tx.fee));
        *from_account.nonce += 1;
        let from_new_balance = from_account.get_balance(from_real_token);
        let from_new_nonce = from_account.nonce;

        updates.push((
            op.from,
            AccountUpdate::UpdateBalance {
                balance_update: (
                    op.tx.token,
                    op.tx.from_sub_account_id,
                    from_old_balance,
                    from_new_balance,
                ),
                old_nonce: from_old_nonce,
                new_nonce: from_new_nonce,
            },
        ));

        let to_old_balance = to_account.get_balance(to_real_token);
        let to_account_nonce = to_account.nonce;
        to_account.add_balance(to_real_token, &op.tx.amount);
        let to_new_balance = to_account.get_balance(to_real_token);

        updates.push((
            op.to,
            AccountUpdate::UpdateBalance {
                balance_update: (
                    op.tx.token,
                    op.tx.to_sub_account_id,
                    to_old_balance,
                    to_new_balance,
                ),
                old_nonce: to_account_nonce,
                new_nonce: to_account_nonce,
            },
        ));

        self.insert_account(op.from, from_account);
        self.insert_account(op.to, to_account);
        // Collect transfer token as fee
        self.collect_fee(op.tx.token, &op.tx.fee, &mut updates);

        Ok(updates)
    }
}
