use num::Zero;
use std::cmp::min;
use zklink_crypto::params::GLOBAL_ASSET_ACCOUNT_ID;
use zklink_types::utils::check_source_token_and_target_token;
use zklink_types::{AccountUpdate, AccountUpdates, FullExit, FullExitOp, Nonce, SubAccountId};

use crate::{handler::TxHandler, state::ZkLinkState};

impl TxHandler<FullExit> for ZkLinkState {
    type Op = FullExitOp;

    fn create_op(&self, tx: FullExit) -> Result<Self::Op, anyhow::Error> {
        // Check tx
        assert!(tx.check_correctness(), "Invalid tx format");

        // Check target token exist in to chain and source token exist in l2
        self.assert_token_of_chain_supported(&tx.l1_target_token, &tx.to_chain_id);
        self.assert_token_supported(&tx.l2_source_token);

        // Check whether the mapping between l1_token and l2_token is correct
        let (is_required, l1_target_token_after_mapping) =
            check_source_token_and_target_token(tx.l2_source_token, tx.l1_target_token);
        assert!(
            is_required,
            "Source token or target token is mismatching in creating FullExitOp"
        );

        let user_token =
            Self::get_actual_token_by_sub_account(tx.sub_account_id, tx.l2_source_token);
        // If exit address not equal to exit account address, for example, account A full exit account B
        // FullExit will still be executed but there will be no effect on account B
        let account_balance = self
            .get_account(tx.account_id)
            .filter(|account| account.address == tx.exit_address)
            .map(|account| account.get_balance(user_token));

        // Use chain id as sub account id
        let global_real_token =
            Self::get_actual_token_by_chain(tx.to_chain_id, l1_target_token_after_mapping);
        let account = self.get_account(GLOBAL_ASSET_ACCOUNT_ID).unwrap();
        let global_asset_remain = account.get_balance(global_real_token);
        // The exit amount must not exceed the balance of global account of chain exit to
        let exit_amount = account_balance
            .map(|balance| min(balance, global_asset_remain))
            .unwrap_or_default();

        let op = FullExitOp {
            tx,
            exit_amount,
            l1_target_token_after_mapping,
        };

        Ok(op)
    }

    fn apply_op(&mut self, op: &mut Self::Op) -> Result<AccountUpdates, anyhow::Error> {
        let mut updates = Vec::new();
        let Some(mut account) = self.get_account(op.tx.account_id) else {
            // Although the account does not exist, we also want to have an update without transition.
            updates.push((
                op.tx.account_id,
                AccountUpdate::UpdateBalance {
                    balance_update: (op.tx.l2_source_token, op.tx.sub_account_id, Zero::zero(), Zero::zero()),
                    old_nonce: Nonce(0),
                    new_nonce: Nonce(0),
                },
            ));
            updates.push((
                GLOBAL_ASSET_ACCOUNT_ID,
                AccountUpdate::UpdateBalance {
                    balance_update: (op.l1_target_token_after_mapping, SubAccountId(*op.tx.to_chain_id), Zero::zero(), Zero::zero()),
                    old_nonce: Nonce(0),
                    new_nonce: Nonce(0),
                },
            ));
            return Ok(updates);
        };

        let mut global_account = self.get_account(GLOBAL_ASSET_ACCOUNT_ID).unwrap();
        {
            let real_token =
                Self::get_actual_token_by_sub_account(op.tx.sub_account_id, op.tx.l2_source_token);
            let old_balance = account.get_balance(real_token);
            let old_nonce = account.nonce;
            account.sub_balance(real_token, &op.exit_amount);
            let new_balance = account.get_balance(real_token);
            let new_nonce = account.nonce;

            updates.push((
                op.tx.account_id,
                AccountUpdate::UpdateBalance {
                    balance_update: (
                        op.tx.l2_source_token,
                        op.tx.sub_account_id,
                        old_balance,
                        new_balance,
                    ),
                    old_nonce,
                    new_nonce,
                },
            ));
        }

        {
            let real_token = Self::get_actual_token_by_chain(
                op.tx.to_chain_id,
                op.l1_target_token_after_mapping,
            );
            let global_old_amount = global_account.get_balance(real_token);
            global_account.sub_balance(real_token, &op.exit_amount);
            let global_new_amount = global_account.get_balance(real_token);

            updates.push((
                GLOBAL_ASSET_ACCOUNT_ID,
                AccountUpdate::UpdateBalance {
                    balance_update: (
                        op.l1_target_token_after_mapping,
                        SubAccountId(*op.tx.to_chain_id),
                        global_old_amount,
                        global_new_amount,
                    ),
                    old_nonce: Nonce(0),
                    new_nonce: Nonce(0),
                },
            ));
        }

        self.insert_account(op.tx.account_id, account);
        self.insert_account(GLOBAL_ASSET_ACCOUNT_ID, global_account);

        Ok(updates)
    }
}
