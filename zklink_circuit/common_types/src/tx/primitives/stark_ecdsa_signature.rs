#![allow(dead_code)]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use zklink_utils::ZeroPrefixHexSerde;

#[derive(Debug, Clone, PartialEq)]
pub struct StarkECDSASignature(pub Vec<u8>);

impl fmt::Display for StarkECDSASignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StarkECDSASignature 0x{}",
            hex::encode(self.0.as_slice())
        )
    }
}

impl<'de> Deserialize<'de> for StarkECDSASignature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = ZeroPrefixHexSerde::deserialize(deserializer)?;
        Ok(Self(bytes))
    }
}

impl Serialize for StarkECDSASignature {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        ZeroPrefixHexSerde::serialize(&self.0, serializer)
    }
}
