use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use zklink_types::{ChainId, TokenId, ZkLinkAddress};
use zklink_crypto::params::{MAX_USD_TOKEN_ID, USDX_TOKEN_ID_UPPER_BOUND};
use zklink_storage::ConnectionPool;
use crate::response::{ExodusResponse, ExodusStatus};

#[derive(Debug, Clone)]
pub struct AcquiredTokens{
    /// All tokens that layer2 registered
    pub token_by_id: HashMap<TokenId, TokenInfo>,
    /// All usdx(usdt, usdc, etc) tokens
    pub usdx_tokens: HashMap<TokenId, TokenInfo>,
}

impl AcquiredTokens {
    pub(crate) async fn load_from_storage(conn_pool: &ConnectionPool) -> Self {
        let mut storage = conn_pool.access_storage()
            .await
            .expect("Failed to access storage");
        let stored_tokens = storage
            .tokens_schema()
            .load_tokens_from_db()
            .await
            .expect("reload token from db failed");
        let mut token_by_id = HashMap::new();
        for (token_id, token) in stored_tokens{
            let mut token_info = TokenInfo::new(token.id);
            for chain_id in token.chains{
                let db_token = storage.tokens_schema()
                    .get_chain_token(*token.id as i32, *chain_id as i16)
                    .await
                    .expect("Failed to get chain token")
                    .expect("Db chain token cannot be None");
                token_info.insert_token_address(chain_id, db_token.address.into())
            }
            token_by_id.insert(token_id, token_info);
        }

        let usdx_tokens = token_by_id
            .iter()
            .filter_map(|(&token_id, token)|
                if USDX_TOKEN_ID_UPPER_BOUND < *token_id
                    && *token_id <= MAX_USD_TOKEN_ID
                { Some((token_id, token.clone())) } else { None}
            )
            .collect();
        Self{ token_by_id, usdx_tokens }
    }

    pub(crate) async fn get_token(&self, token_id: TokenId) -> Result<TokenInfo, ExodusStatus> {
        if let Some(token) = self.token_by_id
            .get(&token_id)
            .cloned()
        {
            Ok(token)
        } else {
            Err(ExodusStatus::TokenNotExist)
        }
    }

    pub(crate) fn tokens(&self) -> ExodusResponse<HashMap<TokenId, TokenInfo>>{
        ExodusResponse::Ok().data(self.token_by_id.clone())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenInfo {
    pub token_id: TokenId,
    pub(crate) addresses: HashMap<ChainId, ZkLinkAddress>,
}

impl TokenInfo {
    fn new(token_id: TokenId) -> Self {
        Self{ token_id, addresses: HashMap::new() }
    }

    fn insert_token_address(&mut self, chain_id: ChainId, address:ZkLinkAddress){
        self.addresses.insert(chain_id, address);
    }
}