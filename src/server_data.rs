#![allow(dead_code)]
use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, info};
use recover_state_config::RecoverStateConfig;
use zklink_crypto::params::USD_TOKEN_ID;
use zklink_prover::{ExitInfo, ExitProofData};
use zklink_storage::{ConnectionPool, StorageProcessor};
use zklink_storage::chain::account::records::StorageAccount;
use zklink_storage::prover::records::StoredExitInfo;
use zklink_types::{ChainId, ZkLinkAddress};
use zklink_types::block::StoredBlockInfo;
use crate::acquired_tokens::AcquiredTokens;
use crate::recovered_state::RecoveredState;
use crate::utils::{BatchExitInfo, convert_balance_resp, convert_to_actix_internal_error, SubAccountBalances};

#[derive(Clone)]
pub struct ServerData {
    conn_pool: ConnectionPool,
    contracts: HashMap<ChainId, ZkLinkAddress>,

    pub recovered_state: RecoveredState,
    pub acquired_tokens: AcquiredTokens,
}

impl ServerData {
    pub async fn new(config: RecoverStateConfig) -> ServerData {
        let conn_pool = ConnectionPool::new(config.db.url, config.db.pool_size);
        let contracts = config.layer1.chain_configs
            .iter()
            .map(|c|(c.chain.chain_id, c.contract.address.clone()))
            .collect();

        info!("Loading accounts state....");
        let timer = Instant::now();
        let recovered_state = RecoveredState::load_from_storage(&conn_pool).await;
        debug!("Load accounts state elapsed time: {} s", timer.elapsed().as_secs());
        info!("End to load accounts state");

        info!("Loading tokens....");
        let acquired_tokens = AcquiredTokens::load_from_storage(&conn_pool).await;
        debug!("Load tokens elapsed time: {} s", timer.elapsed().as_secs());
        info!("End to load tokens");

        Self{
            conn_pool,
            contracts,
            recovered_state,
            acquired_tokens,
        }
    }

    async fn access_storage(&self) -> actix_web::Result<StorageProcessor<'_>> {
        self.conn_pool
            .access_storage()
            .await
            .map_err(convert_to_actix_internal_error)
    }

    pub(crate) async fn get_balances_by_storage(&self, account_address: ZkLinkAddress) -> actix_web::Result<Option<SubAccountBalances>>{
        let mut storage = self.access_storage().await?;
        let Some(StorageAccount{id, ..}) = storage.chain()
            .account_schema()
            .account_by_address(account_address.as_bytes())
            .await
            .map_err(convert_to_actix_internal_error)? else
        {
            return Ok(None)
        };
        let balances = storage.chain()
            .account_schema()
            .account_balances(id,None)
            .await
            .map_err(convert_to_actix_internal_error)?;

        Ok(Some(convert_balance_resp(balances)))
    }

    pub(crate) async fn get_proof(
        &self,
        exit_info: ExitInfo,
    ) -> actix_web::Result<Option<ExitProofData>>{
        let mut storage = self.access_storage().await?;
        let proof = storage.prover_schema()
            .get_proof_by_exit_info((&exit_info).into())
            .await
            .map_err(convert_to_actix_internal_error)?;
        let exit_data = proof.map(|proof|proof.into());
        Ok(exit_data)
    }

    pub(crate) async fn get_proofs(
        &self,
        exit_info: BatchExitInfo
    ) -> actix_web::Result<Option<Vec<ExitProofData>>>{
        let Some(&id) = self.recovered_state
            .account_id_by_address
            .get(&exit_info.address) else {
            return Ok(None)
        };
        let mut storage = self.access_storage().await?;
        let proof = storage.prover_schema()
            .get_proofs(
                *id as i64,
                *exit_info.sub_account_id as i16,
                *exit_info.token_id as i32
            )
            .await
            .map_err(convert_to_actix_internal_error)?;
        let exit_data = proof
            .into_iter()
            .map(|proof|proof.into())
            .collect();
        Ok(Some(exit_data))
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

    pub(crate) async fn generate_proof_tasks(
        &self,
        exit_info: BatchExitInfo,
    ) -> actix_web::Result<()>{
        let Some(&id) = self.recovered_state
            .account_id_by_address
            .get(&exit_info.address) else {
            return Err(actix_web::error::ErrorNotFound("Account not found"))
        };
        let Some(token_info) = self.acquired_tokens
            .token_by_id
            .get(&exit_info.token_id) else {
            return Err(actix_web::error::ErrorNotFound("Token not found"))
        };

        let mut storage = self.access_storage().await?;
        if *exit_info.token_id != USD_TOKEN_ID {
            // process general token
            for (&chain_id, _) in &token_info.addresses{
                storage.prover_schema()
                    .insert_exit_task(StoredExitInfo{
                        chain_id: *chain_id as i16,
                        account_id: id.into(),
                        sub_account_id: *exit_info.sub_account_id as i16,
                        l1_target_token: *exit_info.token_id as i32,
                        l2_source_token: *exit_info.token_id as i32,
                    })
                    .await
                    .map_err(convert_to_actix_internal_error)?;
            }
        } else {
            // process stable coin token(usdx)
            for (&token_id, token) in self.acquired_tokens
                .usdx_tokens
                .iter()
            {
                for (&chain_id, _) in &token.addresses{
                    storage.prover_schema()
                        .insert_exit_task(StoredExitInfo {
                            chain_id: *chain_id as i16,
                            account_id: id.into(),
                            sub_account_id: *exit_info.sub_account_id as i16,
                            l1_target_token: *token_id as i32,
                            l2_source_token: *exit_info.token_id as i32,
                        })
                        .await
                        .map_err(convert_to_actix_internal_error)?;
                }
            }
        }

        Ok(())
    }

    pub(crate) fn get_contracts(&self) -> HashMap<ChainId, ZkLinkAddress>{
        self.contracts.clone()
    }

    pub(crate) fn get_stored_block_info(&self, chain_id: ChainId) -> Option<StoredBlockInfo> {
        if !self.contracts.contains_key(&chain_id) {
            return None
        }
        self.recovered_state.stored_block_info(chain_id)
    }
}