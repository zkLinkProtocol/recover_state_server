use anyhow::{ensure, format_err};
use zklink_crypto::params::{GLOBAL_ASSET_ACCOUNT_ID};
use zklink_types::{AccountUpdate, AccountUpdates, Withdraw, WithdrawOp, Nonce, SubAccountId, ZkLinkTx};
use crate::{
    handler::TxHandler, state::ZkLinkState,
};

impl TxHandler<Withdraw> for ZkLinkState {
    type Op = WithdrawOp;

    fn create_op(&self, tx: Withdraw) -> Result<Self::Op, anyhow::Error> {
        // All stateless checking(tx format, l1 signature) should have been done by rpc
        // and we still redo stateful(which could be changed, eg. balance, nonce, pub_key_hash) checking when execute l2 transaction

        // Check l1 target token exist in chain and l2 source token exist
        self.ensure_token_of_chain_supported(&tx.l1_target_token, &tx.to_chain_id)?;
        self.ensure_token_supported(&tx.l2_source_token)?;

        // Check whether the mapping between l1_token and l2_token is correct
        let (is_required, l1_target_token_after_mapping) =
            ZkLinkTx::check_source_token_and_target_token(tx.l2_source_token, tx.l1_target_token);
        ensure!(is_required, "source token or target token is mismatching");

        // Check account
        let pk = tx.verify_signature().ok_or(format_err!("Invalid l2 signature"))?;
        self.ensure_account_active_and_tx_pk_correct(tx.account_id, pk)?;

        let withdraw_op = WithdrawOp {
            account_id: tx.account_id ,
            tx,
            l1_target_token_after_mapping,
        };

        Ok(withdraw_op)
    }


    fn apply_op(
        &mut self,
        op: &mut Self::Op,
    ) -> Result<AccountUpdates, anyhow::Error> {
        // We ensure account_id and GLOBAL_ASSET_ACCOUNT_ID will be different after rpc check

        let mut updates = Vec::new();
        let mut from_account = self.get_account(op.account_id).unwrap();
        {
            let actual_token = Self::get_actual_token_by_sub_account(op.tx.sub_account_id, op.tx.l2_source_token);

            let from_old_balance = from_account.get_balance(actual_token);
            let from_old_nonce = from_account.nonce;
            ensure!(op.tx.nonce == from_old_nonce, "Nonce does not match");
            ensure!(from_old_balance >= &op.tx.amount + &op.tx.fee, "Insufficient balance");

            from_account.sub_balance(actual_token, &(&op.tx.amount + &op.tx.fee));
            *from_account.nonce += 1;

            let from_new_balance = from_account.get_balance(actual_token);
            let from_new_nonce = from_account.nonce;

            updates.push((
                op.account_id,
                AccountUpdate::UpdateBalance {
                    balance_update: (op.tx.l2_source_token, op.tx.sub_account_id, from_old_balance, from_new_balance),
                    old_nonce: from_old_nonce,
                    new_nonce: from_new_nonce,
                },
            ));
        }
        let mut global_account = self.get_account(GLOBAL_ASSET_ACCOUNT_ID).unwrap();
        {
            let actual_token = Self::get_actual_token_by_chain(op.tx.to_chain_id, op.l1_target_token_after_mapping);
            let global_old_amount = global_account.get_balance(actual_token);
            ensure!(
                global_old_amount >= op.tx.amount,
                "Withdrawal amount is greater than l1 withdrawal limit"
            );
            global_account.sub_balance(actual_token, &op.tx.amount);
            let global_new_amount = global_account.get_balance(actual_token);
            updates.push((
                GLOBAL_ASSET_ACCOUNT_ID,
                AccountUpdate::UpdateBalance {
                    balance_update: (op.l1_target_token_after_mapping, SubAccountId(*op.tx.to_chain_id), global_old_amount, global_new_amount),
                    old_nonce: Nonce(0),
                    new_nonce: Nonce(0),
                },
            ));
        }

        self.insert_account(op.account_id, from_account);
        self.insert_account(GLOBAL_ASSET_ACCOUNT_ID, global_account);
        // Collect withdraw token as fee
        self.collect_fee(op.tx.l2_source_token, &op.tx.fee, &mut updates);

        Ok(updates)
    }
}
