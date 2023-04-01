// Built-in deps
use std::time::Instant;
use chrono::{DateTime, Utc};
use sqlx::types::BigDecimal;
// Local imports
use self::records::*;
use crate::{QueryResult, StorageProcessor};
use crate::chain::block::BlockSchema;

pub mod records;
mod restore_account;

pub(crate) use self::restore_account::restore_account;

/// Account schema contains interfaces to interact with the stored ZkLink accounts.
#[derive(Debug)]
pub struct AccountSchema<'a, 'c>(pub &'a mut StorageProcessor<'c>);

impl<'a, 'c> AccountSchema<'a, 'c> {
    /// Stores account type in the databse
    /// There are 2 types: Owned and CREATE2
    pub async fn set_account_type(
        &mut self,
        account_id: i64,
        account_type: AccountType,
        chain_id: i16,
    ) -> QueryResult<()> {
        let start = Instant::now();

        sqlx::query!(
            r#"
            UPDATE accounts SET account_type = $2, chain_id = $3 WHERE id = $1
            "#,
            account_id,
            account_type as AccountType,
            chain_id,
        )
        .execute(self.0.conn())
        .await?;

        metrics::histogram!("sql.chain.state.set_account_type", start.elapsed());
        Ok(())
    }

    /// Fetches account type from the database
    pub async fn account_type_by_id(
        &mut self,
        account_id: i64,
    ) -> QueryResult<Option<AccountType>> {
        let start = Instant::now();

        let result = sqlx::query_as!(
            StorageAccountType,
            r#"
            SELECT id as account_id, account_type as "account_type!: AccountType"
            FROM accounts WHERE id = $1
            "#,
            account_id
        )
        .fetch_optional(self.0.conn())
        .await?;

        let account_type = result.map(|record| record.account_type as AccountType);
        metrics::histogram!("sql.chain.account.account_type_by_id", start.elapsed());
        Ok(account_type)
    }

    /// Obtains account by its id.
    pub async fn account_by_id(
        &mut self,
        account_id: i64
    ) -> QueryResult<Option<StorageAccount>> {
        let start = Instant::now();

        let account = sqlx::query_as!(
            StorageAccount,
            r#"SELECT id,nonce,address,pubkey_hash,account_type as "account_type: AccountType",chain_id,last_block FROM accounts WHERE id = $1"#,
            account_id
        )
            .fetch_optional(self.0.conn())
            .await?;

        metrics::histogram!(
            "sql.chain.account.account_by_id",
            start.elapsed()
        );

        Ok(account)
    }

    /// Obtains account by its address.
    pub async fn account_by_address(
        &mut self,
        address: &[u8]
    ) -> QueryResult<Option<StorageAccount>> {
        let start = Instant::now();

        let account = sqlx::query_as!(
            StorageAccount,
            r#"SELECT id,nonce,address,pubkey_hash,account_type as "account_type: AccountType",chain_id,last_block FROM accounts WHERE address = $1"#,
            address
        )
            .fetch_optional(self.0.conn())
            .await?;

        metrics::histogram!(
            "sql.chain.account.account_by_address",
            start.elapsed()
        );

        Ok(account)
    }

    /// Obtains sub account token balance
    pub async fn sub_account_token_balance(
        &mut self,
        account_id: i64,
        sub_account_id: i32,
        token_id: i32
    ) -> QueryResult<Option<StorageBalance>> {
        let start = Instant::now();

        let token_balance = sqlx::query_as!(
                StorageBalance,
                r#"SELECT * FROM balances WHERE account_id = $1 AND sub_account_id = $2 AND coin_id = $3"#,
                account_id,
                sub_account_id,
                token_id
            )
            .fetch_optional(self.0.conn())
            .await?;

        metrics::histogram!(
            "sql.chain.account.sub_account_token_balance",
            start.elapsed()
        );

        Ok(token_balance)
    }

    /// Obtains balances for the account by its id and sub account id.
    pub async fn account_balances(
        &mut self,
        account_id: i64,
        sub_account_id: Option<i32>
    ) -> QueryResult<Vec<StorageBalance>> {
        let balances = match sub_account_id {
            Some(sub_account_id) => sqlx::query_as!(
                    StorageBalance,
                    r#"SELECT * FROM balances WHERE account_id = $1 and sub_account_id = $2"#,
                    account_id,
                    sub_account_id
                )
                .fetch_all(self.0.conn())
                .await?,
            None => sqlx::query_as!(
                        StorageBalance,
                    r#"SELECT * FROM balances WHERE account_id = $1"#,
                    account_id
                )
                .fetch_all(self.0.conn())
                .await?
        };

        Ok(balances)
    }

    /// Obtains order slots for the account by its id and sub account id.
    pub async fn account_order_slots(
        &mut self,
        account_id: i64,
        sub_account_id: Option<i32>
    ) -> QueryResult<Vec<StorageOrderNonce>> {
        let orders = match sub_account_id {
            Some(sub_account_id) => sqlx::query_as!(
                        StorageOrderNonce,
                        r#"SELECT * FROM account_order_nonces WHERE account_id = $1 and sub_account_id = $2"#,
                        account_id,
                        sub_account_id
                    )
                    .fetch_all(self.0.conn())
                    .await?,
            None => sqlx::query_as!(
                            StorageOrderNonce,
                        r#"SELECT * FROM account_order_nonces WHERE account_id = $1"#,
                        account_id
                    )
                    .fetch_all(self.0.conn())
                    .await?
        };

        Ok(orders)
    }

    pub async fn earliest_account_balance_updates_from_block(
        &mut self,
        account_id: i64,
        sub_account_id: Option<i32>,
        block_number: i64
    ) -> QueryResult<Vec<StorageAccountUpdate>> {
        let updates = match sub_account_id {
            // Group by coin_id for each account
            Some(sub_account_id) => {
                sqlx::query_as!(
                    StorageAccountUpdate,
                    r#"
                        SELECT a.* FROM account_balance_updates a INNER JOIN
                        (SELECT min(balance_update_id) FROM account_balance_updates
                        WHERE account_id=$1 AND sub_account_id=$2 AND block_number>$3
                        GROUP BY coin_id) b
                        ON a.balance_update_id = b.min
                    "#,
                    account_id,
                    sub_account_id,
                    block_number
                )
                .fetch_all(self.0.conn())
                .await?
            },
            None => {
                // Group by (sub_account_id, coin_id) for each account
                sqlx::query_as!(
                    StorageAccountUpdate,
                    r#"
                        SELECT a.* FROM account_balance_updates a INNER JOIN
                        (SELECT min(balance_update_id) FROM account_balance_updates
                        WHERE account_id=$1 AND block_number>$2
                        GROUP BY sub_account_id, coin_id) b
                        ON a.balance_update_id = b.min
                    "#,
                    account_id,
                    block_number
                )
                    .fetch_all(self.0.conn())
                    .await?
            }
        };

        Ok(updates)
    }

    pub async fn earliest_account_order_updates_from_block(
        &mut self,
        account_id: i64,
        sub_account_id: Option<i32>,
        block_number: i64
    ) -> QueryResult<Vec<StorageAccountOrderUpdate>> {
        let updates = match sub_account_id {
            Some(sub_account_id) => {
                // Group by slot_id for each account
                sqlx::query_as!(
                    StorageAccountOrderUpdate,
                    r#"SELECT a.* FROM account_order_updates a INNER JOIN
                    (SELECT min(order_nonce_update_id) FROM account_order_updates
                    WHERE account_id = $1 AND sub_account_id = $2 AND block_number > $3
                    GROUP BY slot_id) b
                    ON a.order_nonce_update_id = b.min"#,
                    account_id,
                    sub_account_id,
                    block_number
                )
                    .fetch_all(self.0.conn())
                    .await?
            },
            None => {
                // Group by (sub_account_id, slot_id) for each account
                sqlx::query_as!(
                    StorageAccountOrderUpdate,
                    r#"SELECT a.* FROM account_order_updates a INNER JOIN
                    (SELECT min(order_nonce_update_id) FROM account_order_updates
                    WHERE account_id = $1 AND block_number > $2
                    GROUP BY sub_account_id, slot_id) b
                    ON a.order_nonce_update_id = b.min"#,
                    account_id,
                    block_number
                )
                    .fetch_all(self.0.conn())
                    .await?
            }
        };

        Ok(updates)
    }

    pub async fn earliest_account_pubkey_update_from_block(
        &mut self,
        account_id: i64,
        block_number: i64
    ) -> QueryResult<Option<StorageAccountPubkeyUpdate>> {
        // There will be at most one update for each account
        let updates = sqlx::query_as!(
                    StorageAccountPubkeyUpdate,
                    r#"SELECT a.* FROM account_pubkey_updates a INNER JOIN
                    (SELECT min(pubkey_update_id) FROM account_pubkey_updates
                    WHERE account_id = $1 AND block_number > $2) b
                    ON a.pubkey_update_id = b.min
                    "#,
                    account_id,
                    block_number
                )
            .fetch_optional(self.0.conn())
            .await?;

        Ok(updates)
    }

    pub async fn account_snapshot(
        &mut self,
        account_id: i64,
        sub_account_id: Option<i32>,
        block_number: Option<i64>
    ) -> QueryResult<AccountSnapshot> {
        // Query state and earliest updates in a transaction
        let mut transaction = self.0.start_transaction().await?;

        let mut block_schema = BlockSchema(&mut transaction);
        let block_number = match block_number {
            Some(block_number) => block_number,
            None => {
                block_schema
                    .get_last_block_number()
                    .await?
            }
        };

        let mut account_schema = AccountSchema(&mut transaction);

        let mut account = account_schema
            .account_by_id(account_id)
            .await?;
        let mut balances = vec![];
        let mut order_slots = vec![];

        match &mut account {
            Some(account) => {
                balances = account_schema
                    .account_balances(account_id, sub_account_id)
                    .await?;
                order_slots = account_schema
                    .account_order_slots(account_id, sub_account_id)
                    .await?;

                // Obtain earliest updates from block, and then we can know the snapshot of `block_number`
                // from `old_balance` and `old_nonce`.
                let balance_updates = account_schema
                    .earliest_account_balance_updates_from_block(account_id, sub_account_id, block_number)
                    .await?;
                let pubkey_update = account_schema
                    .earliest_account_pubkey_update_from_block(account_id, block_number)
                    .await?;
                let order_updates = account_schema
                    .earliest_account_order_updates_from_block(account_id, sub_account_id, block_number)
                    .await?;

                // Commit after get all data to reduce the time of transaction
                transaction.commit().await?;

                // Recovery snapshot
                // Note, we need to merge account_balance_updates and account_pubkey_updates to recovery nonce
                for u in balance_updates {
                    // Recovery nonce
                    if account.nonce > u.old_nonce {
                        account.nonce = u.old_nonce;
                    }
                    // Recovery balance for each (account_id, sub_account_id, coin_id)
                    let balance = &mut balances
                        .iter_mut()
                        .find(|b| b.account_id == u.account_id
                        && b.sub_account_id == u.sub_account_id
                        && b.coin_id == u.coin_id)
                        .unwrap_or_else(|| panic!("Balance not found in db but update [id = {}] exist", u.balance_update_id));
                    balance.balance = u.old_balance;
                }
                if let Some(update) = pubkey_update{
                    if account.nonce > update.old_nonce {
                        account.nonce = update.old_nonce;
                    }
                };

                for u in order_updates {
                    // Recovery slot for each (account_id, sub_account_id, slot_id)
                    let order_slot = &mut order_slots
                        .iter_mut()
                        .find(|o| o.account_id == u.account_id
                        && o.sub_account_id == u.sub_account_id
                        && o.slot_id == u.slot_id)
                        .unwrap_or_else(|| panic!("Order slot not found in db but update [id = {}] exist", u.update_order_id));
                    // `old_order_nonce` in db for example
                    // "[64,\"0\"]"
                    let json_string: String = serde_json::from_value(u.old_order_nonce).unwrap();
                    let (order_nonce, residue): (i64, BigDecimal) = serde_json::from_str(&json_string).unwrap();
                    order_slot.order_nonce = order_nonce;
                    order_slot.residue = residue;
                }
            },
            None => {
                transaction.commit().await?;
            }
        }

        Ok(AccountSnapshot {
            account,
            balances,
            order_slots,
            block_number
        })
    }

    pub async fn sub_account_balances(&mut self, account_id: i64, token_id: i32) -> QueryResult<Vec<StorageBalance>> {
        let start = Instant::now();

        let account_balances = sqlx::query_as!(
            StorageBalance,
            r#"SELECT * FROM balances WHERE account_id = $1 AND coin_id = $2"#,
            account_id,
            token_id
        )
            .fetch_all(self.0.conn())
            .await?;

        metrics::histogram!(
            "sql.chain.account.sub_account_balances",
            start.elapsed()
        );

        Ok(account_balances)
    }

    pub async fn sub_account_balance_of_token(&mut self, account_id: i64, sub_account_id: i32, token_id: i32) -> QueryResult<Option<StorageBalance>> {
        let account_balance = sqlx::query_as!(
            StorageBalance,
            r#"SELECT * FROM balances WHERE account_id = $1 AND sub_account_id = $2 AND coin_id = $3"#,
            account_id,
            sub_account_id,
            token_id
        )
            .fetch_optional(self.0.conn())
            .await?;

        Ok(account_balance)
    }

    pub async fn account_create_time(&mut self, address: &[u8]) -> QueryResult<DateTime<Utc>> {
        let entry = sqlx::query_as!(
            AccountCreatedAt,
            r#"
                SELECT b.created_at as "created_at!" FROM account_creates a INNER JOIN blocks b
                ON a.block_number = b.number
                WHERE a.address = $1
            "#,
            address
        )
            .fetch_one(self.0.conn())
            .await?;

        Ok(entry.created_at)
    }

    pub async fn is_white_submitter(&mut self, sub_account_ids: &[i32], pubkey_hash: &[u8]) -> QueryResult<bool> {
        let white_submitters = sqlx::query_as!(
                StorageWhiteSubmitter,
                r#"SELECT w.* FROM tx_submitter_whitelist as w INNER JOIN accounts as a
                 ON w.submitter_account_id = a.id
                 WHERE w.sub_account_id = ANY($1) AND a.pubkey_hash = $2"#,
                sub_account_ids,
                pubkey_hash
            )
            .fetch_all(self.0.conn())
            .await?;

        Ok(!white_submitters.is_empty())
    }

    pub async fn add_white_submitter(&mut self,sub_account_id:i32,submitter_account_id:i64)-> QueryResult<()> {
        sqlx::query!(
                r#"
                INSERT INTO tx_submitter_whitelist ( sub_account_id, submitter_account_id )
                VALUES ( $1, $2 )
                "#,
                sub_account_id,
                submitter_account_id,
                    )
            .execute(self.0.conn())
            .await?;
        Ok(())

    }

    pub async fn delete_white_submitter(&mut self,sub_account_id:i32,submitter_account_id:i64)-> QueryResult<()> {
        sqlx::query!(
                r#"
                DELETE FROM tx_submitter_whitelist  WHERE sub_account_id=$1 AND submitter_account_id=$2
                "#,
                sub_account_id,
                submitter_account_id,
                    )
            .execute(self.0.conn())
            .await?;
        Ok(())

    }

    /// load submitter whitelist from table
    pub async fn load_submitter_whitelist(&mut self) -> QueryResult<Vec<StorageWhiteSubmitter>> {
        let white_submitters = sqlx::query_as!(
                StorageWhiteSubmitter,
                r#"SELECT * FROM tx_submitter_whitelist"#,
            )
            .fetch_all(self.0.conn())
            .await?;

        Ok(white_submitters)
    }
}
