use crate::account::PubKeyHash;
use crate::utils::ethereum_sign_message_part;
use crate::Engine;
use crate::{helpers::pack_fee_amount, AccountId, Nonce, TokenId, ZkLinkAddress};
use num::{BigUint, ToPrimitive};
use serde::{Deserialize, Serialize};
use validator::Validate;
use zklink_basic_types::{ChainId, SubAccountId, TimeStamp};
use zklink_crypto::franklin_crypto::eddsa::PrivateKey;
use zklink_crypto::params::TOKEN_MAX_PRECISION;
use zklink_utils::BigUintSerdeAsRadix10Str;

use super::TxSignature;
use crate::tx::validators::*;

/// `Withdraw` transaction performs a withdrawal of funds from zklink account to L1 account.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Withdraw {
    /// Target chain of withdraw.
    #[validate(custom = "chain_id_validator")]
    pub to_chain_id: ChainId,
    /// zkLink network account ID of the transaction initiator.
    #[validate(custom = "account_validator")]
    pub account_id: AccountId,
    /// The source sub-account id of withdraw amount.
    #[validate(custom = "sub_account_validator")]
    pub sub_account_id: SubAccountId,
    /// Address of L1 account to withdraw funds to.
    #[validate(custom = "zklink_address_validator")]
    pub to: ZkLinkAddress,
    /// Source token and target token of withdrawal from l2 to l1.
    /// Also represents the token in which fee will be paid.
    #[validate(custom = "token_validaotr")]
    pub l2_source_token: TokenId,
    #[validate(custom = "token_validaotr")]
    pub l1_target_token: TokenId,
    /// Amount of funds to withdraw, layer1 can not unpack it, do not packaging
    #[serde(with = "BigUintSerdeAsRadix10Str")]
    #[validate(custom = "amount_unpackable")]
    pub amount: BigUint,
    /// Fee for the transaction, need packaging
    #[serde(with = "BigUintSerdeAsRadix10Str")]
    #[validate(custom = "fee_packable")]
    pub fee: BigUint,
    /// Current account nonce.
    #[validate(custom = "nonce_validator")]
    pub nonce: Nonce,
    /// Transaction zkLink signature.
    #[serde(default)]
    pub signature: TxSignature,

    /// Fast withdraw or normal withdraw
    #[validate(custom = "boolean_validator")]
    pub fast_withdraw: u8,
    /// Amount of funds to withdraw.
    #[validate(custom = "withdraw_fee_ratio_validator")]
    pub withdraw_fee_ratio: u16,
    /// Used as request id
    pub ts: TimeStamp,
}

impl Withdraw {
    /// Creates transaction from all the required fields.
    ///
    /// While `signature` field is mandatory for new transactions, it may be `None`
    /// in some cases (e.g. when restoring the network state from the L1 contract data).
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        account_id: AccountId,
        sub_account_id: SubAccountId,
        to_chain_id: ChainId,
        to: ZkLinkAddress,
        l2_source_token: TokenId,
        l1_target_token: TokenId,
        amount: BigUint,
        fee: BigUint,
        nonce: Nonce,
        fast_withdraw: bool,
        withdraw_fee_ratio: u16,
        signature: Option<TxSignature>,
        ts: TimeStamp,
    ) -> Self {
        let fast_withdraw = if fast_withdraw { 1u8 } else { 0u8 };
        Self {
            to_chain_id,
            account_id,
            sub_account_id,
            to,
            l2_source_token,
            l1_target_token,
            amount,
            fee,
            nonce,
            signature: signature.unwrap_or_default(),
            fast_withdraw,
            withdraw_fee_ratio,
            ts,
        }
    }

    /// Creates a signed transaction using private key and
    /// checks for the transaction correcteness.
    #[allow(clippy::too_many_arguments)]
    pub fn new_signed(
        account_id: AccountId,
        to_chain_id: ChainId,
        sub_account_id: SubAccountId,
        to: ZkLinkAddress,
        l2_token: TokenId,
        l1_token: TokenId,
        amount: BigUint,
        fee: BigUint,
        nonce: Nonce,
        fast_withdraw: bool,
        withdraw_fee_ratio: u16,
        private_key: &PrivateKey<Engine>,
        ts: TimeStamp,
    ) -> Result<Self, anyhow::Error> {
        let mut tx = Self::new(
            account_id,
            sub_account_id,
            to_chain_id,
            to,
            l2_token,
            l1_token,
            amount,
            fee,
            nonce,
            fast_withdraw,
            withdraw_fee_ratio,
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
        out.extend_from_slice(&self.account_id.to_be_bytes());
        out.extend_from_slice(&self.sub_account_id.to_be_bytes());
        out.extend_from_slice(self.to.as_bytes());
        out.extend_from_slice(&(*self.l2_source_token as u16).to_be_bytes());
        out.extend_from_slice(&(*self.l1_target_token as u16).to_be_bytes());
        out.extend_from_slice(&self.amount.to_u128().unwrap().to_be_bytes());
        out.extend_from_slice(&pack_fee_amount(&self.fee));
        out.extend_from_slice(&self.nonce.to_be_bytes());
        out.extend_from_slice(&self.fast_withdraw.to_be_bytes());
        out.extend_from_slice(&self.withdraw_fee_ratio.to_be_bytes());
        out.extend_from_slice(&self.ts.to_be_bytes());
        out
    }

    pub fn check_correctness(&self) -> bool {
        self.validate().is_ok()
    }

    /// Restores the `PubKeyHash` from the transaction signature.
    pub fn verify_signature(&self) -> Option<PubKeyHash> {
        self.signature
            .verify_musig(&self.get_bytes())
            .map(|pub_key| PubKeyHash::from_pubkey(&pub_key))
    }

    /// Get the first part of the message we expect to be signed by Ethereum account key.
    /// The only difference is the missing `nonce` since it's added at the end of the transactions
    /// batch message.
    pub fn get_ethereum_sign_message_part(&self, token_symbol: &str) -> String {
        ethereum_sign_message_part(
            "Withdraw",
            token_symbol,
            TOKEN_MAX_PRECISION as u8,
            &self.amount,
            &self.fee,
            &self.to,
        )
    }

    /// Get message that should be signed by Ethereum keys of the account for 2-Factor authentication.
    pub fn get_ethereum_sign_message(&self, token_symbol: &str) -> String {
        let mut message = self.get_ethereum_sign_message_part(token_symbol);
        if !message.is_empty() {
            message.push('\n');
        }
        message.push_str(format!("Nonce: {}", self.nonce).as_str());
        message
    }
}
