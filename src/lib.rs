mod utils;
mod server;
mod recovered_state;

use serde::{Deserialize, Serialize};
use recover_state_config::RecoverStateConfig;
use zklink_storage::{ConnectionPool, StorageProcessor};
use zklink_types::{AccountId, SubAccountId, TokenId, ZkLinkAddress};
use crate::utils::{convert_balance_resp, convert_to_actix_internal_error, SubAccountBalances};
pub use server::run_server;

#[derive(Debug, Clone, Serialize, Deserialize)]
enum AccountQuery {
    AccountId(AccountId),
    AccountAddress(ZkLinkAddress)
}

#[derive(Clone)]
struct ServerData {
    conn_pool: ConnectionPool,
}

impl ServerData {
    pub fn new(config: RecoverStateConfig) -> ServerData {
        let conn_pool = ConnectionPool::new(config.db.url, config.db.pool_size);
        Self{ conn_pool }
    }

    pub(crate) async fn access_storage(&self) -> actix_web::Result<StorageProcessor<'_>> {
        self.conn_pool
            .access_storage()
            .await
            .map_err(convert_to_actix_internal_error)
    }

    pub(crate) async fn get_balances(&self, query: AccountQuery) -> actix_web::Result<Option<SubAccountBalances>>{
        let mut storage = self.access_storage().await?;
        let account_id = match query {
            AccountQuery::AccountId(id) => id,
            AccountQuery::AccountAddress(address) => {
                let Some(account) = storage.chain()
                    .account_schema()
                    .account_by_address(address.as_bytes())
                    .await
                    .map_err(convert_to_actix_internal_error)? else
                {
                    return Ok(None)
                };
                account.id.into()
            }
        };
        let balances = storage.chain()
            .account_schema()
            .account_balances(account_id.into(),None)
            .await
            .map_err(convert_to_actix_internal_error)?;

        Ok(Some(convert_balance_resp(balances)))
    }

    pub(crate) async fn get_proof(
        &self,
        account_query: AccountQuery,
        sub_account_id: SubAccountId,
        token_id: TokenId
    ) -> actix_web::Result<Option<Vec<u8>>>{
        todo!()
    }

    pub(crate) async fn generate_proof_task(
        &self,
        account_query: AccountQuery,
        sub_account_id: SubAccountId,
        token_id: TokenId
    ) -> actix_web::Result<Option<Vec<u8>>>{
        todo!()
    }
}