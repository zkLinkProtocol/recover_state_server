use crate::{helpers::{pack_fee_amount, unpack_fee_amount}, ForcedExit, ZkLinkAddress};
use crate::{AccountId, Nonce, TokenId, ChainId, SubAccountId};
use anyhow::{ensure, format_err};
use num::{BigUint, ToPrimitive};
use serde::{Deserialize, Serialize};
use zklink_crypto::params::{ACCOUNT_ID_BIT_WIDTH, BALANCE_BIT_WIDTH, CHUNK_BYTES, TOKEN_BIT_WIDTH, CHAIN_ID_BIT_WIDTH, GLOBAL_ASSET_ACCOUNT_ID, SUB_ACCOUNT_ID_BIT_WIDTH, FEE_BIT_WIDTH, ETH_ADDRESS_BIT_WIDTH};
use zklink_crypto::primitives::FromBytes;
use zklink_utils::BigUintSerdeAsRadix10Str;
use crate::operations::GetPublicData;
use crate::utils::check_source_token_and_target_token;


/// ForcedExit operation. For details, see the documentation of [`ZkLinkOp`](./operations/enum.ZkLinkOp.html).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForcedExitOp {
    pub tx: ForcedExit,
    /// Account ID of the account to which ForcedExit is applied.
    pub target_account_id: AccountId,
    /// None if withdraw was unsuccessful
    #[serde(with = "BigUintSerdeAsRadix10Str")]
    pub withdraw_amount: BigUint,
    pub l1_target_token_after_mapping: TokenId
}

impl GetPublicData for ForcedExitOp{
    fn get_public_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.push(Self::OP_CODE); // opcode
        data.extend_from_slice(&self.tx.to_chain_id.to_be_bytes());
        data.extend_from_slice(&self.tx.initiator_account_id.to_be_bytes());
        data.extend_from_slice(&self.tx.initiator_sub_account_id.to_be_bytes());
        data.extend_from_slice(&self.target_account_id.to_be_bytes());
        data.extend_from_slice(&self.tx.target_sub_account_id.to_be_bytes());
        data.extend_from_slice(&(*self.tx.l1_target_token as u16).to_be_bytes());
        data.extend_from_slice(&(*self.tx.l2_source_token as u16).to_be_bytes());
        data.extend_from_slice(&(*self.tx.fee_token as u16).to_be_bytes());
        data.extend_from_slice(&self.withdraw_amount.to_u128().unwrap().to_be_bytes());
        data.extend_from_slice(&pack_fee_amount(&self.tx.fee));
        data.extend_from_slice(self.tx.target.as_bytes());
        data.resize(Self::CHUNKS * CHUNK_BYTES, 0x00);
        data
    }
}

impl ForcedExitOp {
    pub const WITHDRAW_DATA_PREFIX: [u8; 1] = [1];

    pub fn get_withdrawal_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&Self::WITHDRAW_DATA_PREFIX); // first byte is a bool variable 'addToPendingWithdrawalsQueue'
        data.extend_from_slice(&self.tx.to_chain_id.to_be_bytes());
        data.extend_from_slice(self.tx.target.as_bytes());
        data.extend_from_slice(&self.tx.target_sub_account_id.to_be_bytes());
        data.extend_from_slice(&(*self.tx.l2_source_token as u16).to_be_bytes());
        data.extend_from_slice(&self.withdraw_amount.to_u128().unwrap().to_be_bytes());
        data
    }

    pub fn from_public_data(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        ensure!(
            bytes.len() == Self::CHUNKS * CHUNK_BYTES,
            "Wrong bytes length for forced exit pubdata"
        );

        let chain_id_offset = 1;
        let initiator_account_id_offset = chain_id_offset + CHAIN_ID_BIT_WIDTH / 8;
        let initiator_sub_account_id_offset = initiator_account_id_offset + ACCOUNT_ID_BIT_WIDTH / 8;
        let target_account_id_offset = initiator_sub_account_id_offset + SUB_ACCOUNT_ID_BIT_WIDTH / 8;
        let target_sub_account_id_offset = target_account_id_offset + ACCOUNT_ID_BIT_WIDTH / 8;
        let l1_target_token_offset = target_sub_account_id_offset + SUB_ACCOUNT_ID_BIT_WIDTH / 8;
        let l2_source_token_offset = l1_target_token_offset + TOKEN_BIT_WIDTH / 8;
        let fee_token_id_offset = l2_source_token_offset + TOKEN_BIT_WIDTH / 8;
        let amount_offset = fee_token_id_offset + TOKEN_BIT_WIDTH / 8;
        let fee_offset = amount_offset + BALANCE_BIT_WIDTH / 8;
        let target_offset = fee_offset + FEE_BIT_WIDTH / 8;
        let end = target_offset + ETH_ADDRESS_BIT_WIDTH / 8;

        let to_chain_id = bytes[chain_id_offset];
        let initiator_account_id =
            u32::from_bytes(&bytes[initiator_account_id_offset..initiator_sub_account_id_offset])
                .ok_or_else(|| {
                    format_err!("Cant get initiator account id from forced exit pubdata")
                })?;
        let initiator_sub_account_id = bytes[initiator_sub_account_id_offset];
        let target_account_id = u32::from_bytes(&bytes[target_account_id_offset..target_sub_account_id_offset])
            .ok_or_else(|| format_err!("Cant get target account id from forced exit pubdata"))?;
        let target_sub_account_id = bytes[target_sub_account_id_offset];
        let l1_target_token = u16::from_bytes(&bytes[l1_target_token_offset..l2_source_token_offset])
            .ok_or_else(|| format_err!("Cant get l1_target_token from forced exit pubdata"))?;
        let l2_source_token = u16::from_bytes(&bytes[l2_source_token_offset..fee_token_id_offset])
            .ok_or_else(|| format_err!("Cant get l2_source_token from forced exit pubdata"))?;
        let fee_token = u16::from_bytes(&bytes[fee_token_id_offset..amount_offset])
            .ok_or_else(|| format_err!("Cant get fee token id from forced exit pubdata"))?;
        let amount = BigUint::from(
            u128::from_bytes(&bytes[amount_offset..fee_offset])
                .ok_or_else(|| format_err!("Cant get amount from forced exit pubdata"))?
        );
        let fee = unpack_fee_amount(&bytes[fee_offset..target_offset])
            .ok_or_else(|| format_err!("Cant get fee from forced pubdata"))?;
        let target = ZkLinkAddress::from_slice(&bytes[target_offset..end])?;
        let nonce = 0; // From pubdata it is unknown

        // Check whether the mapping between l1_token and l2_token is correct
        let (is_required, l1_target_token_after_mapping) =
            check_source_token_and_target_token(l2_source_token.into(), l1_target_token.into());
        ensure!(is_required, "Source token or target token is mismatching in ForcedExit pubdata");

        Ok(Self {
            tx: ForcedExit::new(
                ChainId(to_chain_id),
                AccountId(initiator_account_id),
                SubAccountId(initiator_sub_account_id),
                target,
                SubAccountId(target_sub_account_id),
                TokenId(l2_source_token as u32),
                TokenId(l1_target_token as u32),
                TokenId(fee_token as u32),
                fee,
                Nonce(nonce),
                None,
                Default::default(),
            ),
            target_account_id: AccountId(target_account_id),
            withdraw_amount: amount,
            l1_target_token_after_mapping
        })
    }

    pub fn get_updated_account_ids(&self) -> Vec<AccountId> {
        vec![self.target_account_id, self.tx.initiator_account_id, GLOBAL_ASSET_ACCOUNT_ID]
    }
}
