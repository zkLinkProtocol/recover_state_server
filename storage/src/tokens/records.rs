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

impl Into<Token> for DbTokenOfChain {
    fn into(self) -> Token {
        Token {
            id: TokenId(self.id as u32),
            chains: vec![ChainId(self.chain_id as u8)],
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