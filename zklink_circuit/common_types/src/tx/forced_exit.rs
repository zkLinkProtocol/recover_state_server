use crate::{helpers::pack_fee_amount, AccountId, Nonce, TokenId, ChainId, TimeStamp, ZkLinkAddress};
use num::{BigUint, Zero};
use validator::Validate;

use crate::account::PubKeyHash;
use crate::Engine;
use serde::{Deserialize, Serialize};
use zklink_basic_types::{SubAccountId};
use zklink_crypto::franklin_crypto::eddsa::PrivateKey;
use zklink_crypto::params::TOKEN_MAX_PRECISION;
use zklink_utils::{format_units, BigUintSerdeAsRadix10Str};

use super::TxSignature;
use crate::tx::validators::*;

/// `ForcedExit` transaction is used to withdraw funds from an unowned
/// account to its corresponding L1 address.
///
/// Caller of this function will pay fee for the operation, and has no
/// control over the address on which funds will be withdrawn. Account
/// to which `ForcedExit` is applied must have no public key hash set.
///
/// This operation is expected to be used in cases when account in L1
/// cannot prove its identity in L2 (e.g. it's an existing smart contract),
/// so the funds won't get "locked" in L2.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ForcedExit {
    /// The chain ID of receiver of the transaction.
    #[validate(custom = "chain_id_validator")]
    pub to_chain_id: ChainId,
    /// zkLink network account ID of the transaction initiator.
    #[validate(custom = "account_validator")]
    pub initiator_account_id: AccountId,
    /// sub-account ID of initiator fee token.
    #[validate(custom = "sub_account_validator")]
    pub initiator_sub_account_id: SubAccountId,
    /// Layer1 address of the account to withdraw funds from.
    /// Also this field represents the address in L1 to which funds will be withdrawn.
    #[validate(custom = "zklink_address_validator")]
    pub target: ZkLinkAddress,
    /// Source sub-account ID of the transaction withdraw.
    #[validate(custom = "sub_account_validator")]
    pub target_sub_account_id: SubAccountId,
    /// Source token and target token of ForcedExit from l2 to l1.
    /// Also represents the token in which fee will be paid.
    #[validate(custom = "token_validaotr")]
    pub l2_source_token: TokenId,
    #[validate(custom = "token_validaotr")]
    pub l1_target_token: TokenId,
    /// Fee for the transaction, need packaging
    #[serde(with = "BigUintSerdeAsRadix10Str")]
    #[validate(custom = "fee_packable")]
    pub fee: BigUint,
    #[validate(custom = "token_validaotr")]
    pub fee_token: TokenId,
    /// Current initiator account nonce.
    #[validate(custom = "nonce_validator")]
    pub nonce: Nonce,
    /// Transaction zkLink signature.
    #[serde(default)]
    pub signature: TxSignature,

    /// Used as request id
    pub ts: TimeStamp,
}

impl ForcedExit {
    /// Creates transaction from all the required fields.
    ///
    /// While `signature` field is mandatory for new transactions, it may be `None`
    /// in some cases (e.g. when restoring the network state from the L1 contract data).
    pub fn new(
        to_chain_id: ChainId,
        initiator_account_id: AccountId,
        initiator_sub_account_id: SubAccountId,
        target: ZkLinkAddress,
        target_sub_account_id: SubAccountId,
        l2_source_token: TokenId,
        l1_target_token: TokenId,
        fee_token: TokenId,
        fee: BigUint,
        nonce: Nonce,
        signature: Option<TxSignature>,
        ts: TimeStamp,
    ) -> Self {
        let tx = Self {
            to_chain_id,
            initiator_account_id,
            initiator_sub_account_id,
            target_sub_account_id,
            target,
            l2_source_token,
            l1_target_token,
            fee,
            fee_token,
            nonce,
            signature: signature.unwrap_or_default(),
            ts,
        };
        tx
    }

    /// Creates a signed transaction using private key and
    /// checks for the transaction correcteness.
    pub fn new_signed(
        to_chain_id: ChainId,
        initiator_account_id: AccountId,
        initiator_sub_account_id: SubAccountId,
        target: ZkLinkAddress,
        target_sub_account_id: SubAccountId,
        l2_token: TokenId,
        l1_token: TokenId,
        fee: BigUint,
        fee_token: TokenId,
        nonce: Nonce,
        private_key: &PrivateKey<Engine>,
        ts: TimeStamp,
    ) -> Result<Self, anyhow::Error> {
        let mut tx = Self::new(
            to_chain_id,
            initiator_account_id,
            initiator_sub_account_id,
            target,
            target_sub_account_id,
            l2_token,
            l1_token,
            fee_token,
            fee,
            nonce,
            None,
            ts,
        );
        tx.signature = TxSignature::sign_musig(private_key, &tx.get_bytes());
        if !tx.check_correctness() {
            anyhow::bail!(crate::tx::TRANSACTION_SIGNATURE_ERROR);
        }
        Ok(tx)
    }

    /// Encodes the transaction data as the byte sequence according to the zkLink protocol.
    pub fn get_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&[Self::TX_TYPE]);
        out.extend_from_slice(&self.to_chain_id.to_be_bytes());
        out.extend_from_slice(&self.initiator_account_id.to_be_bytes());
        out.extend_from_slice(&self.initiator_sub_account_id.to_be_bytes());
        out.extend_from_slice(&self.target.as_bytes());
        out.extend_from_slice(&self.target_sub_account_id.to_be_bytes());
        out.extend_from_slice(&(*self.l2_source_token as u16).to_be_bytes());
        out.extend_from_slice(&(*self.l1_target_token as u16).to_be_bytes());
        out.extend_from_slice(&(*self.fee_token as u16).to_be_bytes());
        out.extend_from_slice(&pack_fee_amount(&self.fee));
        out.extend_from_slice(&self.nonce.to_be_bytes());
        out.extend_from_slice(&self.ts.to_be_bytes());
        out
    }

    pub fn check_correctness(&self) -> bool {
        match self.validate() {
            Ok(_) => true,
            Err(_) => false
        }
    }

    /// Restores the `PubKeyHash` from the transaction signature.
    pub fn verify_signature(&self) -> Option<PubKeyHash> {
        self.signature
            .verify_musig(&self.get_bytes())
            .map(|pub_key| PubKeyHash::from_pubkey(&pub_key))
    }

    /// Get the first part of the message we expect to be signed by Ethereum account key.
    /// The only difference is the missing `no_sub_account_idnce` since it's added at the end of the transactions
    /// batch message. The format is:
    ///
    /// ForcedExit {token} to: {target}
    /// [Fee: {fee} {token}]
    ///
    /// Note that the second line is optional.
    pub fn get_ethereum_sign_message_part(&self, l2_source_token_symbol: &str, fee_token_symbol: &str) -> String {
        let mut message = format!(
            "ForcedExit {token} to: {to}",
            token = l2_source_token_symbol,
            to = self.target.to_string()
        );
        if !self.fee.is_zero() {
            message.push_str(
                format!(
                    "\nFee: {fee} {token}",
                    fee = format_units(&self.fee, TOKEN_MAX_PRECISION as u8),
                    token = fee_token_symbol,
                )
                .as_str(),
            );
        }
        message
    }

    /// Gets message that should be signed by Ethereum keys of the account for 2-Factor authentication.
    pub fn get_ethereum_sign_message(&self, l2_source_token_symbol: &str, fee_token_symbol: &str) -> String {
        let mut message = self.get_ethereum_sign_message_part(l2_source_token_symbol, fee_token_symbol);
        message.push_str(format!("\nNonce: {}", self.nonce).as_str());
        message
    }
}
