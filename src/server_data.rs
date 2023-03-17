use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, info};
use recover_state_config::RecoverStateConfig;
use zklink_crypto::params::{MAX_USD_TOKEN_ID, USD_TOKEN_ID, USDX_TOKEN_ID_UPPER_BOUND};
use zklink_prover::{ExitInfo, ExitProofData};
use zklink_storage::{ConnectionPool, StorageProcessor};
use zklink_storage::chain::account::records::StorageAccount;
use zklink_storage::prover::records::StoredExitInfo;
use zklink_types::{AccountMap, ChainId, Token, TokenId, ZkLinkAddress};
use zklink_types::block::{Block, StoredBlockInfo};
use crate::response::TokenInfo;
use crate::utils::{BatchExitInfo, convert_balance_resp, convert_to_actix_internal_error, SubAccountBalances};

#[derive(Clone)]
pub struct ServerData {
    conn_pool: ConnectionPool,
    contracts: HashMap<ChainId, ZkLinkAddress>,
    last_block_info: Block,
    pub(crate) token_by_id: HashMap<TokenId, Token>,
    pub(crate) usdx_tokens: HashMap<TokenId, Token>,
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
        let token_by_id = storage
            .tokens_schema()
            .load_tokens_from_db()
            .await
            .expect("reload token from db failed");
        let usdx_tokens = token_by_id
            .iter()
            .filter_map(|(&token_id, token)|
                if USDX_TOKEN_ID_UPPER_BOUND < *token_id
                    && *token_id <= MAX_USD_TOKEN_ID
                { Some((token_id, token.clone())) } else { None}
            )
            .collect();

        drop(storage);
        Self{
            conn_pool,
            contracts,
            _accounts: accounts,
            last_block_info: last_executed_block,
            token_by_id,
            usdx_tokens
        }
    }

    pub(crate) async fn access_storage(&self) -> actix_web::Result<StorageProcessor<'_>> {
        self.conn_pool
            .access_storage()
            .await
            .map_err(convert_to_actix_internal_error)
    }

    pub(crate) async fn get_balances(&self, account_address: ZkLinkAddress) -> actix_web::Result<Option<SubAccountBalances>>{
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
        let mut storage = self.access_storage().await?;
        let Some(StorageAccount{id, ..}) = storage.chain()
            .account_schema()
            .account_by_address(exit_info.address.as_bytes())
            .await
            .map_err(convert_to_actix_internal_error)? else
        {
            return Ok(None)
        };
        let proof = storage.prover_schema()
            .get_proofs(id, *exit_info.sub_account_id as i16, *exit_info.token_id as i32)
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
        token_info: &Token,
    ) -> actix_web::Result<()>{
        let mut storage = self.access_storage().await?;
        let Some(StorageAccount{id, ..}) = storage.chain()
            .account_schema()
            .account_by_address(exit_info.address.as_bytes())
            .await
            .map_err(convert_to_actix_internal_error)? else
        {
            return Ok(())
        };
        if *exit_info.token_id != USD_TOKEN_ID {
            for &chain_id in &token_info.chains{
                storage.prover_schema()
                    .insert_exit_task(StoredExitInfo{
                        chain_id: *chain_id as i16,
                        account_id: id,
                        sub_account_id: *exit_info.sub_account_id as i16,
                        l1_target_token: *exit_info.token_id as i32,
                        l2_source_token: *exit_info.token_id as i32,
                    })
                    .await
                    .map_err(convert_to_actix_internal_error)?;
            }
        } else {
            for (&token_id, token) in self.usdx_tokens.iter(){
                for &chain_id in &token.chains{
                    storage.prover_schema()
                        .insert_exit_task(StoredExitInfo {
                            chain_id: *chain_id as i16,
                            account_id: id,
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

    pub(crate) async fn get_token(&self, token_id: TokenId) -> actix_web::Result<Option<TokenInfo>>{
        let Some(token) = self.token_by_id.get(&token_id).cloned() else {
            return  Ok(None)
        };

        let mut storage = self.access_storage().await?;
        let mut token_info = TokenInfo::new(token.id);
        for chain_id in token.chains{
            let db_token = storage.tokens_schema()
                .get_chain_token(*token.id as i32, *chain_id as i16)
                .await
                .map_err(convert_to_actix_internal_error)?
                .expect("Failed to get chain token");
            token_info.insert_token_address(chain_id, db_token.address.into())
        }

        Ok(Some(token_info))
    }

    pub(crate) fn get_contracts(&self) -> HashMap<ChainId, ZkLinkAddress>{
        self.contracts.clone()
    }

    pub(crate) fn get_stored_block_info(&self, chain_id: ChainId) -> StoredBlockInfo {
        self.last_block_info
            .stored_block_info(chain_id)
    }
}