use crate::handler::TxHandler;
use crate::state::ZkLinkState;
use anyhow::{ensure, format_err, Error};
use num::{BigUint, CheckedDiv, CheckedMul, One, Zero};
use zklink_crypto::params::{precision_magnified, FEE_DENOMINATOR, MAIN_SUB_ACCOUNT_ID};
use zklink_types::{
    operations::{OrderContext, OrderMatchingOp},
    AccountUpdate, AccountUpdates, Order, OrderMatching,
};

impl TxHandler<OrderMatching> for ZkLinkState {
    type Op = OrderMatchingOp;

    fn create_op(&self, tx: OrderMatching) -> Result<Self::Op, Error> {
        // All stateless checking(tx format, l1 signature) should have been done by rpc
        // and we still redo stateful(which could be changed, eg. balance, nonce, pub_key_hash) checking when execute l2 transaction

        // Check orders and order matching relationship
        let (maker_context, taker_context) = self.verify_order_accounts(&tx, true)?;

        // Check submitter
        let pk = tx
            .verify_signature()
            .ok_or(format_err!("Invalid l2 signature"))?;
        let submitter_account = self.ensure_account_active_and_tx_pk_correct(tx.account_id, pk)?;

        // Check fee token
        self.ensure_token_supported(&tx.fee_token)?;

        // Check submitter balance
        let actual_fee_token =
            Self::get_actual_token_by_sub_account(tx.sub_account_id, tx.fee_token);
        ensure!(
            tx.fee <= submitter_account.get_balance(actual_fee_token),
            "Insufficient submitter balance"
        );

        Ok(OrderMatchingOp {
            tx,
            maker_sell_amount: Default::default(),
            taker_sell_amount: Default::default(),
            maker_context,
            taker_context,
        })
    }

    fn apply_op(&mut self, op: &mut Self::Op) -> Result<AccountUpdates, Error> {
        self.execute_order_matching_op(op, false)
    }

    fn unsafe_apply_op(&mut self, op: &mut Self::Op) -> Result<AccountUpdates, Error> {
        self.execute_order_matching_op(op, true)
    }
}

impl ZkLinkState {
    fn execute_order_matching_op(
        &mut self,
        op: &mut OrderMatchingOp,
        is_recovered: bool,
    ) -> Result<AccountUpdates, Error> {
        // preparing token
        let (maker_sell_token_base, taker_sell_token_base) = if op.tx.maker.is_sell.is_zero() {
            (op.tx.maker.quote_token_id, op.tx.maker.base_token_id)
        } else {
            (op.tx.maker.base_token_id, op.tx.maker.quote_token_id)
        };
        let maker_sell_token = Self::get_actual_token_by_sub_account(
            op.tx.taker.sub_account_id,
            maker_sell_token_base,
        );
        let taker_sell_token = Self::get_actual_token_by_sub_account(
            op.tx.maker.sub_account_id,
            taker_sell_token_base,
        );

        // calculate actual exchanged amounts
        let (taker_obtain_token_amount, maker_obtain_token_amount) =
            Self::calculate_actual_exchanged_amounts(op)
                .ok_or(format_err!("Internal calculation error!"))?;

        let (maker_sell_amount, taker_sell_amount) = if is_recovered {
            (op.maker_sell_amount.clone(), op.taker_sell_amount.clone())
        } else if !op.tx.expect_base_amount.is_zero() && !op.tx.expect_quote_amount.is_zero() {
            if op.tx.maker.is_sell.is_one() {
                // When maker sell base token, taker obtain base token and maker obtain quote token
                ensure!(
                    op.tx.expect_base_amount <= taker_obtain_token_amount,
                    "Expected base token amount does not match actual base token amount"
                );
                ensure!(
                    op.tx.expect_quote_amount <= maker_obtain_token_amount,
                    "Expected quote token amount does not match actual quote token amount"
                );
                (
                    op.tx.expect_base_amount.clone(),
                    op.tx.expect_quote_amount.clone(),
                )
            } else {
                // When maker buy base token, taker obtain quote token and maker obtain base token
                ensure!(
                    op.tx.expect_base_amount <= maker_obtain_token_amount,
                    "Expected base token amount does not match actual base token amount"
                );
                ensure!(
                    op.tx.expect_quote_amount <= taker_obtain_token_amount,
                    "Expected quote token amount does not match actual quote token amount"
                );
                (
                    op.tx.expect_quote_amount.clone(),
                    op.tx.expect_base_amount.clone(),
                )
            }
        } else {
            (taker_obtain_token_amount, maker_obtain_token_amount)
        };

        // calculate submitter collect fee.
        let maker_fee =
            &taker_sell_amount * op.tx.maker.fee_ratio1 / BigUint::from(FEE_DENOMINATOR);
        let taker_fee =
            &maker_sell_amount * op.tx.taker.fee_ratio2 / BigUint::from(FEE_DENOMINATOR);
        let exchanged_base_amount = if op.tx.maker.is_sell.is_one() {
            &maker_sell_amount
        } else {
            &taker_sell_amount
        };

        // update all account
        let mut maker_account = self.get_account(op.tx.maker.account_id).unwrap();
        let mut updates = vec![];
        {
            // 1.update maker account balance and order
            let maker_slot_id =
                Self::get_actual_slot(op.tx.maker.sub_account_id, op.tx.maker.slot_id);
            let order = maker_account.order_slots.entry(maker_slot_id).or_default();

            let old_nonce = order.nonce;
            let old_residue = order.residue.0.clone();
            // modified order
            order.update(exchanged_base_amount, &op.tx.maker);

            let new_nonce = order.nonce;
            let new_residue = order.residue.0.clone();
            updates.push((
                op.tx.maker.account_id,
                AccountUpdate::UpdateTidyOrder {
                    slot_id: op.tx.maker.slot_id,
                    sub_account_id: op.tx.maker.sub_account_id,
                    old_order: (old_nonce, old_residue),
                    new_order: (new_nonce, new_residue),
                },
            ));
            // modified balance
            let old_balance = maker_account.get_balance(maker_sell_token);
            maker_account.sub_balance(maker_sell_token, &maker_sell_amount);
            let new_balance = maker_account.get_balance(maker_sell_token);
            updates.push((
                op.tx.maker.account_id,
                AccountUpdate::UpdateBalance {
                    balance_update: (
                        maker_sell_token_base,
                        op.tx.maker.sub_account_id,
                        old_balance,
                        new_balance,
                    ),
                    old_nonce: maker_account.nonce,
                    new_nonce: maker_account.nonce,
                },
            ));
        }
        {
            // 2.update maker account balance
            let old_balance = maker_account.get_balance(taker_sell_token);
            maker_account.add_balance(taker_sell_token, &(&taker_sell_amount - &maker_fee));
            let new_balance = maker_account.get_balance(taker_sell_token);
            updates.push((
                op.tx.maker.account_id,
                AccountUpdate::UpdateBalance {
                    balance_update: (
                        taker_sell_token_base,
                        op.tx.maker.sub_account_id,
                        old_balance,
                        new_balance,
                    ),
                    old_nonce: maker_account.nonce,
                    new_nonce: maker_account.nonce,
                },
            ));
        }
        let mut taker_account = if op.tx.taker.account_id == op.tx.maker.account_id {
            maker_account.clone()
        } else {
            self.get_account(op.tx.taker.account_id).unwrap()
        };
        {
            // 3.update taker account balance and order
            // maker slot id and taker slot id will not be same even if they are the same account
            let taker_slot_id =
                Self::get_actual_slot(op.tx.taker.sub_account_id, op.tx.taker.slot_id);
            let order = taker_account.order_slots.entry(taker_slot_id).or_default();

            let old_nonce = order.nonce;
            let old_residue = order.residue.0.clone();
            // modified order
            order.update(exchanged_base_amount, &op.tx.taker);
            let new_nonce = order.nonce;
            let new_residue = order.residue.0.clone();
            updates.push((
                op.tx.taker.account_id,
                AccountUpdate::UpdateTidyOrder {
                    slot_id: op.tx.taker.slot_id,
                    sub_account_id: op.tx.taker.sub_account_id,
                    old_order: (old_nonce, old_residue),
                    new_order: (new_nonce, new_residue),
                },
            ));
            // modified balance
            let old_balance = taker_account.get_balance(taker_sell_token);
            taker_account.sub_balance(taker_sell_token, &taker_sell_amount);
            let new_balance = taker_account.get_balance(taker_sell_token);
            updates.push((
                op.tx.taker.account_id,
                AccountUpdate::UpdateBalance {
                    balance_update: (
                        taker_sell_token_base,
                        op.tx.taker.sub_account_id,
                        old_balance,
                        new_balance,
                    ),
                    old_nonce: taker_account.nonce,
                    new_nonce: taker_account.nonce,
                },
            ));
        }
        {
            // 4.update taker account balance
            let old_balance = taker_account.get_balance(maker_sell_token);
            taker_account.add_balance(maker_sell_token, &(&maker_sell_amount - &taker_fee));
            let new_balance = taker_account.get_balance(maker_sell_token);
            updates.push((
                op.tx.taker.account_id,
                AccountUpdate::UpdateBalance {
                    balance_update: (
                        maker_sell_token_base,
                        op.tx.taker.sub_account_id,
                        old_balance,
                        new_balance,
                    ),
                    old_nonce: taker_account.nonce,
                    new_nonce: taker_account.nonce,
                },
            ));
        }
        let mut submitter_account = if op.tx.account_id == op.tx.taker.account_id {
            taker_account.clone()
        } else if op.tx.account_id == op.tx.maker.account_id {
            maker_account.clone()
        } else {
            self.get_account(op.tx.account_id).unwrap()
        };
        let actual_fee_token =
            Self::get_actual_token_by_sub_account(op.tx.sub_account_id, op.tx.fee_token);
        {
            // 5.update submitter account balance
            let old_balance = submitter_account.get_balance(actual_fee_token);
            let old_nonce = submitter_account.nonce;
            submitter_account.sub_balance(actual_fee_token, &op.tx.fee);
            let new_balance = submitter_account.get_balance(actual_fee_token);
            updates.push((
                op.tx.account_id,
                AccountUpdate::UpdateBalance {
                    balance_update: (
                        op.tx.fee_token,
                        op.tx.sub_account_id,
                        old_balance,
                        new_balance,
                    ),
                    old_nonce,
                    new_nonce: old_nonce,
                },
            ));
            // 6.update submitter account sell_token balance
            // trading fee collected to MAIN_SUB_ACCOUNT_ID similar to protocol fee
            let fee_collect_sub_account_id = MAIN_SUB_ACCOUNT_ID;
            let taker_buy_token = Self::get_actual_token_by_sub_account(
                fee_collect_sub_account_id,
                maker_sell_token_base,
            );
            let old_balance = submitter_account.get_balance(taker_buy_token);
            submitter_account.add_balance(taker_buy_token, &taker_fee);
            let new_balance = submitter_account.get_balance(taker_buy_token);
            updates.push((
                op.tx.account_id,
                AccountUpdate::UpdateBalance {
                    balance_update: (
                        maker_sell_token_base,
                        fee_collect_sub_account_id,
                        old_balance,
                        new_balance,
                    ),
                    old_nonce,
                    new_nonce: old_nonce,
                },
            ));
            // 7.update submitter account buy_token balance
            let maker_buy_token = Self::get_actual_token_by_sub_account(
                fee_collect_sub_account_id,
                taker_sell_token_base,
            );
            let old_balance = submitter_account.get_balance(maker_buy_token);
            submitter_account.add_balance(maker_buy_token, &maker_fee);
            let new_balance = submitter_account.get_balance(maker_buy_token);
            updates.push((
                op.tx.account_id,
                AccountUpdate::UpdateBalance {
                    balance_update: (
                        taker_sell_token_base,
                        fee_collect_sub_account_id,
                        old_balance,
                        new_balance,
                    ),
                    old_nonce,
                    new_nonce: old_nonce,
                },
            ));
        }

        // update op
        op.maker_sell_amount = maker_sell_amount;
        op.taker_sell_amount = taker_sell_amount;

        // update account
        if op.tx.maker.account_id != op.tx.taker.account_id
            && op.tx.maker.account_id != op.tx.account_id
        {
            self.insert_account(op.tx.maker.account_id, maker_account);
        }
        if op.tx.taker.account_id != op.tx.account_id {
            self.insert_account(op.tx.taker.account_id, taker_account);
        }
        self.insert_account(op.tx.account_id, submitter_account);
        self.collect_fee(op.tx.fee_token, &op.tx.fee, &mut updates);

        Ok(updates)
    }

    fn calculate_actual_exchanged_amounts(op: &OrderMatchingOp) -> Option<(BigUint, BigUint)> {
        // The unit of actual quantity is base token(eg. BTC)
        // and the unit of actual exchanged is quote token(eg. USD)
        let maker_quantity = &op.maker_context.residue;
        let taker_quantity = &op.taker_context.residue;

        // Return the token obtained by taker and maker
        let actual_quantity = maker_quantity.min(taker_quantity);
        let actual_exchanged = actual_quantity
            .checked_mul(&op.tx.maker.price)?
            .checked_div(&precision_magnified())?;
        if op.tx.maker.is_sell.is_one() {
            // When maker sell base token, taker obtain base token and maker obtain quote token
            (actual_quantity.clone(), actual_exchanged)
        } else {
            // When maker buy base token, taker obtain quote token and maker obtain base token
            (actual_exchanged, actual_quantity.clone())
        }
        .into()
    }

    fn verify_order_accounts(
        &self,
        tx: &OrderMatching,
        check_signature: bool,
    ) -> Result<(OrderContext, OrderContext), Error> {
        let verify_account = |order: &Order, price: &BigUint| {
            // Check order account
            let account = if check_signature {
                let pk = order.signature.pub_key_hash();
                self.ensure_account_active_and_tx_pk_correct(order.account_id, pk)?
            } else {
                self.get_account(order.account_id)
                    .ok_or_else(|| format_err!("Account does not exist"))?
            };

            let slot =
                account.get_order(Self::get_actual_slot(order.sub_account_id, order.slot_id));

            // Check order nonce
            let residue = if !slot.residue.is_zero() {
                ensure!(
                    order.nonce == slot.nonce || order.nonce == slot.nonce + 1,
                    "Order nonce does not match"
                );
                if order.nonce == slot.nonce {
                    // Same nonce and existing residue means we want to continue match last order
                    &slot.residue.0
                } else {
                    // Or we want to refresh last order
                    &order.amount
                }
            } else {
                // if slot is free(non-residue), means the next order nonce must be same with current nonce
                ensure!(order.nonce == slot.nonce, "Order nonce does not match");
                &order.amount
            };

            let (necessary_amount, token) = if order.is_sell.is_one() {
                // Sell base token(eg. BTC), account need base token
                (residue.clone(), order.base_token_id)
            } else {
                // Buy base token and account need quote token(eg. USD)
                let amount = residue * price / precision_magnified();
                ensure!(!amount.is_zero(), "Residual value is too small");
                (amount, order.quote_token_id)
            };
            let balance = account.get_balance(Self::get_actual_token_by_sub_account(
                order.sub_account_id,
                token,
            ));
            ensure!(balance >= necessary_amount, "Insufficient Balance");

            Ok(OrderContext {
                residue: residue.clone(),
            })
        };

        let maker_context =
            verify_account(&tx.maker, &tx.maker.price).map_err(|e| format_err!("Maker {}", e))?;
        let taker_context =
            verify_account(&tx.taker, &tx.maker.price).map_err(|e| format_err!("Taker {}", e))?;
        Ok((maker_context, taker_context))
    }
}
