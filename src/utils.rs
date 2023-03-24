use std::collections::HashMap;
use bigdecimal::num_bigint::{BigUint, ToBigInt};
use zklink_storage::chain::account::records::StorageBalance;
use zklink_types::{ChainId, Deposit, SubAccountId, TokenId, ZkLinkAddress};
use zklink_utils::BigUintSerdeWrapper;
use serde::{Deserialize, Serialize};

pub type SerialId = u64;
pub type SubAccountBalances = HashMap<SubAccountId, HashMap<TokenId, BigUintSerdeWrapper>>;
pub type UnprocessedPriorityOps  = HashMap<SerialId, PublicData>;

#[derive(Debug, Serialize, Deserialize,Clone)]
pub enum PublicData{
    Deposit(DepositData),
    FullExit
}

#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct DepositData{
    chain_id: ChainId,
    sub_account_id: SubAccountId,
    l2_target_token_id: TokenId,
    l1_source_token_id: TokenId,
    amount: BigUint,
    owner: ZkLinkAddress,
}

impl From<Deposit> for DepositData {
    fn from(value: Deposit) -> Self {
        Self{
            chain_id: value.from_chain_id,
            sub_account_id: value.sub_account_id,
            l2_target_token_id: value.l2_target_token,
            l1_source_token_id: value.l1_source_token,
            amount: value.amount,
            owner: value.to,
        }
    }
}

pub fn convert_balance_resp(balances: Vec<StorageBalance>) -> SubAccountBalances {
    let mut resp: SubAccountBalances = HashMap::new();
    for balance in balances.iter() {
        let sub_account_id = SubAccountId::from(balance.sub_account_id as u8);
        let token_id = TokenId::from(balance.coin_id as u32);
        resp.entry(sub_account_id)
            .or_default()
            .insert(token_id, balance.balance.to_bigint().unwrap().into());
    }
    resp
}