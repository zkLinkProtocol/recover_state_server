//! Utilities used in tx module.

// External uses.
use std::convert::TryInto;
use num::{BigUint, Zero};
use serde::{
    de::{value::SeqAccessDeserializer, Error, SeqAccess, Visitor},
    Deserialize, Deserializer,
};

// Workspace uses.
use zklink_utils::format_units;

// Local uses.
use crate::ZkLinkAddress;

/// Deserializes either a `String` or `Vec<u8>` into `Vec<u8>`.
/// The reason we cannot expect just a vector is backward compatibility: messages
/// used to be stored as strings.
pub fn deserialize_eth_message<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrVec;

    impl<'de> Visitor<'de> for StringOrVec {
        type Value = Vec<u8>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a byte array or a string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(v.as_bytes().to_vec())
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            Deserialize::deserialize(SeqAccessDeserializer::new(seq))
        }
    }

    deserializer.deserialize_any(StringOrVec)
}

/// Serialize `H256` as `Vec<u8>`.
///
/// This workaround used for backward compatibility
/// with the old serialize/deserialize behaviour of the fields
/// whose type changed from `Vec<u8>` to `H256`.
pub mod h256_as_vec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::iter;
    use zklink_basic_types::H256;

    pub fn serialize<S>(val: &H256, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let val = val.as_bytes().to_vec();
        val.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<H256, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expected_size = H256::len_bytes();

        let mut val = Vec::deserialize(deserializer)?;
        if let Some(padding_size) = expected_size.checked_sub(val.len()) {
            if padding_size > 0 {
                val = iter::repeat(0).take(padding_size).chain(val).collect();
            }
        }

        Ok(H256::from_slice(&val))
    }
}


use std::convert::AsMut;
use zklink_basic_types::{SlotId, SubAccountId, TokenId, ChainId};
use zklink_crypto::params::{
    MAX_ORDER_NUMBER, MAX_TOKEN_NUMBER, USDX_TOKEN_ID_RANGE,USD_TOKEN_ID, USDX_TOKEN_ID_LOWER_BOUND, USDX_TOKEN_ID_UPPER_BOUND
};

pub fn clone_into_array<A, T>(slice: &[T]) -> A
    where
        A: Default + AsMut<[T]>,
        T: Clone,
{
    let mut a = A::default();
    <A as AsMut<[T]>>::as_mut(&mut a).clone_from_slice(slice);
    a
}

/// Construct the first part of the message that should be signed by Ethereum key.
/// The pattern is as follows:
///
/// [{Transfer/Withdraw} {amount} {token} to: {to_address}]
/// [Fee: {fee} {token}]
///
/// Note that both lines are optional.
pub fn ethereum_sign_message_part(
    transaction: &str,
    token_symbol: &str,
    decimals: u8,
    amount: &BigUint,
    fee: &BigUint,
    to: &ZkLinkAddress,
) -> String {
    let mut message = if !amount.is_zero() {
        format!(
            "{transaction} {amount} {token} to: {to}",
            transaction = transaction,
            amount = format_units(amount, decimals),
            token = token_symbol,
            to = to.to_string()
        )
    } else {
        String::new()
    };
    if !fee.is_zero() {
        if !message.is_empty() {
            message.push('\n');
        }
        message.push_str(
            format!(
                "Fee: {fee} {token}",
                fee = format_units(fee, decimals),
                token = token_symbol
            )
            .as_str(),
        );
    }
    message
}

/// Check l1 token(deposited from layer one or withdraw to layer one) and l2 token(token exist in layer two)
/// Returns the mapping token id in layer two of `l1_token`
pub fn check_source_token_and_target_token(l2_token: TokenId, l1_token: TokenId) -> (bool, TokenId) {
    let mut real_l1_token = l1_token;
    let is_required_tokens = if *l2_token == USD_TOKEN_ID {
        *real_l1_token = *l1_token - USDX_TOKEN_ID_RANGE;
        USDX_TOKEN_ID_LOWER_BOUND <= *real_l1_token && *real_l1_token <= USDX_TOKEN_ID_UPPER_BOUND
    } else if USDX_TOKEN_ID_LOWER_BOUND <= *l2_token && *l2_token <= USDX_TOKEN_ID_UPPER_BOUND {
        false
    } else {
        l2_token == l1_token
    };
    (is_required_tokens, real_l1_token)
}

pub fn calculate_actual_slot(sub_account_id: SubAccountId, slot_id: SlotId) -> SlotId{
    SlotId(*slot_id + *sub_account_id as u32 * MAX_ORDER_NUMBER as u32)
}

pub fn calculate_actual_token(sub_account_id: SubAccountId, token_id: TokenId) -> TokenId{
    TokenId(*token_id + *sub_account_id as u32 * MAX_TOKEN_NUMBER as u32)
}

pub fn calculate_actual_token_by_chain(chain_id: ChainId, token_id: TokenId) -> TokenId{
    let token_id = *token_id - USDX_TOKEN_ID_RANGE;
    TokenId(token_id + *chain_id as u32 * MAX_TOKEN_NUMBER as u32)
}

pub fn recover_raw_slot(slot_id: SlotId) -> SlotId{
    SlotId(*slot_id % MAX_ORDER_NUMBER as u32)
}

pub fn recover_sub_account_by_slot(slot_id: SlotId) -> SubAccountId{
    SubAccountId((*slot_id / MAX_ORDER_NUMBER as u32).try_into().unwrap())
}

pub fn recover_raw_token(token_id: TokenId) -> TokenId{
    TokenId(*token_id % MAX_TOKEN_NUMBER as u32)
}

pub fn recover_sub_account_by_token(token_id: TokenId) -> SubAccountId{
    SubAccountId((*token_id / MAX_TOKEN_NUMBER as u32) as u8)
}

// The following is used for global asset account.
pub fn recover_chain(token_id: TokenId) -> ChainId{
    ChainId((*token_id / MAX_TOKEN_NUMBER as u32) as u8)
}