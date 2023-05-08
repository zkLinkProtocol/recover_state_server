#![allow(dead_code)]
mod acquired_tokens;
mod proofs_cache;
mod recover_progress;
mod recovered_state;

pub use acquired_tokens::{AcquiredTokens, TokenInfo};
pub use proofs_cache::ProofsCache;
pub use recover_progress::{Progress, RecoverProgress};
pub use recovered_state::RecoveredState;

use bigdecimal::num_bigint::ToBigInt;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::OnceCell;
use tokio::time::interval;
use tracing::{debug, info, warn};

use zklink_crypto::params::USD_TOKEN_ID;
use zklink_prover::exit_type::{ProofId, ProofInfo};
use zklink_prover::{ExitInfo, ExitProofData};
use zklink_storage::chain::account::records::{StorageAccount, StorageBalance};
use zklink_storage::{ConnectionPool, StorageProcessor};
use zklink_types::block::StoredBlockInfo;
use zklink_types::utils::check_source_token_and_target_token;
use zklink_types::{AccountId, ChainId, SubAccountId, TokenId, ZkLinkAddress, ZkLinkTx};

use crate::request::BatchExitRequest;
use crate::response::{
    ExodusResponse, ExodusStatus, PendingTasksCount, Proofs, PublicData, SubAccountBalances,
    TaskId, UnprocessedPriorityOp,
};

const GET_PROOFS_NUM_LIMIT: u32 = 100;

pub struct AppData {
    conn_pool: ConnectionPool,

    pub contracts: HashMap<ChainId, ZkLinkAddress>,
    pub(crate) recover_progress: RecoverProgress,
    proofs_cache: ProofsCache,

    pub recovered_state: OnceCell<RecoveredState>,
    pub acquired_tokens: OnceCell<AcquiredTokens>,
}

impl AppData {
    pub async fn new(
        conn_pool: ConnectionPool,
        contracts: HashMap<ChainId, ZkLinkAddress>,
        proofs_cache: ProofsCache,
        recover_progress: RecoverProgress,
    ) -> AppData {
        Self {
            conn_pool,
            contracts,
            recover_progress,
            proofs_cache,
            recovered_state: Default::default(),
            acquired_tokens: Default::default(),
        }
    }

    pub fn is_not_sync_completed(&self) -> bool {
        !self.acquired_tokens.initialized()
            || !self.acquired_tokens.initialized()
            || !self.recover_progress.is_completed()
    }

    pub fn recovered_state(&self) -> &RecoveredState {
        self.recovered_state.get().unwrap()
    }

    pub fn acquired_tokens(&self) -> &AcquiredTokens {
        self.acquired_tokens.get().unwrap()
    }

    // Periodically clean up blacklisted users (to prevent users from requesting too many proof tasks)
    pub async fn black_list_escaping(self: Arc<Self>, clean_interval: u32){
        let mut storage = self.access_storage().await;
        let mut ticker = interval(Duration::from_secs(10));
        loop{
            if let Err(err) = storage
                .recover_schema()
                .clean_escaped_user(clean_interval)
                .await
            {
                warn!("Failed to clean escaped user, err: {}", err);
            }

            ticker.tick().await;
        }
    }

    pub(crate) async fn sync_recover_progress(self: Arc<Self>) {
        self.recover_progress
            .sync_from_database(&self.conn_pool)
            .await;

        self.recovered_state
            .get_or_init(|| async {
                info!("Loading accounts state....");
                let timer = Instant::now();
                let recovered_state = RecoveredState::load_from_storage(&self.conn_pool).await;
                debug!(
                    "Load accounts state elapsed time: {} ms",
                    timer.elapsed().as_millis()
                );
                info!("End to load accounts state");
                recovered_state
            })
            .await;
        self.acquired_tokens
            .get_or_init(|| async {
                info!("Loading tokens....");
                let timer = Instant::now();
                let acquired_tokens = AcquiredTokens::load_from_storage(&self.conn_pool).await;
                debug!(
                    "Load tokens elapsed time: {} ms",
                    timer.elapsed().as_millis()
                );
                info!("End to load tokens");
                acquired_tokens
            })
            .await;
    }

    async fn access_storage(&self) -> StorageProcessor<'_> {
        self.conn_pool.access_storage_with_retry().await
    }

    pub(crate) async fn get_balances_by_storage(
        &self,
        account_address: ZkLinkAddress,
    ) -> Result<SubAccountBalances, ExodusStatus> {
        let mut storage = self.access_storage().await;
        let Some(StorageAccount{id, ..}) = storage.chain()
            .account_schema()
            .account_by_address(account_address.as_bytes())
            .await? else
        {
            return Err(ExodusStatus::AccountNotExist)
        };
        let balances = storage
            .chain()
            .account_schema()
            .account_balances(id, None)
            .await?;

        Ok(convert_balance_resp(balances))
    }

    pub(crate) async fn get_proof(
        &self,
        mut exit_info: ExitInfo,
    ) -> Result<ExitProofData, ExodusStatus> {
        if !check_source_token_and_target_token(
            exit_info.l2_source_token,
            exit_info.l1_target_token,
        )
        .0
        {
            return Err(ExodusStatus::InvalidL1L2Token);
        }
        if let Some(&id) = self
            .recovered_state()
            .account_id_by_address
            .get(&exit_info.account_address)
        {
            exit_info.account_id = id;
        } else {
            return Err(ExodusStatus::AccountNotExist);
        };

        let exit_data = self.proofs_cache.get_proof(exit_info).await?;

        Ok(exit_data)
    }

    pub(crate) async fn get_proofs(
        &self,
        exit_info: BatchExitRequest,
    ) -> Result<Vec<ExitProofData>, ExodusStatus> {
        let (&account_id, token_info) = self.check_exit_info(
            &exit_info.address,
            exit_info.sub_account_id,
            exit_info.token_id,
        )?;

        let batch_exit_info = self
            .generate_batch_proofs_tasks(exit_info, token_info, account_id)
            .await;

        let mut all_exit_data = Vec::new();
        for exit_info in batch_exit_info {
            let exit_data = self.proofs_cache.get_proof(exit_info).await?;
            all_exit_data.push(exit_data)
        }
        Ok(all_exit_data)
    }

    pub(crate) async fn get_proof_task_id(
        &self,
        mut exit_task: ExitInfo,
    ) -> Result<TaskId, ExodusStatus> {
        if !check_source_token_and_target_token(
            exit_task.l2_source_token,
            exit_task.l1_target_token,
        )
        .0
        {
            return Err(ExodusStatus::InvalidL1L2Token);
        }
        exit_task.account_id = *self
            .check_exit_info(
                &exit_task.account_address,
                exit_task.sub_account_id,
                exit_task.l2_source_token,
            )?
            .0;
        let mut storage = self.access_storage().await;
        let remaining_tasks = storage
            .prover_schema()
            .get_task_id((&exit_task).into())
            .await?
            .map(Into::into)
            .ok_or(ExodusStatus::ExitProofTaskNotExist)?;
        Ok(remaining_tasks)
    }

    pub(crate) async fn generate_proof_task(
        &self,
        mut exit_info: ExitInfo,
    ) -> Result<TaskId, ExodusStatus> {
        if !check_source_token_and_target_token(
            exit_info.l2_source_token,
            exit_info.l1_target_token,
        )
        .0
        {
            return Err(ExodusStatus::InvalidL1L2Token);
        }
        exit_info.account_id = *self
            .check_exit_info(
                &exit_info.account_address,
                exit_info.sub_account_id,
                exit_info.l2_source_token,
            )?
            .0;
        if self.proofs_cache.cache.contains_key(&exit_info) {
            return Err(ExodusStatus::ProofTaskAlreadyExists);
        }


        let mut storage = self.access_storage().await;
        // Check for black list
        let exist_address = storage.recover_schema()
            .check_and_insert_user(exit_info.account_address.as_bytes())
            .await?;
        if exist_address {
            return Err(ExodusStatus::ExistTaskWithinThreeHour);
        }
        // Update to database
        let task_id = storage
            .prover_schema()
            .insert_exit_task((&exit_info).into())
            .await?;

        // Update to cache
        self.proofs_cache
            .cache
            .insert(exit_info, ProofInfo::new(task_id))
            .await;
        Ok(task_id.into())
    }

    pub(crate) async fn generate_proof_tasks(
        &self,
        batch_exit_info: BatchExitRequest,
    ) -> Result<HashMap<ProofId, ExitInfo>, ExodusStatus> {
        let (&account_id, token_info) = self.check_exit_info(
            &batch_exit_info.address,
            batch_exit_info.sub_account_id,
            batch_exit_info.token_id,
        )?;

        // Generate all exit task by BatchExitRequest
        let batch_exit_tasks = self
            .generate_batch_proofs_tasks(batch_exit_info, token_info, account_id)
            .await;

        // Returns if any task exists
        if self
            .proofs_cache
            .cache
            .contains_key(batch_exit_tasks.first().unwrap())
        {
            return Err(ExodusStatus::ProofTaskAlreadyExists);
        }

        // Update to database
        let mut storage = self.access_storage().await;
        let tasks_ids = storage
            .prover_schema()
            .insert_batch_exit_tasks(batch_exit_tasks.iter().map(|t| t.into()).collect())
            .await?;

        // Update to cache
        let mut tasks = HashMap::with_capacity(batch_exit_tasks.len());
        for (exit_task, task_id) in batch_exit_tasks.into_iter().zip(tasks_ids) {
            tasks.insert(task_id as ProofId, exit_task.clone());
            self.proofs_cache
                .cache
                .insert(exit_task, ProofInfo::new(task_id))
                .await;
        }
        Ok(tasks)
    }

    pub(crate) async fn get_unprocessed_priority_ops(
        &self,
        chain_id: ChainId,
    ) -> Result<Vec<UnprocessedPriorityOp>, ExodusStatus> {
        let mut storage = self.access_storage().await;
        let priority_ops = storage
            .chain()
            .operations_schema()
            .get_unprocessed_priority_txs(*chain_id as i16)
            .await?;
        let unprocessed_priority_ops = priority_ops
            .into_iter()
            .map(|(serial_id, tx)| UnprocessedPriorityOp {
                serial_id,
                pub_data: match tx {
                    ZkLinkTx::Deposit(op) => PublicData::Deposit((*op).into()),
                    ZkLinkTx::FullExit(_) => PublicData::FullExit,
                    _ => unreachable!(),
                },
            })
            .collect();
        Ok(unprocessed_priority_ops)
    }

    pub(crate) fn get_contracts(&self) -> ExodusResponse<HashMap<ChainId, ZkLinkAddress>> {
        ExodusResponse::Ok().data(self.contracts.clone())
    }

    pub(crate) async fn running_max_task_id(&self) -> Result<TaskId, ExodusStatus> {
        let mut storage = self.access_storage().await;
        let running_task_id = storage.prover_schema().get_running_max_task_id().await?;
        Ok(running_task_id.into())
    }

    pub(crate) async fn pending_tasks_count(&self) -> Result<PendingTasksCount, ExodusStatus> {
        let mut storage = self.access_storage().await;
        let pending_tasks_count = storage.prover_schema().get_pending_tasks_count().await?;
        Ok(PendingTasksCount {
            count: pending_tasks_count as u32,
        })
    }

    pub(crate) async fn get_proofs_by_page(
        &self,
        page: u32,
        num: u32,
    ) -> Result<Proofs, ExodusStatus> {
        if num > GET_PROOFS_NUM_LIMIT {
            return Err(ExodusStatus::ProofsLoadTooMany);
        }
        let mut storage = self.access_storage().await;
        let proofs = storage
            .prover_schema()
            .get_proofs_by_page(page as i64, num as i64)
            .await?;
        let proofs = proofs
            .into_iter()
            .map(|proof| {
                let mut proof: ExitProofData = proof.into();
                let account = self
                    .recovered_state()
                    .accounts
                    .get(&proof.exit_info.account_id)
                    .unwrap();
                proof.exit_info.account_address = account.address.clone();
                proof
            })
            .collect();

        let total_completed_num = storage
            .prover_schema()
            .get_total_completed_proofs_num()
            .await? as u32;
        Ok(Proofs {
            proofs,
            total_completed_num,
        })
    }

    pub(crate) fn get_stored_block_info(
        &self,
        chain_id: ChainId,
    ) -> Result<StoredBlockInfo, ExodusStatus> {
        if !self.contracts.contains_key(&chain_id) {
            return Err(ExodusStatus::ChainNotExist);
        }
        Ok(self.recovered_state().stored_block_info(chain_id))
    }

    pub(crate) async fn get_recover_progress(&self) -> Result<Progress, ExodusStatus> {
        if !self.recover_progress.is_completed() {
            let mut storage = self.access_storage().await;
            let verified_block_num = storage
                .chain()
                .block_schema()
                .get_last_block_number()
                .await?;
            drop(storage);
            self.recover_progress
                .update_progress(verified_block_num.into());
        }
        Ok(self.recover_progress.get_progress())
    }

    pub async fn generate_batch_proofs_tasks(
        &self,
        batch_exit_info: BatchExitRequest,
        token_info: &TokenInfo,
        account_id: AccountId,
    ) -> Vec<ExitInfo> {
        let mut exit_infos = Vec::new();
        if *batch_exit_info.token_id != USD_TOKEN_ID {
            // get general token
            for &chain_id in token_info.addresses.keys() {
                exit_infos.push(ExitInfo {
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
            for (&token_id, token) in self.acquired_tokens().usdx_tokens.iter() {
                for &chain_id in token.addresses.keys() {
                    exit_infos.push(ExitInfo {
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

    fn check_exit_info(
        &self,
        address: &ZkLinkAddress,
        sub_account_id: SubAccountId,
        token_id: TokenId,
    ) -> Result<(&AccountId, &TokenInfo), ExodusStatus> {
        let Some(account_id) = self.recovered_state()
            .account_id_by_address
            .get(address) else {
            return Err(ExodusStatus::AccountNotExist)
        };
        let Some(token_info) = self.acquired_tokens()
            .token_by_id
            .get(&token_id) else {
            return Err(ExodusStatus::TokenNotExist)
        };
        if self
            .recovered_state()
            .empty_balance(*account_id, sub_account_id, token_info.token_id)
        {
            return Err(ExodusStatus::NonBalance);
        }

        Ok((account_id, token_info))
    }
}

pub fn convert_balance_resp(balances: Vec<StorageBalance>) -> SubAccountBalances {
    let mut resp: SubAccountBalances = HashMap::new();
    for balance in balances.iter() {
        let sub_account_id = SubAccountId::from(balance.sub_account_id as u8);
        let token_id = TokenId::from(balance.coin_id as u32);
        resp.entry(sub_account_id)
            .or_default()
            .insert(token_id, balance.balance.to_bigint().unwrap().into());
    }
    resp
}
