use serde::{Deserialize, Serialize};
use zklink_types::{ChainId, TokenId, ZkLinkAddress};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BalanceRequest {
    pub address: ZkLinkAddress
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoredBlockInfoRequest {
    pub chain_id: ChainId
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenRequest {
    pub token_id: TokenId
}