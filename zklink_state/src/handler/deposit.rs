use zklink_crypto::params::{GLOBAL_ASSET_ACCOUNT_ID};
use zklink_types::{Account, AccountUpdate, AccountUpdates, Deposit, DepositOp, Nonce, SubAccountId, ZkLinkTx};

use crate::{
    handler::TxHandler, state::ZkLinkState,
};

impl TxHandler<Deposit> for ZkLinkState {
    type Op = DepositOp;

    fn create_op(&self, tx: Deposit) -> Result<Self::Op, anyhow::Error> {
        // Check correct
        assert!(tx.check_correctness(), "Invalid tx format");

        // Check source token exist in from chain and target token exist in l2
        self.assert_token_of_chain_supported(&tx.l1_source_token, &tx.from_chain_id);
        self.assert_token_supported(&tx.l2_target_token);

        // Check whether the mapping between l1_token and l2_token is correct
        let (is_required, l1_source_token_after_mapping) =
            ZkLinkTx::check_source_token_and_target_token(tx.l2_target_token, tx.l1_source_token);
        assert!(is_required, "Source token or target token is mismatching in creating DepositOp");

        let account_id = self.get_account_by_address(&tx.to)
            .map_or_else(
                ||self.get_free_account_id(),
                |(account_id, _)|account_id
            );

        let op = DepositOp {
            tx,
            account_id,
            l1_source_token_after_mapping
        };

        Ok(op)
    }


    fn apply_op(
        &mut self,
        op: &mut Self::Op,
    ) -> Result<AccountUpdates, anyhow::Error> {
        let mut updates = Vec::new();

        let mut global_account = self.get_account(GLOBAL_ASSET_ACCOUNT_ID).unwrap();
        let mut account = self.get_account(op.account_id).unwrap_or_else(|| {
            let (account, upd) = Account::create_account(op.account_id, op.tx.to.clone());
            updates.extend(upd.into_iter());
            account
        });

        {
            let real_token = Self::get_actual_token_by_sub_account(op.tx.sub_account_id, op.tx.l2_target_token);
            let old_amount = account.get_balance(real_token);
            let old_nonce = account.nonce;
            account.add_balance(real_token, &op.tx.amount);
            let new_amount = account.get_balance(real_token);
            updates.push((
                op.account_id,
                AccountUpdate::UpdateBalance {
                    balance_update: (op.tx.l2_target_token, op.tx.sub_account_id, old_amount, new_amount),
                    old_nonce,
                    new_nonce: old_nonce,
                },
            ));
        }

        {
            // Use chain id as sub account id
            let real_token = Self::get_actual_token_by_chain(op.tx.from_chain_id, op.l1_source_token_after_mapping);
            let global_old_amount = global_account.get_balance(real_token);
            global_account.add_balance(real_token, &op.tx.amount);
            let global_new_amount = global_account.get_balance(real_token);
            updates.push((
                GLOBAL_ASSET_ACCOUNT_ID,
                AccountUpdate::UpdateBalance {
                    balance_update: (op.l1_source_token_after_mapping, SubAccountId(*op.tx.from_chain_id), global_old_amount, global_new_amount),
                    old_nonce: Nonce(0),
                    new_nonce: Nonce(0),
                },
            ));
        }

        self.insert_account(op.account_id, account);
        self.insert_account(GLOBAL_ASSET_ACCOUNT_ID, global_account);
        Ok(updates)
    }
}
