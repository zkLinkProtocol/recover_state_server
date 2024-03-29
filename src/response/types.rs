use bigdecimal::num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zklink_prover::exit_type::ProofId;
use zklink_prover::ExitProofData;
use zklink_types::{ChainId, Deposit, SubAccountId, TokenId, ZkLinkAddress};
use zklink_utils::{BigUintSerdeAsRadix10Str, BigUintSerdeWrapper};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Proofs {
    pub(crate) total_completed_num: u32,
    pub(crate) proofs: Vec<ExitProofData>,
}

pub type SerialId = u64;
pub type SubAccountBalances = HashMap<SubAccountId, HashMap<TokenId, BigUintSerdeWrapper>>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnprocessedPriorityOp {
    pub(crate) serial_id: SerialId,
    pub(crate) pub_data: PublicData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PendingTasksCount {
    pub(crate) count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskId {
    id: ProofId,
}

impl From<i64> for TaskId {
    fn from(value: i64) -> Self {
        Self {
            id: value as ProofId,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PublicData {
    Deposit(DepositData),
    FullExit,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DepositData {
    chain_id: ChainId,
    sub_account_id: SubAccountId,
    l2_target_token_id: TokenId,
    l1_source_token_id: TokenId,
    #[serde(with = "BigUintSerdeAsRadix10Str")]
    amount: BigUint,
    owner: ZkLinkAddress,
}

impl From<Deposit> for DepositData {
    fn from(value: Deposit) -> Self {
        Self {
            chain_id: value.from_chain_id,
            sub_account_id: value.sub_account_id,
            l2_target_token_id: value.l2_target_token,
            l1_source_token_id: value.l1_source_token,
            amount: value.amount,
            owner: value.to,
        }
    }
}
