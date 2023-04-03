//! The declaration of the most primitive types used in zklink network.
//! Most of them are just re-exported from the `web3` crate.

#[macro_use]
mod macros;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::num::ParseIntError;
use std::ops::{Add, Deref, DerefMut, Sub};
use std::str::FromStr;

pub use ethers::{types::{H160, U128, U256, H256}};

basic_type!(
    /// Unique identifier of the slot in the zklink network.
    SlotId,
    u32
);

basic_type!(
    /// Unique identifier of the token in the zklink network.
    TokenId,
    u32
);

basic_type!(
    /// unix timestamp
    TimeStamp,
    u32
);

basic_type!(
    /// Unique identifier of the account in the zklink network.
    AccountId,
    u32
);

basic_type!(
    /// zklink network block sequential index.
    BlockNumber,
    u32
);

basic_type!(
    /// zklink account nonce.
    Nonce,
    u32
);

basic_type!(
    /// Unique identifier of the priority operation in the zklink network.
    PriorityOpId,
    u64
);

basic_type!(
    /// Block number in the Ethereum network.
    EthBlockId,
    u64
);

basic_type!(
    /// Unique identifier of the chain in the network
    ChainId,
    u8
);
basic_type!(
    /// Unique identifier of the SubAccount in the network
    SubAccountId,
    u8
);

