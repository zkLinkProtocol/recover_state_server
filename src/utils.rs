use std::collections::HashMap;
use tracing::error;
use bigdecimal::num_bigint::{ToBigInt};
use zklink_storage::chain::account::records::StorageBalance;
use zklink_types::{SubAccountId, TokenId};
use zklink_utils::BigUintSerdeWrapper;

pub type SubAccountBalances = HashMap<SubAccountId, HashMap<TokenId, BigUintSerdeWrapper>>;

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

pub fn convert_to_actix_internal_error(e: anyhow::Error) -> actix_web::Error {
    error!("Encountered a burst error: {}", e);
    actix_web::error::ErrorInternalServerError(e)
}