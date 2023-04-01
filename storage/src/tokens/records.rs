// External imports
use serde::{Deserialize, Serialize};
use sqlx::{types::BigDecimal, FromRow};
// Workspace imports
// Local imports
use chrono::{DateTime, Utc};
use zklink_types::{Token, TokenId, ChainId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, FromRow)]
pub struct DbTokenOfChain {
    pub id: i32,
    pub chain_id: i16,
    pub address: Vec<u8>,
    pub decimals: i16,
    pub fast_withdraw: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, FromRow)]
pub struct DbTokenId {
   pub id: Option<i32>,
}

impl From<DbTokenOfChain> for Token {
    fn from(val: DbTokenOfChain) -> Self {
        Token {
            id: TokenId(val.id as u32),
            chains: vec![ChainId(val.chain_id as u8)],
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct DbToken {
    pub token_id: i32,
    pub symbol: String,
    pub price_id: String,
    pub usd_price: BigDecimal,
    pub last_update_time: DateTime<Utc>
}