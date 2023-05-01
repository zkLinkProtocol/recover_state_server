// Built-in deps
use std::{cmp, collections::HashMap, time::Instant};
// External imports
use num::bigint::ToBigInt;
use num::BigInt;
use sqlx::types::BigDecimal;
use tracing::{debug, info};
// Workspace imports
use zklink_types::{
    helpers::{apply_updates, reverse_updates},
    AccountId, AccountMap, AccountUpdate, AccountUpdates, BlockNumber, ChainId, PubKeyHash, H256,
};
// Local imports
use crate::chain::{
    account::{records::*, restore_account},
    block::BlockSchema,
};
use crate::diff::StorageAccountDiff;
use crate::{QueryResult, StorageProcessor};
use zklink_types::block::FailedExecutedTx;

/// State schema is capable of managing... well, the state of the chain.
///
/// This roughly includes the two main topics:
/// - Account management (applying the diffs to the account map).
/// - Block events (which blocks were committed/verified).
///
/// # Representation of the Sidechain State in the DB:
///
/// Saving state is done in two steps:
/// 1. When the block is committed, we save all state updates
///   (tables: `account_creates`, `account_balance_updates`)
///   (tables: `accounts`, `balances`)
#[derive(Debug)]
pub struct StateSchema<'a, 'c>(pub &'a mut StorageProcessor<'c>);

impl<'a, 'c> StateSchema<'a, 'c> {
    /// Stores the list of updates to the account map in the database.
    /// At this step, the changes are not verified yet, and thus are not applied.
    pub async fn commit_state_update(
        &mut self,
        block_number: BlockNumber,
        accounts_updated: &[(AccountId, AccountUpdate, H256)],
    ) -> QueryResult<()> {
        let start = Instant::now();
        let mut transaction = self.0.start_transaction().await?;

        // Simply go through the every account update, and update the corresponding table.
        // This may look scary, but every match arm is very simple by its nature.

        let update_order_ids = 0..accounts_updated.len();

        for (update_order_id, (id, upd, hash)) in update_order_ids.zip(accounts_updated.iter()) {
            debug!(
                "Committing state update for account {} in block {}",
                **id, *block_number
            );
            let hash = hash.0.to_vec();
            match upd {
                AccountUpdate::Create { ref address, .. } => {
                    let account_id = i64::from(**id);
                    let block_number = i64::from(*block_number);
                    let address = address.as_bytes().to_vec();
                    let update_order_id = update_order_id as i32;
                    sqlx::query!(
                        r#"
                        INSERT INTO account_creates ( account_id, block_number, address, update_order_id, tx_hash)
                        VALUES ( $1, $2, $3, $4, $5)
                        "#,
                        account_id, block_number, address,  update_order_id, hash
                    )
                        .execute(transaction.conn())
                        .await?;
                }
                //  Close tx is removed
                AccountUpdate::Delete { .. } => {}
                AccountUpdate::UpdateBalance {
                    balance_update: (token, sub_account_id, ref old_balance, ref new_balance),
                    old_nonce,
                    new_nonce,
                } => {
                    let account_id = i64::from(**id);
                    let block_number = i64::from(*block_number);
                    let coin_id = token.0 as i32;
                    let sub_account_id = sub_account_id.0 as i32;
                    let old_balance = BigDecimal::from(BigInt::from(old_balance.clone()));
                    let new_balance = BigDecimal::from(BigInt::from(new_balance.clone()));
                    let old_nonce = i64::from(old_nonce.0);
                    let new_nonce = i64::from(new_nonce.0);
                    let update_order_id = update_order_id as i32;

                    sqlx::query!(
                        r#"
                        INSERT INTO account_balance_updates ( account_id, block_number, coin_id, sub_account_id, old_balance, new_balance, old_nonce, new_nonce, update_order_id, tx_hash )
                        VALUES ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                        "#,
                        account_id,
                        block_number,
                        coin_id,
                        sub_account_id,
                        old_balance,
                        new_balance,
                        old_nonce,
                        new_nonce,
                        update_order_id,
                        hash,
                    )
                        .execute(transaction.conn())
                        .await?;
                }
                AccountUpdate::UpdateTidyOrder {
                    slot_id,
                    sub_account_id,
                    old_order: old_order_nonce,
                    new_order: new_order_nonce,
                } => {
                    let update_order_id = update_order_id as i32;
                    let account_id = i64::from(**id);
                    let block_number = i64::from(*block_number);
                    let slot_id = slot_id.0 as i32;
                    let sub_account_id = sub_account_id.0 as i32;
                    let old_order = (
                        old_order_nonce.0 .0 as i64,
                        BigDecimal::from(old_order_nonce.1.to_bigint().unwrap()),
                    );
                    let new_order = (
                        new_order_nonce.0 .0 as i64,
                        BigDecimal::from(new_order_nonce.1.to_bigint().unwrap()),
                    );
                    let old_order_info =
                        serde_json::to_string(&old_order).expect("value to json string");
                    let new_order_info =
                        serde_json::to_string(&new_order).expect("value to json string");
                    sqlx::query!(
                        r#"
                        INSERT INTO account_order_updates ( update_order_id, account_id, block_number,
                        slot_id, old_order_nonce, new_order_nonce, sub_account_id, tx_hash)
                        VALUES ( $1, $2, $3, $4, $5, $6, $7, $8)
                        "#,
                        update_order_id,
                        account_id,
                        block_number,
                        slot_id,
                        serde_json::Value::String(old_order_info),
                        serde_json::Value::String(new_order_info),
                        sub_account_id,
                        hash,
                    )
                        .execute(transaction.conn())
                        .await?;
                }
                AccountUpdate::ChangePubKeyHash {
                    ref old_pub_key_hash,
                    ref new_pub_key_hash,
                    old_nonce,
                    new_nonce,
                } => {
                    let update_order_id = update_order_id as i32;
                    let account_id = i64::from(**id);
                    let block_number = i64::from(*block_number);
                    let old_pubkey_hash = old_pub_key_hash.data.to_vec();
                    let new_pubkey_hash = new_pub_key_hash.data.to_vec();
                    let old_nonce = i64::from(old_nonce.0);
                    let new_nonce = i64::from(new_nonce.0);
                    sqlx::query!(
                        r#"
                        INSERT INTO account_pubkey_updates ( update_order_id, account_id, block_number, old_pubkey_hash, new_pubkey_hash, old_nonce, new_nonce, tx_hash )
                        VALUES ( $1, $2, $3, $4, $5, $6, $7, $8 )
                        "#,
                        update_order_id, account_id, block_number, old_pubkey_hash, new_pubkey_hash, old_nonce, new_nonce, hash
                    )
                        .execute(transaction.conn())
                        .await?;
                }
            }
        }

        transaction.commit().await?;

        metrics::histogram!("sql.chain.state.commit_state_update", start.elapsed());
        Ok(())
    }

    pub async fn commit_failed_txs(
        &mut self,
        block_number: BlockNumber,
        failed_txs: Vec<FailedExecutedTx>,
    ) -> QueryResult<()> {
        let mut transaction = self
            .0
            .start_transaction()
            .await
            .expect("Failed initializing a DB transaction");
        transaction
            .chain()
            .block_schema()
            .save_block_failed_transactions(block_number, failed_txs)
            .await
            .expect("worker must commit the op into db");

        transaction
            .commit()
            .await
            .expect("Unable to commit DB transaction");
        Ok(())
    }

    pub async fn apply_account_type_updates(
        &mut self,
        account_types: Vec<(AccountId, AccountType, ChainId)>,
    ) -> QueryResult<()> {
        let mut transaction = self.0.start_transaction().await?;
        for (id, account_type, chain_id) in account_types {
            transaction
                .chain()
                .account_schema()
                .set_account_type(*id as i64, account_type, chain_id.0 as i16)
                .await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    /// Applies the previously stored list of account changes to the stored state.
    pub async fn apply_state_update(&mut self, block_number: BlockNumber) -> QueryResult<()> {
        let start = Instant::now();
        info!("Applying {:?} state update for storage", block_number);
        let mut transaction = self.0.start_transaction().await?;

        // Collect the stored updates. This includes collecting entries from three tables:
        // `account_creates` (for creating/removing accounts),
        // `account_balance_updates` (for changing the balance of accounts),
        // `account_pubkey_updates` (for changing the accounts public keys).
        let account_balance_diff = sqlx::query_as!(
            StorageAccountUpdate,
            "SELECT * FROM account_balance_updates WHERE block_number = $1",
            i64::from(*block_number)
        )
        .fetch_all(transaction.conn())
        .await?;

        let account_creation_diff = sqlx::query_as!(
            StorageAccountCreation,
            "
                SELECT * FROM account_creates
                WHERE block_number = $1
            ",
            i64::from(*block_number)
        )
        .fetch_all(transaction.conn())
        .await?;

        let account_change_pubkey_diff = sqlx::query_as!(
            StorageAccountPubkeyUpdate,
            "
                SELECT * FROM account_pubkey_updates
                WHERE block_number = $1
            ",
            i64::from(*block_number)
        )
        .fetch_all(transaction.conn())
        .await?;

        let account_order_diff = sqlx::query_as!(
            StorageAccountOrderUpdate,
            "SELECT * FROM account_order_updates WHERE block_number = $1 ",
            i64::from(*block_number),
        )
        .fetch_all(transaction.conn())
        .await?;

        // Collect the updates into one list of `StorageAccountDiff`.
        let account_updates: Vec<StorageAccountDiff> = {
            let mut account_diff = Vec::new();
            account_diff.extend(
                account_balance_diff
                    .into_iter()
                    .map(StorageAccountDiff::from),
            );
            account_diff.extend(
                account_creation_diff
                    .into_iter()
                    .map(StorageAccountDiff::from),
            );
            account_diff.extend(account_order_diff.into_iter().map(StorageAccountDiff::from));
            account_diff.extend(
                account_change_pubkey_diff
                    .into_iter()
                    .map(StorageAccountDiff::from),
            );

            account_diff.sort_by(StorageAccountDiff::cmp_order);
            account_diff
        };

        debug!("Sorted account update list: {:?}", account_updates);

        // Then go through the collected list of changes and apply them by one.
        for acc_update in account_updates.into_iter() {
            match acc_update {
                StorageAccountDiff::BalanceUpdate(upd) => {
                    sqlx::query!(
                        r#"
                        INSERT INTO balances ( account_id, coin_id, sub_account_id, balance )
                        VALUES ( $1, $2, $3, $4)
                        ON CONFLICT (account_id, coin_id, sub_account_id )
                        DO UPDATE
                          SET balance = $4
                        "#,
                        upd.account_id,
                        upd.coin_id,
                        upd.sub_account_id,
                        upd.new_balance.clone(),
                    )
                    .execute(transaction.conn())
                    .await?;

                    sqlx::query!(
                        r#"
                        UPDATE accounts
                        SET last_block = $1, nonce = $2
                        WHERE id = $3
                        "#,
                        upd.block_number,
                        upd.new_nonce,
                        upd.account_id,
                    )
                    .execute(transaction.conn())
                    .await?;
                }

                StorageAccountDiff::Create(upd) => {
                    sqlx::query!(
                        r#"
                        INSERT INTO accounts ( id, last_block, nonce, address, pubkey_hash, account_type, chain_id )
                        VALUES ( $1, $2, $3, $4, $5, $6, $7)
                        "#,
                        upd.account_id,
                        upd.block_number,
                        0i64,
                        upd.address,
                        PubKeyHash::default().data.to_vec(),
                        AccountType::Unknown as AccountType,
                        0,
                    )
                        .execute(transaction.conn())
                        .await?;
                }
                StorageAccountDiff::ChangePubKey(upd) => {
                    sqlx::query!(
                        r#"
                        UPDATE accounts 
                        SET last_block = $1, nonce = $2, pubkey_hash = $3
                        WHERE id = $4
                        "#,
                        upd.block_number,
                        upd.new_nonce,
                        upd.new_pubkey_hash,
                        upd.account_id,
                    )
                    .execute(transaction.conn())
                    .await?;
                }
                StorageAccountDiff::ChangeOrderNonce(upd) => {
                    let new_order_nonce: (i64, BigDecimal) =
                        serde_json::from_str(upd.new_order_nonce.as_str().unwrap()).unwrap();
                    sqlx::query!(
                        r#"
                        INSERT INTO account_order_nonces ( account_id, slot_id, order_nonce, residue, sub_account_id)
                        VALUES ( $1, $2, $3, $4, $5)
                        ON CONFLICT (account_id, slot_id, sub_account_id)
                        DO UPDATE
                          SET order_nonce = $3, residue = $4
                        "#,
                        upd.account_id,
                        upd.slot_id,
                        new_order_nonce.0,
                        new_order_nonce.1,
                        upd.sub_account_id,
                    )
                        .execute(transaction.conn())
                        .await?;
                }
            }
        }

        transaction.commit().await?;

        metrics::histogram!("sql.chain.state.apply_state_update", start.elapsed());
        Ok(())
    }

    /// Loads the state account map state along
    /// with a block number to which this state applies.
    pub async fn load_circuit_state(&mut self, block: i64) -> QueryResult<(i64, AccountMap)> {
        let start = Instant::now();
        let mut transaction = self.0.start_transaction().await?;

        let (last_block, mut accounts) = StateSchema(&mut transaction).load_last_state().await?;
        debug!(
            "Verified state block: {}, accounts: {:#?}",
            last_block, accounts
        );

        let state_diff = StateSchema(&mut transaction)
            .load_state_diff(last_block, Some(block))
            .await?;

        // Fetch updates from blocks: verif_block +/- 1, ... , block
        let result = if let Some((block, state_diff)) = state_diff {
            apply_updates(&mut accounts, state_diff);
            Ok((block, accounts))
        } else {
            Ok((last_block, accounts))
        };

        transaction.commit().await?;

        metrics::histogram!("sql.chain.state.load_committed_state", start.elapsed());
        result
    }

    /// Loads the verified account map state along with a block number
    /// to which this state applies.
    /// If the provided block number is `None`, then the latest committed
    /// state will be loaded.
    pub async fn load_last_state(&mut self) -> QueryResult<(i64, AccountMap)> {
        let start = Instant::now();
        let mut transaction = self.0.start_transaction().await?;

        let last_block = BlockSchema(&mut transaction)
            .get_last_block_number()
            .await?;

        // todo fetch all accounts at once will be failed when accounts table is big. we can fetch by page or store the tree at local file system
        let accounts = sqlx::query_as!(
            StorageAccount,
            r#"
            SELECT id, nonce, last_block, address, pubkey_hash, account_type as "account_type!: AccountType", chain_id FROM accounts
            "#)
            .fetch_all(transaction.conn())
            .await?;

        let mut account_map = AccountMap::default();

        // Get the balance of 2^15=32768 accounts at once
        // See https://doc.rust-lang.org/nightly/std/slice/struct.Chunks.html
        for stored_accounts in accounts.chunks(2usize.pow(15)) {
            let stored_account_ids: Vec<_> = stored_accounts.iter().map(|acc| acc.id).collect();
            let balances = sqlx::query_as!(
                StorageBalance,
                "SELECT * FROM balances WHERE account_id = ANY($1)",
                &stored_account_ids
            )
            .fetch_all(transaction.conn())
            .await?;

            let mut balances_for_id: HashMap<AccountId, Vec<StorageBalance>> = HashMap::new();

            for balance in balances.into_iter() {
                balances_for_id
                    .entry(AccountId(balance.account_id as u32))
                    .and_modify(|balances| balances.push(balance.clone()))
                    .or_insert_with(|| vec![balance]);
            }
            let order_nonces = sqlx::query_as!(
                StorageOrderNonce,
                "SELECT * FROM account_order_nonces WHERE account_id = ANY($1)",
                &stored_account_ids
            )
            .fetch_all(transaction.conn())
            .await?;

            let mut order_nonces_for_id: HashMap<AccountId, Vec<StorageOrderNonce>> =
                HashMap::new();
            for order_nonce in order_nonces.into_iter() {
                order_nonces_for_id
                    .entry(AccountId(order_nonce.account_id as u32))
                    .and_modify(|order_nonces| order_nonces.push(order_nonce.clone()))
                    .or_insert_with(|| vec![order_nonce]);
            }

            for stored_account in stored_accounts {
                let id = AccountId(stored_account.id as u32);
                let balances = balances_for_id.remove(&id).unwrap_or_default();
                let order_nonces = order_nonces_for_id.remove(&id).unwrap_or_default();
                let (id, account) = restore_account(stored_account, balances, order_nonces);
                account_map.insert(id, account);
            }
        }

        transaction.commit().await?;
        metrics::histogram!("sql.chain.state.load_verified_state", start.elapsed());
        Ok((last_block, account_map))
    }

    /// Loads the committed (not necessarily verified) account map state along
    /// with a block number to which this state applies.
    /// If the provided block number is `None`, then the latest committed
    /// state will be loaded.
    pub async fn load_committed_state(
        &mut self,
        block: Option<i64>,
    ) -> QueryResult<(i64, AccountMap)> {
        let start = Instant::now();
        let mut transaction = self.0.start_transaction().await?;

        let (verified_block, mut accounts) =
            StateSchema(&mut transaction).load_last_state().await?;
        debug!(
            "Verified state block: {}, accounts: {:#?}",
            verified_block, accounts
        );

        // Fetch updates from blocks: verif_block +/- 1, ... , block
        let result = if let Some((block, state_diff)) = StateSchema(&mut transaction)
            .load_state_diff(verified_block, block)
            .await?
        {
            debug!("Loaded state diff: {:#?}", state_diff);
            apply_updates(&mut accounts, state_diff);
            (block, accounts)
        } else {
            (verified_block, accounts)
        };

        transaction.commit().await?;

        metrics::histogram!("sql.chain.state.load_committed_state", start.elapsed());
        Ok(result)
    }

    /// Returns the list of updates, and the block number such that if we apply
    /// these updates to the state of the block #(from_block), we will obtain state of the block
    /// #(returned block number).
    /// Returned block number is either `to_block`, latest committed block before `to_block`.
    /// If `to_block` is `None`, then it will be assumed to be the number of the latest committed
    /// block.
    pub async fn load_state_diff(
        &mut self,
        from_block: i64,
        to_block: Option<i64>,
    ) -> QueryResult<Option<(i64, AccountUpdates)>> {
        let start = Instant::now();
        let mut transaction = self.0.start_transaction().await?;

        // Resolve the end of range: if it was not provided, we have to fetch
        // the latest committed block.
        let to_block_resolved = if let Some(to_block) = to_block {
            to_block
        } else {
            let last_block = sqlx::query!("SELECT max(number) FROM blocks",)
                .fetch_one(transaction.conn())
                .await?;

            last_block.max.unwrap_or(0i64)
        };

        // Determine the order: are we going forward or backwards.
        // Depending on that, determine the start/end of the block range as well.
        let (time_forward, start_block, end_block) = (
            from_block <= to_block_resolved,
            cmp::min(from_block, to_block_resolved),
            cmp::max(from_block, to_block_resolved),
        );

        // Collect the stored updates. This includes collecting entries from three tables:
        // `account_creates` (for creating/removing accounts),
        // `account_balance_updates` (for changing the balance of accounts),
        // `account_pubkey_updates` (for changing the accounts public keys).
        // The updates are loaded for the given blocks range.
        let account_balance_diff = sqlx::query_as!(
            StorageAccountUpdate,
            "SELECT * FROM account_balance_updates WHERE block_number > $1 AND block_number <= $2 ",
            start_block,
            end_block,
        )
        .fetch_all(transaction.conn())
        .await?;

        let account_creation_diff = sqlx::query_as!(
            StorageAccountCreation,
            "SELECT * FROM account_creates WHERE block_number > $1 AND block_number <= $2 ",
            start_block,
            end_block,
        )
        .fetch_all(transaction.conn())
        .await?;

        let account_pubkey_diff = sqlx::query_as!(
            StorageAccountPubkeyUpdate,
            "SELECT * FROM account_pubkey_updates WHERE block_number > $1 AND block_number <= $2 ",
            start_block,
            end_block,
        )
        .fetch_all(transaction.conn())
        .await?;

        let account_order_diff = sqlx::query_as!(
            StorageAccountOrderUpdate,
            "SELECT * FROM account_order_updates WHERE block_number > $1 AND block_number <= $2 ",
            start_block,
            end_block,
        )
        .fetch_all(transaction.conn())
        .await?;

        transaction.commit().await?;
        debug!(
            "Loading state diff: forward: {}, start_block: {}, end_block: {}, unbounded: {}",
            time_forward,
            start_block,
            end_block,
            to_block.is_none()
        );

        // Fold the updates into one list and determine the actual last block
        // (since user-provided one may not exist yet).
        let (mut account_updates, last_block) = {
            let mut account_diff = Vec::new();
            account_diff.extend(
                account_balance_diff
                    .into_iter()
                    .map(StorageAccountDiff::from),
            );
            account_diff.extend(
                account_creation_diff
                    .into_iter()
                    .map(StorageAccountDiff::from),
            );
            account_diff.extend(
                account_pubkey_diff
                    .into_iter()
                    .map(StorageAccountDiff::from),
            );
            account_diff.extend(account_order_diff.into_iter().map(StorageAccountDiff::from));
            let last_block = account_diff
                .iter()
                .map(|acc| acc.block_number())
                .max()
                .unwrap_or(0);

            account_diff.sort_by(StorageAccountDiff::cmp_order);
            (
                account_diff
                    .into_iter()
                    .map(|d| d.into())
                    .collect::<AccountUpdates>(),
                last_block,
            )
        };

        // Reverse the blocks order if needed.
        if !time_forward {
            reverse_updates(&mut account_updates);
        }

        // Determine the block number which state will be obtained after
        // applying the changes.
        let block_after_updates = if time_forward {
            last_block
        } else {
            start_block
        };

        metrics::histogram!("sql.chain.state.load_state_diff", start.elapsed());

        // We don't want to return an empty list to avoid the confusion, so return
        // `None` if there are no changes.
        if !account_updates.is_empty() {
            Ok(Some((block_after_updates, account_updates)))
        } else {
            Ok(None)
        }
    }
}
