#![allow(unused_doc_comments)]
use crate::helpers::{is_fee_amount_packable, is_token_amount_packable};
use crate::ZkLinkAddress;
use num::BigUint;
use validator::ValidationError;
use zklink_basic_types::{AccountId, ChainId, Nonce, SlotId, SubAccountId, TokenId};
use zklink_crypto::params::{
    GLOBAL_ASSET_ACCOUNT_ID, MAX_ACCOUNT_ID, MAX_CHAIN_ID, MAX_NONCE, MAX_PRICE, MAX_REAL_SLOT_ID,
    MAX_REAL_TOKEN_ID, MAX_SUB_ACCOUNT_ID, MIN_PRICE, TOKEN_ID_ZERO, USDX_TOKEN_ID_LOWER_BOUND,
    USDX_TOKEN_ID_UPPER_BOUND,
};

/// Check transaction account value validation
///
/// - account id should <= MAX_ACCOUNT_ID
/// - account id should not be GLOBAL_ASSET_ACCOUNT_ID(not invalid in transaction)
pub fn account_validator(account_id: &AccountId) -> Result<(), ValidationError> {
    if *account_id > MAX_ACCOUNT_ID {
        return Err(ValidationError::new("account id out of range"));
    }
    if *account_id == GLOBAL_ASSET_ACCOUNT_ID {
        return Err(ValidationError::new("account eq GLOBAL_ASSET_ACCOUNT_ID"));
    }
    Ok(())
}

/// Check transaction sub_account value validation
///
/// - sub_account id should <= MAX_SUB_ACCOUNT_ID
pub fn sub_account_validator(sub_account_id: &SubAccountId) -> Result<(), ValidationError> {
    if *sub_account_id > MAX_SUB_ACCOUNT_ID {
        return Err(ValidationError::new("sub_account id out of range"));
    }
    Ok(())
}

/// Check layer1 unpackable amount value validation
///
/// - amount should <= u128::MAX
pub fn amount_unpackable(amount: &BigUint) -> Result<(), ValidationError> {
    if *amount > BigUint::from(u128::MAX) {
        return Err(ValidationError::new("amount out of range"));
    }
    Ok(())
}

/// Check layer1 packable amount value validation
///
/// - amount should <= 34359738367000000000000000000000000000u128
/// - amount should keep same after pack and unpack
pub fn amount_packable(amount: &BigUint) -> Result<(), ValidationError> {
    if !is_token_amount_packable(amount) {
        return Err(ValidationError::new("amount is not packable"));
    }
    Ok(())
}

/// Check layer1 packable amount value validation
///
/// - fee should <= 20470000000000000000000000000000000u128
/// - fee should keep same after pack and unpack
pub fn fee_packable(fee: &BigUint) -> Result<(), ValidationError> {
    if !is_fee_amount_packable(fee) {
        return Err(ValidationError::new("fee is not packable"));
    }
    Ok(())
}

/// Check token value validation
///
/// - token id should <= MAX_TOKEN_ID
/// - token id should not use 0 and [2,16]
pub fn token_validaotr(token_id: &TokenId) -> Result<(), ValidationError> {
    if *token_id > MAX_REAL_TOKEN_ID {
        return Err(ValidationError::new("token id out of range"));
    }
    if **token_id == TOKEN_ID_ZERO
        || (**token_id >= USDX_TOKEN_ID_LOWER_BOUND && **token_id <= USDX_TOKEN_ID_UPPER_BOUND)
    {
        return Err(ValidationError::new("token id should not use 0 or [2, 16]"));
    }
    Ok(())
}

/// Check zklink address value validation
///
/// - zklink address should not be 0 and GLOBAL_ASSET_ACCOUNT_ADDRESS 0xffffffffffffffffffffffffffffffffffffffff
pub fn zklink_address_validator(zklink_address: &ZkLinkAddress) -> Result<(), ValidationError> {
    if zklink_address.is_zero() {
        return Err(ValidationError::new("zklink address is 0"));
    }
    if zklink_address.is_global_account_address() {
        return Err(ValidationError::new(
            "zklink address is global asset account address",
        ));
    }
    Ok(())
}

/// Check chain id validation
///
/// - chain id should <= MAX_CHAIN_ID
pub fn chain_id_validator(chain_id: &ChainId) -> Result<(), ValidationError> {
    if *chain_id > MAX_CHAIN_ID {
        return Err(ValidationError::new("chain id out of range"));
    }
    Ok(())
}

/// Check boolean flag value validation
///
/// - boolean should be 0 or 1
pub fn boolean_validator(boolean: u8) -> Result<(), ValidationError> {
    if boolean > 1u8 {
        return Err(ValidationError::new("boolean value should be 0 or 1"));
    }
    Ok(())
}

/// Check withdraw fee ratio value validation
///
/// - withdraw_fee_ratio should <= 10000
pub fn withdraw_fee_ratio_validator(withdraw_fee_ratio: u16) -> Result<(), ValidationError> {
    if withdraw_fee_ratio > 10000u16 {
        return Err(ValidationError::new("withdraw fee ratio out of range"));
    }
    Ok(())
}

/// Check order matching price value validation
///
/// - price should > MIN_PRICE(1)
/// - price should < MAX_PRICE(\[(2 ** 15 - 1)/10 ^18\] * 10^18 = 1329227995784915872000000000000000000)
pub fn price_validator(price: &BigUint) -> Result<(), ValidationError> {
    if *price <= BigUint::from(MIN_PRICE) || *price >= BigUint::from(MAX_PRICE) {
        return Err(ValidationError::new("price value out of range"));
    }
    Ok(())
}

/// Check slot id validation
///
/// - slot_id should <= MAX_SLOT_ID
pub fn slot_id_validator(slot_id: &SlotId) -> Result<(), ValidationError> {
    if *slot_id > MAX_REAL_SLOT_ID {
        return Err(ValidationError::new("slot id out of range"));
    }
    Ok(())
}

/// Check nonce validation
///
/// - nonce should < MAX_NONCE
pub fn nonce_validator(nonce: &Nonce) -> Result<(), ValidationError> {
    if *nonce >= MAX_NONCE {
        return Err(ValidationError::new("The nonce has reached its maximum."));
    }
    Ok(())
}

#[cfg(test)]
mod validators_tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_account_validate() {
        #[derive(Debug, Validate)]
        struct Mock {
            #[validate(custom = "account_validator")]
            pub account_id: AccountId,
        }

        impl Mock {
            pub fn new(account_id: AccountId) -> Self {
                Self { account_id }
            }
        }
        /// should success
        let mock = Mock::new(MAX_ACCOUNT_ID);
        assert!(mock.validate().is_ok());
        /// out of range
        let mock = Mock::new(MAX_ACCOUNT_ID + 1);
        assert!(mock.validate().is_err());
        /// invalid
        let mock = Mock::new(GLOBAL_ASSET_ACCOUNT_ID);
        assert!(mock.validate().is_err());
    }

    #[test]
    fn test_sub_account_validate() {
        #[derive(Debug, Validate)]
        struct Mock {
            #[validate(custom = "sub_account_validator")]
            pub sub_account_id: SubAccountId,
        }

        impl Mock {
            pub fn new(sub_account_id: SubAccountId) -> Self {
                Self { sub_account_id }
            }
        }
        /// should success
        let mock = Mock::new(MAX_SUB_ACCOUNT_ID);
        assert!(mock.validate().is_ok());
        /// out of range
        let mock = Mock::new(MAX_SUB_ACCOUNT_ID + 1);
        assert!(mock.validate().is_err());
    }

    #[test]
    fn test_amount_unpackable() {
        #[derive(Debug, Validate)]
        struct Mock {
            #[validate(custom = "amount_unpackable")]
            pub amount: BigUint,
        }

        impl Mock {
            pub fn new(amount: BigUint) -> Self {
                Self { amount }
            }
        }
        /// should success
        let mock = Mock::new(BigUint::from(u128::MAX));
        assert!(mock.validate().is_ok());
        /// out of range
        let mock = Mock::new(BigUint::from(u128::MAX) + BigUint::from(1u128));
        assert!(mock.validate().is_err());
    }

    #[test]
    fn test_amount_packable() {
        #[derive(Debug, Validate)]
        struct Mock {
            #[validate(custom = "amount_packable")]
            pub amount: BigUint,
        }

        impl Mock {
            pub fn new(amount: BigUint) -> Self {
                Self { amount }
            }
        }
        /// should success
        let mock = Mock::new(BigUint::from(34359738367000000000000000000000000000u128));
        assert!(mock.validate().is_ok());
        /// out of range
        let mock = Mock::new(BigUint::from(34359738367000000000000000000000000001u128));
        assert!(mock.validate().is_err());
        /// unpackable
        let mock = Mock::new(BigUint::from(34359738366999999999999999999999999999u128));
        assert!(mock.validate().is_err());
    }

    #[test]
    fn test_fee_packable() {
        #[derive(Debug, Validate)]
        struct Mock {
            #[validate(custom = "fee_packable")]
            pub fee: BigUint,
        }

        impl Mock {
            pub fn new(fee: BigUint) -> Self {
                Self { fee }
            }
        }
        /// should success
        let mock = Mock::new(BigUint::from(20470000000000000000000000000000000u128));
        assert!(mock.validate().is_ok());
        /// out of range
        let mock = Mock::new(BigUint::from(20469999999999999999999999999999999u128));
        assert!(mock.validate().is_err());
    }

    #[test]
    fn test_token_validate() {
        #[derive(Debug, Validate)]
        struct Mock {
            #[validate(custom = "token_validaotr")]
            pub token_id: TokenId,
        }

        impl Mock {
            pub fn new(token_id: TokenId) -> Self {
                Self { token_id }
            }
        }
        /// should success
        let mock = Mock::new(MAX_REAL_TOKEN_ID);
        assert!(mock.validate().is_ok());
        /// out of range
        let mock = Mock::new(MAX_REAL_TOKEN_ID + 1);
        assert!(mock.validate().is_err());
        /// invalid
        let mock = Mock::new(TokenId(TOKEN_ID_ZERO));
        assert!(mock.validate().is_err());
        let mock = Mock::new(TokenId(USDX_TOKEN_ID_LOWER_BOUND));
        assert!(mock.validate().is_err());
        let mock = Mock::new(TokenId(USDX_TOKEN_ID_UPPER_BOUND));
        assert!(mock.validate().is_err());
    }

    #[test]
    fn test_zklink_address_validate() {
        #[derive(Debug, Validate)]
        struct Mock {
            #[validate(custom = "zklink_address_validator")]
            pub zklink_address: ZkLinkAddress,
        }

        impl Mock {
            pub fn new(zklink_address: ZkLinkAddress) -> Self {
                Self { zklink_address }
            }
        }
        /// should success
        let v1: Vec<u8> = vec![1; 32];
        let v2: Vec<u8> = vec![0; 32];
        let v3: Vec<u8> = vec![0xff; 32];
        let mock = Mock::new(ZkLinkAddress::from(v1));
        assert!(mock.validate().is_ok());
        /// out of range
        let mock = Mock::new(ZkLinkAddress::from(v2));
        assert!(mock.validate().is_err());
        /// out of range
        let mock = Mock::new(ZkLinkAddress::from(v3));
        assert!(mock.validate().is_err());
    }

    #[test]
    fn test_boolean_validate() {
        #[derive(Debug, Validate)]
        struct Mock {
            #[validate(custom = "boolean_validator")]
            pub boolean: u8,
        }

        impl Mock {
            pub fn new(boolean: u8) -> Self {
                Self { boolean }
            }
        }
        /// should success
        let mock = Mock::new(0);
        assert!(mock.validate().is_ok());
        let mock = Mock::new(1);
        assert!(mock.validate().is_ok());
        /// out of range
        let mock = Mock::new(2);
        assert!(mock.validate().is_err());
    }

    #[test]
    fn test_chain_id_validate() {
        #[derive(Debug, Validate)]
        struct Mock {
            #[validate(custom = "chain_id_validator")]
            pub chain_id: ChainId,
        }

        impl Mock {
            pub fn new(chain_id: ChainId) -> Self {
                Self { chain_id }
            }
        }
        /// should success
        let mock = Mock::new(MAX_CHAIN_ID);
        assert!(mock.validate().is_ok());
        /// out of range
        let mock = Mock::new(MAX_CHAIN_ID + 1);
        assert!(mock.validate().is_err());
    }

    #[test]
    fn test_withdraw_fee_ratio_validate() {
        #[derive(Debug, Validate)]
        struct Mock {
            #[validate(custom = "withdraw_fee_ratio_validator")]
            pub withdraw_fee_ratio: u16,
        }

        impl Mock {
            pub fn new(withdraw_fee_ratio: u16) -> Self {
                Self { withdraw_fee_ratio }
            }
        }
        /// should success
        let mock = Mock::new(10000u16);
        assert!(mock.validate().is_ok());
        /// out of range
        let mock = Mock::new(10001u16);
        assert!(mock.validate().is_err());
    }

    #[test]
    fn test_price_validate() {
        #[derive(Debug, Validate)]
        struct Mock {
            #[validate(custom = "price_validator")]
            pub price: BigUint,
        }

        impl Mock {
            pub fn new(price: BigUint) -> Self {
                Self { price }
            }
        }
        /// should success
        let mock = Mock::new(BigUint::from(MIN_PRICE + 1));
        assert!(mock.validate().is_ok());
        let mock = Mock::new(BigUint::from(MAX_PRICE - 1));
        assert!(mock.validate().is_ok());
        /// out of range
        let mock = Mock::new(BigUint::from(MIN_PRICE));
        assert!(mock.validate().is_err());
        let mock = Mock::new(BigUint::from(MAX_PRICE));
        assert!(mock.validate().is_err());
    }

    #[test]
    fn test_slot_id_validate() {
        #[derive(Debug, Validate)]
        struct Mock {
            #[validate(custom = "slot_id_validator")]
            pub slot_id: SlotId,
        }

        impl Mock {
            pub fn new(slot_id: SlotId) -> Self {
                Self { slot_id }
            }
        }
        /// should success
        let mock = Mock::new(MAX_REAL_SLOT_ID);
        assert!(mock.validate().is_ok());
        /// out of range
        let mock = Mock::new(MAX_REAL_SLOT_ID + 1);
        assert!(mock.validate().is_err());
    }

    #[test]
    fn test_nonce_validate() {
        #[derive(Debug, Validate)]
        struct Mock {
            #[validate(custom = "nonce_validator")]
            pub nonce: Nonce,
        }

        impl Mock {
            pub fn new(nonce: Nonce) -> Self {
                Self { nonce }
            }
        }
        /// should success
        let mock = Mock::new(MAX_NONCE - 1);
        assert!(mock.validate().is_ok());
        /// out of range
        let mock = Mock::new(MAX_NONCE);
        assert!(mock.validate().is_err());
    }
}
