#![allow(dead_code)]
use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, info};
use zklink_crypto::params::USD_TOKEN_ID;
use zklink_prover::{ExitInfo, ExitProofData};
use zklink_storage::{ConnectionPool, StorageProcessor};
use zklink_storage::chain::account::records::StorageAccount;
use zklink_storage::prover::records::StoredExitProof;
use zklink_types::{AccountId, ChainId, SubAccountId, TokenId, ZkLinkAddress, ZkLinkTx};
use zklink_types::block::StoredBlockInfo;
use zklink_types::utils::check_source_token_and_target_token;
use crate::acquired_tokens::{AcquiredTokens, TokenInfo};
use crate::proofs_cache::ProofsCache;
use crate::recover_progress::{Progress, RecoverProgress};
use crate::recovered_state::RecoveredState;
use crate::request::BatchExitRequest;
use crate::response::{ExodusResponse, ExodusStatus};
use crate::utils::{convert_balance_resp, UnprocessedPriorityOp, SubAccountBalances, PublicData};

#[derive(Clone)]
pub struct AppData {
    conn_pool: ConnectionPool,
    pub contracts: HashMap<ChainId, ZkLinkAddress>,
    recover_progress: RecoverProgress,
    proofs_cache: ProofsCache,

    pub recovered_state: RecoveredState,
    pub acquired_tokens: AcquiredTokens,
}

impl AppData {
    pub async fn new(conn_pool: ConnectionPool, contracts: HashMap<ChainId, ZkLinkAddress>, proofs_cache: ProofsCache, recover_progress: RecoverProgress) -> AppData {
        info!("Loading accounts state....");
        let timer = Instant::now();
        let recovered_state = RecoveredState::load_from_storage(&conn_pool).await;
        debug!("Load accounts state elapsed time: {} ms", timer.elapsed().as_millis());
        info!("End to load accounts state");

        info!("Loading tokens....");
        let acquired_tokens = AcquiredTokens::load_from_storage(&conn_pool).await;
        debug!("Load tokens elapsed time: {} ms", timer.elapsed().as_millis());
        info!("End to load tokens");

        Self{
            conn_pool,
            contracts,
            recover_progress,
            proofs_cache,
            recovered_state,
            acquired_tokens,
        }
    }

    async fn access_storage(&self) -> anyhow::Result<StorageProcessor<'_>> {
        self.conn_pool
            .access_storage_with_retry()
            .await
    }

    pub(crate) async fn get_balances_by_storage(&self, account_address: ZkLinkAddress) -> Result<SubAccountBalances, ExodusStatus>{
        let mut storage = self.access_storage().await?;
        let Some(StorageAccount{id, ..}) = storage.chain()
            .account_schema()
            .account_by_address(account_address.as_bytes())
            .await? else
        {
            return Err(ExodusStatus::AccountNotExist)
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
    ) -> Result<ExitProofData, ExodusStatus>{
        if !check_source_token_and_target_token(
            exit_info.l2_source_token,
            exit_info.l1_target_token
        ).0 {
            return Err(ExodusStatus::InvalidL1L2Token)
        }
        if let Some(&id) = self.recovered_state
            .account_id_by_address
            .get(&exit_info.account_address)
        {
            exit_info.account_id = id;
        } else {
            return Err(ExodusStatus::AccountNotExist)
        };

        let exit_data = self.proofs_cache
            .get_proof(exit_info)
            .await?;

        Ok(exit_data)
    }

    pub async fn generate_batch_proofs_tasks(
        &self,
        batch_exit_info: BatchExitRequest,
        token_info: &TokenInfo,
        account_id: AccountId
    ) -> Vec<ExitInfo>{
        let mut exit_infos = Vec::new();
        if *batch_exit_info.token_id != USD_TOKEN_ID {
            // get general token
            for &chain_id in token_info.addresses.keys(){
                exit_infos.push(ExitInfo{
                    chain_id,
                    account_address: batch_exit_info.address.clone(),
                    account_id,
                    sub_account_id: batch_exit_info.sub_account_id,
                    l1_target_token: batch_exit_info.token_id,
                    l2_source_token: batch_exit_info.token_id,
                });
            }
        } else {
            // get stable coin token(usdx)
            for (&token_id, token) in self.acquired_tokens
                .usdx_tokens
                .iter()
            {
                for &chain_id in token.addresses.keys(){
                    exit_infos.push(ExitInfo{
                        chain_id,
                        account_address: batch_exit_info.address.clone(),
                        account_id,
                        sub_account_id: batch_exit_info.sub_account_id,
                        l1_target_token: token_id,
                        l2_source_token: batch_exit_info.token_id,
                    });
                }
            }
        }
        exit_infos
    }

    pub(crate) async fn get_proofs(
        &self,
        exit_info: BatchExitRequest
    ) -> Result<Vec<ExitProofData>, ExodusStatus>{
        let (&account_id, token_info) = self.check_exit_info(
            &exit_info.address,
            exit_info.sub_account_id,
            exit_info.token_id
        )?;

        let batch_exit_info = self.generate_batch_proofs_tasks(
            exit_info,
            token_info,
            account_id
        ).await;

        let mut all_exit_data = Vec::new();
        for exit_info in batch_exit_info{
            let exit_data = self.proofs_cache.get_proof(exit_info).await?;
            all_exit_data.push(exit_data)
        }
        Ok(all_exit_data)
    }

    pub(crate) async fn generate_proof_task(
        &self,
        mut exit_info: ExitInfo,
    ) -> Result<(), ExodusStatus>{
        if !check_source_token_and_target_token(
            exit_info.l2_source_token,
            exit_info.l1_target_token
        ).0 {
            return Err(ExodusStatus::InvalidL1L2Token)
        }
        exit_info.account_id = *self.check_exit_info(
            &exit_info.account_address,
            exit_info.sub_account_id,
            exit_info.l2_source_token
        )?.0;
        if self.proofs_cache.cache.contains_key(&exit_info){
            return Err(ExodusStatus::ProofTaskAlreadyExists)
        }

        // Update to database
        let mut storage = self.access_storage().await?;
        storage.prover_schema()
            .insert_exit_task((&exit_info).into())
            .await?;

        // Update to cache
        self.proofs_cache.cache.insert(exit_info, None).await;
        Ok(())
    }

    pub(crate) async fn get_proof_task_location(
        &self,
        mut exit_task: ExitInfo,
    ) -> Result<u64, ExodusStatus>{
        if !check_source_token_and_target_token(
            exit_task.l2_source_token,
            exit_task.l1_target_token
        ).0 {
            return Err(ExodusStatus::InvalidL1L2Token)
        }
        exit_task.account_id = *self.check_exit_info(
            &exit_task.account_address,
            exit_task.sub_account_id,
            exit_task.l2_source_token
        )?.0;
        let mut storage = self.access_storage().await?;
        let remaining_tasks = storage
            .prover_schema()
            .get_remaining_tasks_before_start((&exit_task).into())
            .await?;
        Ok(remaining_tasks as u64)
    }

    pub(crate) async fn generate_proof_tasks(
        &self,
        batch_exit_info: BatchExitRequest,
    ) -> Result<(), ExodusStatus>{
        let (&account_id, token_info) = self.check_exit_info(
            &batch_exit_info.address,
            batch_exit_info.sub_account_id,
            batch_exit_info.token_id
        )?;

        // Generate all exit task by BatchExitRequest
        let batch_exit_tasks = self.generate_batch_proofs_tasks(
            batch_exit_info,
            token_info,
            account_id
        ).await;

        // Returns if any task exists
        if self.proofs_cache.cache.contains_key(batch_exit_tasks.first().unwrap()){
            return Err(ExodusStatus::ProofTaskAlreadyExists)
        }

        // Update to database
        let mut storage = self.access_storage().await?;
        storage.prover_schema()
            .insert_batch_exit_tasks(
                batch_exit_tasks.iter().map(|t|t.into()).collect()
            ).await?;

        // Update to cache
        for exit_task in batch_exit_tasks {
            self.proofs_cache.cache.insert(exit_task, None).await;
        }
        Ok(())
    }

    pub(crate) async fn get_unprocessed_priority_ops(&self, chain_id: ChainId) -> Result<Vec<UnprocessedPriorityOp>, ExodusStatus>{
        let mut storage = self.access_storage().await?;
        let priority_ops = storage.chain()
            .operations_schema()
            .get_unprocessed_priority_txs(*chain_id as i16)
            .await?;
        let unprocessed_priority_ops = priority_ops.into_iter()
            .map(|(serial_id, tx)|{
                UnprocessedPriorityOp{
                    serial_id,
                    pub_data: match tx {
                        ZkLinkTx::Deposit(op) => PublicData::Deposit((*op).into()),
                        ZkLinkTx::FullExit(_) => PublicData::FullExit,
                        _ => unreachable!()
                    }
                }
            })
            .collect();
        Ok(unprocessed_priority_ops)
    }

    pub(crate) fn get_contracts(&self) -> ExodusResponse<HashMap<ChainId, ZkLinkAddress>> {
        ExodusResponse::Ok().data(self.contracts.clone())
    }

    pub(crate) async fn get_proofs_by_id(&self, id: Option<u32>, num: u32) -> Result<Vec<StoredExitProof>, ExodusStatus> {
        let mut storage = self.access_storage().await?;
        let proofs = storage.prover_schema()
            .get_latest_proofs_by_id(id.map(|id|id as i64), num as i64)
            .await?;
        Ok(proofs)
    }

    pub(crate) fn get_stored_block_info(&self, chain_id: ChainId) -> Result<StoredBlockInfo, ExodusStatus> {
        if !self.contracts.contains_key(&chain_id) {
            return Err(ExodusStatus::ChainNotExist)
        }
        Ok(self.recovered_state.stored_block_info(chain_id))
    }

    pub(crate) async fn get_recover_progress(&self) -> Result<Progress, ExodusStatus> {
        if !self.recover_progress.is_completed_state().await{
            let mut storage = self.access_storage().await?;
            let verified_block_num = storage.chain()
                .block_schema()
                .get_last_block_number()
                .await?;
            drop(storage);
            self.recover_progress
                .update_progress(verified_block_num.into())
                .await;
        }
        Ok(self.recover_progress.get_progress().await)
    }

    fn check_exit_info(
        &self,
        address: &ZkLinkAddress,
        sub_account_id: SubAccountId,
        token_id: TokenId
    ) -> Result<(&AccountId , &TokenInfo), ExodusStatus> {
        let Some(account_id) = self.recovered_state
            .account_id_by_address
            .get(address) else {
            return Err(ExodusStatus::AccountNotExist)
        };
        let Some(token_info) = self.acquired_tokens
            .token_by_id
            .get(&token_id) else {
            return Err(ExodusStatus::TokenNotExist)
        };
        if self.recovered_state.empty_balance(*account_id, sub_account_id, token_info.token_id) {
            return Err(ExodusStatus::NonBalance)
        }

        Ok((account_id, token_info))
    }
}