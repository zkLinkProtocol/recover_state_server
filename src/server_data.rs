#![allow(dead_code)]
use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, info};
use recover_state_config::RecoverStateConfig;
use zklink_crypto::params::{MAX_USD_TOKEN_ID, USD_TOKEN_ID, USDX_TOKEN_ID_UPPER_BOUND};
use zklink_prover::{ExitInfo, ExitProofData};
use zklink_storage::{ConnectionPool, StorageProcessor};
use zklink_storage::chain::account::records::StorageAccount;
use zklink_storage::prover::records::StoredExitInfo;
use zklink_types::{AccountId, AccountMap, ChainId, Token, TokenId, ZkLinkAddress};
use zklink_types::block::{Block, StoredBlockInfo};
use zklink_types::utils::{recover_raw_token, recover_sub_account_by_token};
use crate::response::TokenInfo;
use crate::utils::{BatchExitInfo, convert_balance_resp, convert_to_actix_internal_error, SubAccountBalances};

#[derive(Clone)]
pub struct ServerData {
    conn_pool: ConnectionPool,
    contracts: HashMap<ChainId, ZkLinkAddress>,

    last_block_info: Block,
    token_by_id: HashMap<TokenId, Token>,
    usdx_tokens: HashMap<TokenId, Token>,
    account_id_by_address: HashMap<ZkLinkAddress, AccountId>,
    accounts: AccountMap,
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

        let account_id_by_address = accounts
            .iter()
            .map(|(id, acc)|(acc.address.clone(), *id))
            .collect();

        Self{
            conn_pool,
            contracts,
            last_block_info: last_executed_block,
            token_by_id,
            usdx_tokens,
            account_id_by_address,
            accounts,
        }
    }

    pub(crate) async fn access_storage(&self) -> actix_web::Result<StorageProcessor<'_>> {
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

    pub(crate) async fn get_balances_by_cache(&self, account_address: ZkLinkAddress) -> actix_web::Result<Option<SubAccountBalances>>{
        let Some(&id) = self.account_id_by_address
            .get(&account_address) else {
            return Ok(None)
        };
        let balances = self.accounts
            .get(&id)
            .expect("Account should be exist")
            .get_existing_token_balances();

        let mut resp: SubAccountBalances = HashMap::new();
        for (&token_id, balance) in balances.iter() {
            let sub_account_id = recover_sub_account_by_token(token_id);
            let real_token_id = recover_raw_token(token_id);
            resp.entry(sub_account_id)
                .or_default()
                .insert(real_token_id, balance.reserve0.clone());
        }
        Ok(Some(resp))
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
        let Some(&id) = self.account_id_by_address
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
        let Some(&id) = self.account_id_by_address
            .get(&exit_info.address) else {
            return Err(actix_web::error::ErrorNotFound("Account not found"))
        };
        let Some(token_info) = self.token_by_id
            .get(&exit_info.token_id) else {
            return Err(actix_web::error::ErrorNotFound("Token not found"))
        };

        let mut storage = self.access_storage().await?;
        if *exit_info.token_id != USD_TOKEN_ID {
            for &chain_id in &token_info.chains{
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
            for (&token_id, token) in self.usdx_tokens.iter(){
                for &chain_id in &token.chains{
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

    pub(crate) async fn get_token(&self, token_id: TokenId) -> actix_web::Result<Option<TokenInfo>>{
        let Some(token) = self.token_by_id
            .get(&token_id)
            .cloned() else {
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

    pub(crate) fn get_stored_block_info(&self, chain_id: ChainId) -> Option<StoredBlockInfo> {
        if !self.contracts.contains_key(&chain_id) {
            return None
        }
        Some(self.last_block_info.stored_block_info(chain_id))
    }
}