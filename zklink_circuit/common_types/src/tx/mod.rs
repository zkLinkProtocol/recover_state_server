//! zklink network L2 transactions.

mod zklink_tx;
mod change_pubkey;
mod forced_exit;
mod primitives;
mod transfer;
mod withdraw;
mod order_matching;

#[cfg(test)]
mod tests;
mod deposit;
mod fullexit;
pub mod validators;

// Re-export transactions.
#[doc(hidden)]
pub use self::{
    change_pubkey::{
        ChangePubKey, CREATE2Data, EthECDSAData, ChangePubKeyAuthData,
    },
    forced_exit::ForcedExit,
    transfer::Transfer,
    withdraw::Withdraw,
    deposit::Deposit,
    fullexit::FullExit,
    order_matching::{OrderMatching, Order},
    zklink_tx::{EthSignData, ZkLinkTx, ZkLinkTxType},
};

// Re-export primitives associated with transactions.
pub use self::primitives::{
    eip1271_signature::EIP1271Signature, layer1_signature::TxLayer1Signature,
    packed_eth_signature::PackedEthSignature, packed_public_key::PackedPublicKey,
    packed_signature::PackedSignature, signature::TxSignature,
    tx_hash::TxHash, stark_ecdsa_signature::StarkECDSASignature,
};

pub(crate) static TRANSACTION_SIGNATURE_ERROR: &str = "\
The transaction signature is incorrect. \
Check if the sender address matches the private key, \
the recipient address is not zero, \
and the amount is correct and packable";
