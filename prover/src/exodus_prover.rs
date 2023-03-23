use tracing::info;
use recover_state_config::RecoverStateConfig;
use zklink_crypto::circuit::account::CircuitAccount;
use zklink_crypto::circuit::CircuitAccountTree;
use zklink_crypto::params::account_tree_depth;
use zklink_storage::ConnectionPool;
use zklink_types::block::Block;
use crate::exit_proof::create_exit_proof;
use crate::exit_type::ExitProofData;
use crate::ExitInfo;

#[derive(Clone)]
pub struct ExodusProver{
    config: RecoverStateConfig,
    conn_pool: ConnectionPool,
    circuit_account_tree: CircuitAccountTree,
    pub last_executed_block: Block,
}

impl ExodusProver {
    pub async fn new(config: RecoverStateConfig) -> Self {
        let conn_pool = ConnectionPool::new(config.db.url.clone(), config.db.pool_size);
        let mut storage = conn_pool
            .access_storage()
            .await
            .expect("Storage access failed");
        // Process unfinished tasks before the last shutdown.
        storage.prover_schema()
            .process_unfinished_tasks()
            .await
            .expect("Storage access failed");
        // loads circuit account tree
        let last_executed_block_number = storage
            .chain()
            .block_schema()
            .get_last_verified_confirmed_block()
            .await
            .expect("Failed to load last verified confirmed block number");
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
        circuit_account_tree.root_hash();
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
        drop(storage);
        Self{
            config,
            conn_pool,
            circuit_account_tree,
            last_executed_block,
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

    pub async fn load_new_task(&self) -> anyhow::Result<Option<ExitInfo>> {
        let mut storage = self.conn_pool
            .access_storage()
            .await?;
        let task = storage.prover_schema()
            .load_exit_proof_task()
            .await?
            .map(|t| {
                info!("Loading new task: {}", t);
                assert!(
                    t.created_at.is_none()
                    && t.finished_at.is_none()
                    && t.proof.is_none()
                    && t.amount.is_none()
                );
                (&t).into()
            });
        Ok(task)
    }

    pub async fn cancel_this_task(&self, exit_info: &ExitInfo) -> anyhow::Result<()> {
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
            amount: Some(amount.into()),
            proof: Some(proof),
        };
        Ok(proof_data)
    }

    pub(crate) async fn store_exit_proof(&self, proof: &ExitProofData) -> anyhow::Result<()>{
        let mut storage = self.conn_pool
            .access_storage()
            .await?;
        storage.prover_schema()
            .store_exit_proof(proof.into())
            .await?;
        Ok(())
    }
}