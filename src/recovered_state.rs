use std::collections::HashMap;
use zklink_storage::ConnectionPool;
use zklink_types::block::{Block, StoredBlockInfo};
use zklink_types::{AccountId, AccountMap, ChainId, SubAccountId, TokenId, ZkLinkAddress};
use zklink_types::utils::{calculate_actual_token, recover_raw_token, recover_sub_account_by_token};
use crate::utils::SubAccountBalances;

#[derive(Debug, Clone)]
pub struct RecoveredState{
    pub last_block_info: Block,
    pub account_id_by_address: HashMap<ZkLinkAddress, AccountId>,
    pub accounts: AccountMap,
}

impl RecoveredState {
    pub(crate) async fn load_from_storage(conn_pool: &ConnectionPool) -> Self {
        let mut storage = conn_pool.access_storage()
            .await
            .expect("Failed to access storage");
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

        // loads the stored block info of last executed block.
        let last_block_info = storage
            .chain()
            .block_schema()
            .get_block(last_executed_block_number)
            .await
            .expect("Failed to get last verified confirmed block")
            .expect("Block should be existed");
        let account_id_by_address = accounts
            .iter()
            .map(|(id, acc)|(acc.address.clone(), *id))
            .collect();

        Self{
            last_block_info,
            account_id_by_address,
            accounts,
        }
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

    pub fn empty_balance(&self, account_id: AccountId, sub_account_id: SubAccountId, token_id: TokenId) -> bool {
        let real_token_id = calculate_actual_token(sub_account_id, token_id);
        let account = self.accounts
            .get(&account_id).unwrap();
        account.get_existing_token_balances()
            .get(&real_token_id)
            .map_or(true, |balance| balance.is_zero())
    }

    pub(crate) fn stored_block_info(&self, chain_id: ChainId) -> Option<StoredBlockInfo> {
        Some(self.last_block_info.stored_block_info(chain_id))
    }
}