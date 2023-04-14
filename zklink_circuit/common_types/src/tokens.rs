use crate::{ChainId, TokenId};
use serde::{Deserialize, Serialize};

/// Token supported in zkLink protocol
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct Token {
    /// id is used for tx signature and serialization
    pub id: TokenId,
    /// chains is used to mark which chain(s) the token can be used
    pub chains: Vec<ChainId>,
}

impl Token {
    pub fn new(id: TokenId) -> Self {
        Self { id, chains: vec![] }
    }
}
