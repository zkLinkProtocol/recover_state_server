use serde::{Deserialize, Serialize};
use zklink_types::{ChainId, SubAccountId, TokenId, ZkLinkAddress};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BalanceRequest {
    pub address: ZkLinkAddress,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoredBlockInfoRequest {
    pub chain_id: ChainId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnprocessedDepositRequest {
    pub chain_id: ChainId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenRequest {
    pub token_id: TokenId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProofsRequest {
    pub page: u32,
    pub proofs_num: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BatchExitRequest {
    pub(crate) address: ZkLinkAddress,
    pub(crate) sub_account_id: SubAccountId,
    pub(crate) token_id: TokenId,
}
