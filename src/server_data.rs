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
use zklink_types::{AccountId, ChainId, SubAccountId, TokenId, ZkLinkAddress};
use zklink_types::block::StoredBlockInfo;
use zklink_types::utils::check_source_token_and_target_token;
use crate::acquired_tokens::{AcquiredTokens, TokenInfo};
use crate::recovered_state::RecoveredState;
use crate::request::BatchExitRequest;
use crate::response::{ExodusResponse, ExodusError};
use crate::utils::{convert_balance_resp, SubAccountBalances};

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

    async fn access_storage(&self) -> anyhow::Result<StorageProcessor<'_>> {
        self.conn_pool
            .access_storage()
            .await
    }

    pub(crate) async fn get_balances_by_storage(&self, account_address: ZkLinkAddress) -> Result<SubAccountBalances, ExodusError>{
        let mut storage = self.access_storage().await?;
        let Some(StorageAccount{id, ..}) = storage.chain()
            .account_schema()
            .account_by_address(account_address.as_bytes())
            .await? else
        {
            return Err(ExodusError::AccountNotExist)
        };
        let balances = storage.chain()
            .account_schema()
            .account_balances(id,None)
            .await?;

        Ok(convert_balance_resp(balances))
    }

    pub(crate) async fn get_proof(
        &self,
        mut exit_info: ExitInfo,
    ) -> Result<ExitProofData, ExodusError>{
        if !check_source_token_and_target_token(
            exit_info.l2_source_token,
            exit_info.l1_target_token
        ).0 {
            return Err(ExodusError::InvalidL1L2Token)
        }
        if let Some(&id) = self.recovered_state
            .account_id_by_address
            .get(&exit_info.account_address)
        {
            exit_info.account_id = id;
        } else {
            return Err(ExodusError::AccountNotExist)
        };

        let mut storage = self.access_storage().await?;
        let proof = storage.prover_schema()
            .get_proof_by_exit_info((&exit_info).into())
            .await?;
        let Some(exit_data) = proof.map(|proof| {
            let mut proof: ExitProofData  = proof.into();
            proof.exit_info.account_address = exit_info.account_address;
            proof
        }) else {
            return Err(ExodusError::ExitProofTaskNotExist)
        };

        Ok(exit_data)
    }

    pub(crate) async fn get_proofs(
        &self,
        exit_info: BatchExitRequest
    ) -> Result<Vec<ExitProofData>, ExodusError>{
        let Some(&id) = self.recovered_state
            .account_id_by_address
            .get(&exit_info.address) else {
            return Err(ExodusError::AccountNotExist)
        };
        let mut storage = self.access_storage().await?;
        let proof = storage.prover_schema()
            .get_proofs(
                *id as i64,
                *exit_info.sub_account_id as i16,
                *exit_info.token_id as i32
            )
            .await?;
        let exit_data = proof
            .into_iter()
            .map(|proof|{
                let mut proof: ExitProofData = proof.into();
                proof.exit_info.account_address = exit_info.address.clone();
                proof
            })
            .collect();
        Ok(exit_data)
    }

    pub(crate) async fn generate_proof_task(
        &self,
        mut exit_info: ExitInfo,
    ) -> Result<(), ExodusError>{
        if !check_source_token_and_target_token(
            exit_info.l2_source_token,
            exit_info.l1_target_token
        ).0 {
            return Err(ExodusError::InvalidL1L2Token)
        }
        exit_info.account_id = *self.check_exit_info(
            &exit_info.account_address,
            exit_info.sub_account_id,
            exit_info.l2_source_token
        )?.0;

        let mut storage = self.access_storage().await?;
        storage.prover_schema()
            .insert_exit_task((&exit_info).into())
            .await?;
        Ok(())
    }

    pub(crate) async fn generate_proof_tasks(
        &self,
        exit_info: BatchExitRequest,
    ) -> Result<(), ExodusError>{
        let (&account_id, token_info) = self.check_exit_info(
            &exit_info.address,
            exit_info.sub_account_id,
            exit_info.token_id
        )?;

        let mut storage = self.access_storage().await?;
        if *exit_info.token_id != USD_TOKEN_ID {
            // process general token
            for (&chain_id, _) in &token_info.addresses{
                storage.prover_schema()
                    .insert_exit_task(StoredExitInfo{
                        chain_id: *chain_id as i16,
                        account_id: account_id.into(),
                        sub_account_id: *exit_info.sub_account_id as i16,
                        l1_target_token: *exit_info.token_id as i32,
                        l2_source_token: *exit_info.token_id as i32,
                    })
                    .await?;
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
                            account_id: account_id.into(),
                            sub_account_id: *exit_info.sub_account_id as i16,
                            l1_target_token: *token_id as i32,
                            l2_source_token: *exit_info.token_id as i32,
                        })
                        .await?;
                }
            }
        }

        Ok(())
    }

    pub(crate) fn get_contracts(&self) -> ExodusResponse<HashMap<ChainId, ZkLinkAddress>> {
        ExodusResponse::Ok().data(self.contracts.clone())
    }

    pub(crate) fn get_stored_block_info(&self, chain_id: ChainId) -> Result<StoredBlockInfo, ExodusError> {
        if !self.contracts.contains_key(&chain_id) {
            return Err(ExodusError::ChainNotExist)
        }
        Ok(self.recovered_state.stored_block_info(chain_id))
    }

    fn check_exit_info(
        &self,
        address: &ZkLinkAddress,
        sub_account_id: SubAccountId,
        token_id: TokenId
    ) -> Result<(&AccountId , &TokenInfo), ExodusError> {
        let Some(account_id) = self.recovered_state
            .account_id_by_address
            .get(address) else {
            return Err(ExodusError::AccountNotExist)
        };
        let Some(token_info) = self.acquired_tokens
            .token_by_id
            .get(&token_id) else {
            return Err(ExodusError::TokenNotExist)
        };
        if self.recovered_state.empty_balance(*account_id, sub_account_id, token_info.token_id) {
            return Err(ExodusError::NonBalance)
        }

        Ok((account_id, token_info))
    }
}