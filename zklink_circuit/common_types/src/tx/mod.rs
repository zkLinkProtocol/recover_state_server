//! zklink network L2 transactions.

mod change_pubkey;
mod forced_exit;
mod order_matching;
mod primitives;
mod transfer;
mod withdraw;
mod zklink_tx;

mod deposit;
mod fullexit;
#[cfg(test)]
mod tests;
pub mod validators;

// Re-export transactions.
#[doc(hidden)]
pub use self::{
    change_pubkey::{CREATE2Data, ChangePubKey, ChangePubKeyAuthData, EthECDSAData},
    deposit::Deposit,
    forced_exit::ForcedExit,
    fullexit::FullExit,
    order_matching::{Order, OrderMatching},
    transfer::Transfer,
    withdraw::Withdraw,
    zklink_tx::{EthSignData, ZkLinkTx, ZkLinkTxType},
};

// Re-export primitives associated with transactions.
pub use self::primitives::{
    eip1271_signature::EIP1271Signature, layer1_signature::TxLayer1Signature,
    packed_eth_signature::PackedEthSignature, packed_public_key::PackedPublicKey,
    packed_signature::PackedSignature, signature::TxSignature,
    stark_ecdsa_signature::StarkECDSASignature, tx_hash::TxHash,
};

pub(crate) static TRANSACTION_SIGNATURE_ERROR: &str = "\
The transaction signature is incorrect. \
Check if the sender address matches the private key, \
the recipient address is not zero, \
and the amount is correct and packable";
