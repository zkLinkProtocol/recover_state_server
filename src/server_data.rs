use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, info};
use serde::{Deserialize, Serialize};
use recover_state_config::RecoverStateConfig;
use zklink_prover::ExitInfo;
use zklink_storage::{ConnectionPool, StorageProcessor};
use zklink_types::{AccountId, AccountMap, ChainId, TokenId, ZkLinkAddress};
use crate::utils::{convert_balance_resp, convert_to_actix_internal_error, SubAccountBalances};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountQuery {
    AccountId(AccountId),
    AccountAddress(ZkLinkAddress)
}

#[derive(Clone)]
pub struct ServerData {
    conn_pool: ConnectionPool,
    contracts: HashMap<ChainId, ZkLinkAddress>,
    _accounts: AccountMap,
}

impl ServerData {
    pub async fn new(config: RecoverStateConfig) -> ServerData {
        let conn_pool = ConnectionPool::new(config.db.url, config.db.pool_size);
        let mut storage = conn_pool.access_storage()
            .await
            .expect("Failed to access storage");
        let contracts = config.layer1.chain_configs
            .iter()
            .map(|c|(c.chain.chain_id, c.contract.address.clone()))
            .collect();
        info!("Loading accounts state....");
        let timer = Instant::now();
        let last_executed_block_number = storage
            .chain()
            .block_schema()
            .get_last_verified_confirmed_block()
            .await
            .expect("Failed to load last verified confirmed block number");
        let accounts = storage.chain()
            .state_schema()
            .load_circuit_state(last_executed_block_number)
            .await
            .expect("Failed to load verified state")
            .1;
        debug!("Elapsed time: {} s", timer.elapsed().as_secs());
        info!("End to load accounts state");
        drop(storage);
        Self{ conn_pool, contracts, _accounts: accounts }
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
        _exit_info: ExitInfo,
    ) -> actix_web::Result<Option<Vec<u8>>>{
        todo!()
    }

    pub(crate) async fn get_proofs(
        &self,
        _address: ZkLinkAddress,
        _token_id: TokenId
    ) -> actix_web::Result<Option<Vec<u8>>>{
        todo!()
    }

    pub(crate) async fn generate_proof_task(
        &self,
        exit_info: ExitInfo,
    ) -> actix_web::Result<()>{
        let mut storage = self.access_storage().await?;
        storage.prover_schema()
            .insert_exit_task((&exit_info).into())
            .await
            .map_err(convert_to_actix_internal_error)?;
        Ok(())
    }

    pub(crate) fn get_contracts(&self) -> HashMap<ChainId, ZkLinkAddress>{
        self.contracts.clone()
    }
}