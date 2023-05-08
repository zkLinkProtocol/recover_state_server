use std::collections::HashMap;
// Built-in deps
use std::time::Instant;
use zklink_types::{Token, TokenId};
// Workspace imports
use self::records::{DbToken, DbTokenOfChain};
use crate::{QueryResult, StorageProcessor};

pub mod records;
mod utils;

/// Tokens schema handles the `tokens` table, providing methods to
/// get and store new tokens.
#[derive(Debug)]
pub struct TokensSchema<'a, 'c>(pub &'a mut StorageProcessor<'c>);

impl<'a, 'c> TokensSchema<'a, 'c> {
    /// Persists the token_price in the database.
    pub async fn store_token_price(&mut self, token: DbToken) -> QueryResult<()> {
        let start = Instant::now();
        sqlx::query!(
            r#"
            INSERT INTO token_price ( token_id, symbol, price_id, usd_price, last_update_time )
            VALUES ( $1, $2, $3, $4, $5)
            ON CONFLICT(token_id) DO UPDATE SET last_update_time=$5, usd_price=$4
            "#,
            token.token_id,
            token.symbol,
            token.price_id,
            token.usd_price,
            token.last_update_time
        )
        .execute(self.0.conn())
        .await?;

        metrics::histogram!("sql.token.store_token", start.elapsed());
        Ok(())
    }

    /// load token from token_price table
    pub async fn load_tokens_price(&mut self) -> QueryResult<Vec<DbToken>> {
        let start = Instant::now();
        let tokens = sqlx::query_as!(
            DbToken,
            r#"
            SELECT * FROM token_price
            ORDER BY token_id ASC
            "#,
        )
        .fetch_all(self.0.conn())
        .await?;

        metrics::histogram!("sql.token.load_tokens_price", start.elapsed());
        Ok(tokens)
    }

    /// Update token `usd_price` and `last_update_time` to token_price table
    pub async fn update_token_price(&mut self, token: DbToken) -> QueryResult<()> {
        let start = Instant::now();
        sqlx::query!(
            r#"
            UPDATE token_price SET usd_price = $1, last_update_time = $2
            where token_id = $3
            "#,
            token.usd_price,
            token.last_update_time,
            token.token_id
        )
        .execute(self.0.conn())
        .await?;

        metrics::histogram!("sql.token.store_token", start.elapsed());
        Ok(())
    }

    /// Loads all the stored tokens from the database.
    /// Alongside with the tokens added via `store_token` method, the default `ETH` token
    /// is returned.
    pub async fn load_tokens(&mut self) -> QueryResult<Vec<DbToken>> {
        let start = Instant::now();
        let tokens = sqlx::query_as!(
            DbToken,
            r#"
            SELECT * FROM token_price
            "#,
        )
        .fetch_all(self.0.conn())
        .await?;

        metrics::histogram!("sql.token.load_tokens", start.elapsed());

        Ok(tokens)
    }

    pub async fn load_chain_tokens(&mut self) -> QueryResult<Vec<DbTokenOfChain>> {
        let chain_tokens = sqlx::query_as!(
            DbTokenOfChain,
            r#"
            SELECT * FROM tokens
            "#
        )
        .fetch_all(self.0.conn())
        .await?;

        Ok(chain_tokens)
    }

    pub async fn load_tokens_from_db(&mut self) -> QueryResult<HashMap<TokenId, Token>> {
        let tokens = self.load_tokens().await?;
        let chain_tokens = self.load_chain_tokens().await?;
        let mut token_by_id: HashMap<TokenId, Token> = HashMap::new();
        for token in &tokens {
            let mut t = Token::new(token.token_id.into());
            for chain_token in &chain_tokens {
                if chain_token.id == token.token_id {
                    t.chains.push(chain_token.chain_id.into());
                }
            }
            token_by_id.insert(t.id, t);
        }
        Ok(token_by_id)
    }

    /// Loads all the stored tokens from the database.
    /// Alongside with the tokens added via `store_token` method, the default `ETH` token
    /// is returned.
    pub async fn save_tokens(&mut self, tokens: Vec<DbTokenOfChain>) -> QueryResult<()> {
        let start = Instant::now();
        let mut transaction = self.0.start_transaction().await.unwrap();
        for token in tokens {
            sqlx::query!(
                r#"
                INSERT INTO tokens (id, chain_id, address, decimals, fast_withdraw) values ($1, $2, $3, $4, $5)
                ON CONFLICT (id, chain_id) DO UPDATE set address = $3, decimals = $4, fast_withdraw = $5
                "#,
                    token.id,
                    token.chain_id,
                    token.address,
                    token.decimals,
                    token.fast_withdraw
            )
                .execute(transaction.conn())
                .await?;
        }
        transaction.commit().await?;

        metrics::histogram!("sql.token.load_tokens", start.elapsed());
        Ok(())
    }

    /// Get token from Database by id
    pub async fn get_token(&mut self, token_id: i32) -> QueryResult<Option<DbToken>> {
        let start = Instant::now();
        let token = sqlx::query_as!(
            DbToken,
            r#"
            SELECT * FROM token_price WHERE token_id = $1
            "#,
            token_id
        )
        .fetch_optional(self.0.conn())
        .await?;

        metrics::histogram!("sql.token.get_count", start.elapsed());
        Ok(token)
    }

    /// Get token of chain from Database by id
    pub async fn get_chain_token(
        &mut self,
        token_id: i32,
        chain_id: i16,
    ) -> QueryResult<Option<DbTokenOfChain>> {
        let chain_token = sqlx::query_as!(
            DbTokenOfChain,
            r#"
            SELECT * FROM tokens WHERE id = $1 AND chain_id = $2
            "#,
            token_id,
            chain_id
        )
        .fetch_optional(self.0.conn())
        .await?;

        Ok(chain_token)
    }
}
