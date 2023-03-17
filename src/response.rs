use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use zklink_types::{ChainId, TokenId, ZkLinkAddress};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenInfo {
    token_id: TokenId,
    addresses: HashMap<ChainId, ZkLinkAddress>,
}

impl TokenInfo {
    pub(crate) fn new(token_id: TokenId) -> Self {
        Self{ token_id, addresses: HashMap::new() }
    }

    pub(crate) fn insert_token_address(&mut self, chain_id: ChainId, address:ZkLinkAddress){
        self.addresses.insert(chain_id, address);
    }
}