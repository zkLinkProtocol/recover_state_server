use tracing::info;
use recover_state_config::RecoverStateConfig;
use zklink_basic_types::{ChainId, SubAccountId};
use zklink_crypto::proof::EncodedSingleProof;
use zklink_types::{AccountId, ZkLinkAddress, TokenId};
use zklink_storage::ConnectionPool;
use zklink_types::block::StoredBlockInfo;
use zklink_utils::BigUintSerdeWrapper;
use crate::exit_proof::create_exit_proof;

#[derive(Serialize, Debug)]
struct ExitProofData {
    pub exit_info: ExitInfo,
    amount: BigUintSerdeWrapper,
    proof: EncodedSingleProof,
}

#[derive(Serialize, Debug)]
pub struct ExitInfo {
    #[serde(skip)]
    pub chain_id: ChainId,
    pub account_address: ZkLinkAddress,
    pub account_id: AccountId,
    pub sub_account_id: SubAccountId,
    pub l1_target_token: TokenId,
    pub l2_source_token: TokenId,
}

#[derive(Clone)]
pub struct ExodusProver{
    config: RecoverStateConfig,
    conn_pool: ConnectionPool
}

impl ExodusProver {
    pub fn new(config: RecoverStateConfig) -> Self {
        let conn_pool = ConnectionPool::new(config.db.url, config.db.pool_size);
        Self{
            config,
            conn_pool
        }
    }

    pub async fn create_proof(&self, mut exit_info: ExitInfo) -> anyhow::Result<(StoredBlockInfo, ExitProofData)>{
        let mut storage = self.conn_pool
            .access_storage()
            .await
            .expect("Storage access failed");

        let timer = std::time::Instant::now();
        let l1_target_token = storage
            .tokens_schema()
            .get_token(*exit_info.l1_target_token as i32)
            .await
            .expect("Db access fail")
            .expect(
                "Token not found. If you're addressing an ERC-20 token by it's symbol, \
                  it may not be available after data restore. Try using token address in that case",
            )
            .token_id;
        let l2_source_token = storage
            .tokens_schema()
            .get_token(*exit_info.l2_source_token as i32)
            .await
            .expect("Db access fail")
            .expect(
                "Token not found.",
            )
            .token_id;

        exit_info.account_address = storage
            .chain()
            .account_schema()
            .account_by_id(*exit_info.account_id as i64)
            .await
            .expect("DB access fail")
            .expect("Account not found in the db")
            .address
            .into();

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
            .expect("Failed to get last verified confirmed block")
            .expect("Block should be existed");

        info!("Restored state from db: {} s", timer.elapsed().as_secs());
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

        let stored_block_info = last_executed_block.stored_block_info(exit_info.chain_id);
        let proof_data = ExitProofData {
            exit_info,
            amount: amount.into(),
            proof,
        };
        Ok((stored_block_info, proof_data))
    }
}