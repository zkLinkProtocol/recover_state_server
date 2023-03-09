use std::fmt::Formatter;
use tracing::info;
use recover_state_config::RecoverStateConfig;
use zklink_basic_types::{ChainId, SubAccountId};
use zklink_crypto::circuit::account::CircuitAccount;
use zklink_crypto::circuit::CircuitAccountTree;
use zklink_crypto::params::account_tree_depth;
use zklink_crypto::proof::EncodedSingleProof;
use zklink_types::{AccountId, ZkLinkAddress, TokenId, AccountMap};
use zklink_storage::{ConnectionPool, QueryResult};
use zklink_storage::recover_state::records::{StoredExitInfo, StoredExitProof};
use zklink_types::block::StoredBlockInfo;
use zklink_utils::BigUintSerdeWrapper;
use crate::exit_proof::create_exit_proof;

#[derive(Serialize, Debug)]
pub struct ExitProofData {
    pub exit_info: ExitInfo,
    amount: BigUintSerdeWrapper,
    proof: EncodedSingleProof,
}

impl From<ExitProofData> for StoredExitProof  {
    fn from(value: ExitProofData) -> Self {
        Self{
            chain_id: *value.exit_info.chain_id as i16,
            account_id: *value.exit_info.account_id as i64,
            sub_account_id: *value.exit_info.sub_account_id as i16,
            l1_target_token: *value.exit_info.l1_target_token as i32,
            l2_source_token: *value.exit_info.l2_source_token as i32,
            proof: Some(serde_json::to_value(value.proof).unwrap()),
            created_at: None,
            finished_at: None,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct ExitInfo {
    #[serde(skip)]
    pub chain_id: ChainId,
    pub account_address: ZkLinkAddress,
    pub account_id: AccountId,
    pub sub_account_id: SubAccountId,
    pub l1_target_token: TokenId,
    pub l2_source_token: TokenId,
}

impl From<&StoredExitProof> for ExitInfo {
    fn from(value: &StoredExitProof) -> Self {
        Self{
            chain_id: value.chain_id.into(),
            account_address: Default::default(),
            account_id: value.account_id.into(),
            sub_account_id: value.sub_account_id.into(),
            l1_target_token: value.l1_target_token.into(),
            l2_source_token: value.l2_source_token.into(),
        }
    }
}

impl From<&ExitInfo> for StoredExitInfo {
    fn from(value: &ExitInfo) -> Self {
        Self{
            chain_id: *value.chain_id as i16,
            account_id: *value.account_id as i64,
            sub_account_id: *value.sub_account_id as i16,
            l1_target_token: *value.l1_target_token as i32,
            l2_source_token: *value.l2_source_token as i32,
        }
    }
}

impl std::fmt::Display for ExitInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
         write!(
             f, "(chain_id:{}, account_address:{}, account_id:{}, \
             sub_account_id:{}, l1_target_token:{}, l2_source_token:{})",
             self.chain_id, self.account_address, self.account_id,
             self.sub_account_id, self.l1_target_token, self.l2_source_token
         )
    }
}

#[derive(Clone)]
pub struct ExodusProver{
    config: RecoverStateConfig,
    conn_pool: ConnectionPool,
    circuit_account_tree: CircuitAccountTree,
    pub stored_block_info: StoredBlockInfo,
}

impl ExodusProver {
    pub async fn new(config: RecoverStateConfig) -> Self {
        let conn_pool = ConnectionPool::new(config.db.url.clone(), config.db.pool_size);
        let mut storage = conn_pool
            .access_storage()
            .await
            .expect("Storage access failed");
        // Process
        storage.prover_schema()
            .process_unfinished_tasks()
            .await
            .expect("Storage access failed");
        // loads circuit account tree
        let mut circuit_account_tree = CircuitAccountTree::new(account_tree_depth());
        for (id, account) in storage.chain()
            .state_schema()
            .load_circuit_state(last_executed_block_number)
            .await
            .expect("Failed to load verified state")
            .1
        {
            circuit_account_tree.insert(*id, CircuitAccount::from(account));
        }
        // loads the stored block info of last executed block.
        let last_executed_block_number = storage
            .chain()
            .block_schema()
            .get_last_verified_confirmed_block()
            .await
            .expect("Failed to load last verified confirmed block number");
        let last_executed_block = storage
            .chain()
            .block_schema()
            .get_block(last_executed_block_number)
            .await
            .expect("Failed to get last verified confirmed block")
            .expect("Block should be existed");
        let stored_block_info = last_executed_block.stored_block_info(exit_info.chain_id);

        Self{
            config,
            conn_pool,
            circuit_account_tree,
            stored_block_info,
        }
    }

    pub async fn exist_available_workers(&self) -> bool {
        match self.running_tasks_num().await{
            Ok(num) => num < self.max_workers_num,
            Err(e) => {
                info!("Failed to get running tasks num: {}", e);
                false
            }
        }
    }

    /// The number of tasks that are currently generating proof.
    pub async fn running_tasks_num(&self) -> anyhow::Result<u32> {
        let mut storage = self.conn_pool
            .access_storage()
            .await?;
        let running_task_num = storage.prover_schema()
            .count_running_tasks()
            .await?;
        Ok(running_task_num as u32)
    }

    pub async fn load_not_start_tasks(&self) -> anyhow::Result<u32> {
        let mut storage = self.conn_pool
            .access_storage()
            .await?;
        let running_task_num = storage.prover_schema()
            .load_running_tasks()
            .await?;
        Ok(running_task_num as u32)
    }

    pub async fn load_new_task(&self) -> anyhow::Result<Option<ExitInfo>> {
        let mut storage = self.conn_pool
            .access_storage()
            .await?;
        let task = storage.prover_schema()
            .load_exit_proof_task()
            .await?
            .map(|t| {
                assert!(
                    t.proof.is_none()
                    && t.created_at.is_none()
                    && t.finished_at.is_none()
                );
                t.into()
            });
        Ok(task)
    }

    pub async fn cancel_this_task(&self, exit_info: ExitInfo) -> anyhow::Result<()> {
        let mut storage = self.conn_pool
            .access_storage()
            .await?;
        storage.prover_schema()
            .cancel_this_exit_proof_task(exit_info.into())
            .await?;
        Ok(())
    }

    pub async fn check_exit_info(&self, mut exit_info: ExitInfo) -> ExitInfo{
        let mut storage = self.conn_pool
            .access_storage()
            .await
            .expect("Storage access failed");
        storage
            .tokens_schema()
            .get_token(*exit_info.l1_target_token as i32)
            .await
            .expect("Db access fail")
            .expect(
                "Token not found. If you're addressing an ERC-20 token by it's symbol, \
                  it may not be available after data restore. Try using token address in that case",
            );
        storage
            .tokens_schema()
            .get_token(*exit_info.l2_source_token as i32)
            .await
            .expect("Db access fail")
            .expect(
                "Token not found.",
            );

        exit_info.account_address = storage
            .chain()
            .account_schema()
            .account_by_id(*exit_info.account_id as i64)
            .await
            .expect("DB access fail")
            .expect("Account not found in the db")
            .address
            .into();
        exit_info
    }

    pub fn create_exit_proof(&self, exit_info: ExitInfo) -> anyhow::Result<ExitProofData> {
        let (proof, amount) = create_exit_proof(
            &self.config,
            &self.circuit_account_tree,
            exit_info.account_id,
            exit_info.sub_account_id,
            exit_info.l2_source_token,
            exit_info.l1_target_token,
            exit_info.chain_id,
            self.config.layer1.chain_configs.len()
        )
            .expect("Failed to generate exit proof");

        let proof_data = ExitProofData {
            exit_info,
            amount: amount.into(),
            proof,
        };
        Ok(proof_data)
    }

    pub(crate) async fn store_exit_proof(&self, proof: ExitProofData) -> anyhow::Result<()>{
        let mut storage = self.conn_pool
            .access_storage()
            .await?;
        storage.prover_schema()
            .store_exit_proof(proof.into())
            .await?;
        Ok(())
    }
}