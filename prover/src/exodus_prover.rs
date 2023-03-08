use recover_state_config::RecoverStateConfig;
use zklink_storage::ConnectionPool;
use zklink_types::block::StoredBlockInfo;
use crate::exit_proof::create_exit_proof;

struct ExodusProver{
    config: RecoverStateConfig,
    conn_pool: ConnectionPool
}

impl ExodusProver {
    pub fn new(config: RecoverStateConfig) -> Self {
        let conn_pool = ConnectionPool::new(config.db.url, config.db.pool_size);
        Self{
            conn_pool
        }
    }

    pub async fn create_proof(&self) -> (StoredBlockInfo, ExitProofData){
        let mut storage = self.conn_pool
            .access_storage()
            .await
            .expect("Storage access failed");

        let l1_target_token = storage
            .tokens_schema()
            .get_token(l1_target_token)
            .await
            .expect("Db access fail")
            .expect(
                "Token not found. If you're addressing an ERC-20 token by it's symbol, \
                  it may not be available after data restore. Try using token address in that case",
            )
            .token_id;
        let l2_source_token = storage
            .tokens_schema()
            .get_token(l2_source_token)
            .await
            .expect("Db access fail")
            .expect(
                "Token not found.",
            )
            .token_id;
        let address = storage
            .chain()
            .account_schema()
            .account_by_id(account_id as i64)
            .await
            .expect("DB access fail")
            .expect("Account not found in the db")
            .address;

        let last_executed_block_number = storage
            .chain()
            .block_schema()
            .get_last_verified_confirmed_block()
            .await
            .expect("Failed to load last verified confirmed block number");

        let accounts = storage
            .chain()
            .state_schema()
            .load_circuit_state(last_executed_block_number)
            .await
            .expect("Failed to load verified state")
            .1;

        let last_executed_block = storage
            .chain()
            .block_schema()
            .get_block(last_executed_block_number)
            .await
            .expect("Failed to get last verified confirmed block").unwrap();

        vlog::info!("Restored state from db: {} s", timer.elapsed().as_secs());
        let stored_block_info = (
            last_executed_block.block_number,
            last_executed_block.number_of_processed_prior_ops(ChainId(chain_id)),
            last_executed_block
                .get_processable_operations_hash_of_chain(ChainId(chain_id)),
            last_executed_block.timestamp,
            last_executed_block.get_eth_encoded_root(),
            last_executed_block.block_commitment,
            last_executed_block.sync_hash,
        );
        let (proof, amount) = create_exit_proof(
            &self.config,
            accounts,
            AccountId(account_id),
            SubAccountId(sub_account_id),
            l2_source_token.into(),
            l1_target_token.into(),
            ChainId(chain_id),
            self.config.layer1.chain_configs.len()
        )
            .expect("Failed to generate exit proof");

        let proof_data = ExitProofData {
            l2_source_token: l2_source_token.into(),
            l1_target_token: l1_target_token.into(),
            account_id: AccountId(account_id),
            account_address: address.into(),
            amount: amount.into(),
            sub_account_id: SubAccountId(sub_account_id),
            proof,
            chain_id: ChainId(chain_id)
        };
    }
}