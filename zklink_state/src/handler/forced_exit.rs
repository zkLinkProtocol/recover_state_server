use std::cmp::min;
use anyhow::{ensure, format_err};
use zklink_crypto::params::{GLOBAL_ASSET_ACCOUNT_ID};
use zklink_types::{AccountUpdate, AccountUpdates, ForcedExit, ForcedExitOp, PubKeyHash, SubAccountId, Nonce, ZkLinkTx};

use crate::{
    handler::TxHandler,
    state::ZkLinkState,
};

impl TxHandler<ForcedExit> for ZkLinkState {
    type Op = ForcedExitOp;

    fn create_op(&self, tx: ForcedExit) -> Result<Self::Op, anyhow::Error> {
        // All stateless checking(tx format, l1 signature) should have been done by rpc
        // and we still redo stateful(which could be changed, eg. balance, nonce, pub_key_hash) checking when execute l2 transaction

        // Check l1 target token exist in chain and l2 source token exist
        self.ensure_token_of_chain_supported(&tx.l1_target_token, &tx.to_chain_id)?;
        self.ensure_token_supported(&tx.l2_source_token)?;

        // Check whether the mapping between l1_token and l2_token is correct
        let (is_required, l1_target_token_after_mapping) =
            ZkLinkTx::check_source_token_and_target_token(tx.l2_source_token, tx.l1_target_token);
        ensure!(is_required, "Source token or target token is mismatching in creating ForcedExitOp");

        // Check fee token exist
        self.ensure_token_supported(&tx.fee_token)?;

        // Check initial account
        let pk = tx.verify_signature().ok_or(format_err!("Invalid l2 signature"))?;
        self.ensure_account_active_and_tx_pk_correct(tx.initiator_account_id, pk)?;

        // Check that target account does not have been active yet
        // and initiator_account will not be same with target_account as a result
        let (target_account_id, account) = self
            .get_account_by_address(&tx.target)
            .ok_or_else(|| format_err!("Target account does not exist"))?;
        ensure!(
            target_account_id != GLOBAL_ASSET_ACCOUNT_ID,
            "Target account cannot be global asset account"
        );
        ensure!(
            account.pub_key_hash == PubKeyHash::default(),
            "Target account already activated"
        );

        // Obtain the token balance to be withdrawn.
        let account_balance = {
            let real_token = Self::get_actual_token_by_sub_account(tx.target_sub_account_id, tx.l2_source_token);
            account.get_balance(real_token)
        };
        let amount_remain = {
            // Use chain id as sub account id
            let global_real_token = Self::get_actual_token_by_chain(tx.to_chain_id, l1_target_token_after_mapping);
            let global_account = self.get_account(GLOBAL_ASSET_ACCOUNT_ID).unwrap();
            global_account.get_balance(global_real_token)
        };
        let account_balance = min(account_balance, amount_remain);

        let forced_exit_op = ForcedExitOp {
            tx,
            target_account_id,
            withdraw_amount: account_balance,
            l1_target_token_after_mapping
        };

        Ok(forced_exit_op)
    }


    fn apply_op(
        &mut self,
        op: &mut Self::Op,
    ) -> Result<AccountUpdates, anyhow::Error> {
        // We ensure that initiator_account_id, target_account_id and GLOBAL_ASSET_ACCOUNT_ID
        // all different after rpc and handler check
        let initiator_account_id = op.tx.initiator_account_id;
        let target_account_id = op.target_account_id;

        let mut updates = Vec::new();
        let mut initiator_account = self.get_account(initiator_account_id).unwrap();
        let mut target_account = self.get_account(target_account_id).unwrap();
        let mut global_account = self.get_account(GLOBAL_ASSET_ACCOUNT_ID).unwrap();
        let real_fee_token = Self::get_actual_token_by_sub_account(op.tx.initiator_sub_account_id,op.tx.fee_token);
        let real_token = Self::get_actual_token_by_sub_account(op.tx.target_sub_account_id, op.tx.l2_source_token);
        let global_real_token = Self::get_actual_token_by_chain(op.tx.to_chain_id, op.l1_target_token_after_mapping);


        // Check that initiator account has enough balance to cover fees.
        let initiator_old_balance = initiator_account.get_balance(real_fee_token);
        let initiator_old_nonce = initiator_account.nonce;

        ensure!(op.tx.nonce == initiator_old_nonce, "Nonce does not match");
        ensure!(
            initiator_old_balance >= op.tx.fee,
            "Insufficient balance of initiator account"
        );

        // Check that target account has required amount of tokens to withdraw.
        // (normally, it should, since we're declaring this amount ourselves, but
        // this check is added for additional safety).
        let target_old_balance = target_account.get_balance(real_token);
        // amount must be equal to either target old balance or global account token remain.
        // this is equal to amount == min(target_old_balance, global_account.get_balance(global_real_token))
        ensure!(
            op.withdraw_amount == target_old_balance || op.withdraw_amount == global_account.get_balance(global_real_token),
            "Insufficient balance or withdrawal amount is greater than withdrawal limit"
        );

        // Take fees from the initiator account (and update initiator account nonce).
        initiator_account.sub_balance(real_fee_token, &op.tx.fee);
        *initiator_account.nonce += 1;

        // Withdraw funds from the target account (note that target account nonce is not affected).
        target_account.sub_balance(real_token, &op.withdraw_amount);

        // Store required data to generate account updates later.
        let initiator_new_balance = initiator_account.get_balance(real_fee_token);
        let initiator_new_nonce = initiator_account.nonce;

        let target_new_balance = target_account.get_balance(real_token);
        let target_nonce = target_account.nonce;
        updates.push((
            initiator_account_id,
            AccountUpdate::UpdateBalance {
                balance_update: (op.tx.fee_token, op.tx.initiator_sub_account_id, initiator_old_balance, initiator_new_balance),
                old_nonce: initiator_old_nonce,
                new_nonce: initiator_new_nonce,
            },
        ));

        updates.push((
            target_account_id,
            AccountUpdate::UpdateBalance {
                balance_update: (op.tx.l2_source_token, op.tx.target_sub_account_id, target_old_balance, target_new_balance),
                old_nonce: target_nonce,
                new_nonce: target_nonce,
            },
        ));
        {
            let global_old_amount = global_account.get_balance(global_real_token);
            global_account.sub_balance(global_real_token, &op.withdraw_amount);
            let global_new_amount = global_account.get_balance(global_real_token);

            updates.push((
                GLOBAL_ASSET_ACCOUNT_ID,
                AccountUpdate::UpdateBalance {
                    balance_update: (op.l1_target_token_after_mapping, SubAccountId(*op.tx.to_chain_id), global_old_amount, global_new_amount),
                    old_nonce: Nonce(0),
                    new_nonce: Nonce(0),
                },
            ));
        }

        // Update accounts in the tree.
        self.insert_account(op.tx.initiator_account_id, initiator_account);
        self.insert_account(op.target_account_id, target_account);
        self.insert_account(GLOBAL_ASSET_ACCOUNT_ID, global_account);
        self.collect_fee(op.tx.fee_token, &op.tx.fee, &mut updates);

        Ok(updates)
    }
}
