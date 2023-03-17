use num::bigint::ToBigInt;
use serde::{Deserialize, Serialize};
use zklink_basic_types::{AccountId, ChainId, SubAccountId, TokenId};
use zklink_crypto::proof::EncodedSingleProof;
use zklink_storage::prover::records::{StoredExitInfo, StoredExitProof};
use zklink_types::ZkLinkAddress;
use zklink_utils::BigUintSerdeWrapper;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExitProofData {
    pub exit_info: ExitInfo,
    pub amount: Option<BigUintSerdeWrapper>,
    pub proof: Option<EncodedSingleProof>,
}

impl From<&ExitProofData> for StoredExitProof  {
    fn from(value: &ExitProofData) -> Self {
        Self{
            chain_id: *value.exit_info.chain_id as i16,
            account_id: *value.exit_info.account_id as i64,
            sub_account_id: *value.exit_info.sub_account_id as i16,
            l1_target_token: *value.exit_info.l1_target_token as i32,
            l2_source_token: *value.exit_info.l2_source_token as i32,
            amount: value.amount.as_ref().map(|amount|amount.0.to_bigint().unwrap().into()),
            proof: value.proof.clone().map(|proof|serde_json::to_value(proof).unwrap()),
            created_at: None,
            finished_at: None,
        }
    }
}

impl From<StoredExitProof> for ExitProofData  {
    fn from(value: StoredExitProof) -> Self {
        Self{
            exit_info: (&value).into(),
            proof: value.proof.map(|proof|serde_json::from_value(proof).unwrap()),
            amount: value.amount.map(|amount|amount.to_bigint().unwrap().into()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExitInfo {
    pub chain_id: ChainId,
    pub account_address: ZkLinkAddress,
    pub account_id: AccountId,
    pub sub_account_id: SubAccountId,
    pub l1_target_token: TokenId,
    pub l2_source_token: TokenId,
}

impl From<&StoredExitProof> for ExitInfo {
    fn from(value: &StoredExitProof) -> Self {
        Self{
            chain_id: value.chain_id.into(),
            account_address: Default::default(),
            account_id: value.account_id.into(),
            sub_account_id: value.sub_account_id.into(),
            l1_target_token: value.l1_target_token.into(),
            l2_source_token: value.l2_source_token.into(),
        }
    }
}

impl From<&ExitInfo> for StoredExitInfo {
    fn from(value: &ExitInfo) -> Self {
        Self{
            chain_id: *value.chain_id as i16,
            account_id: *value.account_id as i64,
            sub_account_id: *value.sub_account_id as i16,
            l1_target_token: *value.l1_target_token as i32,
            l2_source_token: *value.l2_source_token as i32,
        }
    }
}

impl std::fmt::Display for ExitInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "(chain_id:{}, account_address:{:?}, account_id:{}, \
             sub_account_id:{}, l1_target_token:{}, l2_source_token:{})",
            self.chain_id, self.account_address, self.account_id,
            self.sub_account_id, self.l1_target_token, self.l2_source_token
        )
    }
}