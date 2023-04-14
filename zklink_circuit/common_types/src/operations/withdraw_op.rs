use crate::operations::GetPublicData;
use crate::utils::check_source_token_and_target_token;
use crate::{
    helpers::{pack_fee_amount, unpack_fee_amount},
    Withdraw, ZkLinkAddress,
};
use crate::{AccountId, Nonce, TokenId};
use anyhow::{ensure, format_err};
use num::{BigUint, ToPrimitive};
use serde::{Deserialize, Serialize};
use zklink_basic_types::{ChainId, SubAccountId};
use zklink_crypto::params::{
    ACCOUNT_ID_BIT_WIDTH, BALANCE_BIT_WIDTH, CHAIN_ID_BIT_WIDTH, CHUNK_BYTES,
    ETH_ADDRESS_BIT_WIDTH, FEE_BIT_WIDTH, GLOBAL_ASSET_ACCOUNT_ID, NONCE_BIT_WIDTH,
    SUB_ACCOUNT_ID_BIT_WIDTH, TOKEN_BIT_WIDTH,
};
use zklink_crypto::primitives::FromBytes;

/// Withdraw operation. For details, see the documentation of [`ZkLinkOp`](./operations/enum.ZkLinkOp.html).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawOp {
    pub tx: Withdraw,
    pub account_id: AccountId,
    pub l1_target_token_after_mapping: TokenId,
}

impl GetPublicData for WithdrawOp {
    fn get_public_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.push(Self::OP_CODE); // opcode
        data.extend_from_slice(&self.tx.to_chain_id.to_be_bytes()); // 1
        data.extend_from_slice(&self.account_id.to_be_bytes()); // 4
        data.extend_from_slice(&self.tx.sub_account_id.to_be_bytes()); // 1
        data.extend_from_slice(&(*self.tx.l1_target_token as u16).to_be_bytes()); // 2
        data.extend_from_slice(&(*self.tx.l2_source_token as u16).to_be_bytes()); // 2
        data.extend_from_slice(&self.tx.amount.to_u128().unwrap().to_be_bytes()); // 16
        data.extend_from_slice(&pack_fee_amount(&self.tx.fee)); // 2
        data.extend_from_slice(self.tx.to.as_bytes()); // 32
        data.extend_from_slice(
            if self.tx.fast_withdraw == 1 {
                self.tx.nonce
            } else {
                Nonce(0)
            }
            .to_be_bytes()
            .as_ref(),
        ); // 4
        data.extend_from_slice(&self.tx.withdraw_fee_ratio.to_be_bytes()); // 2
        data.resize(Self::CHUNKS * CHUNK_BYTES, 0x00);
        data
    }
}

impl WithdrawOp {
    pub const WITHDRAW_DATA_PREFIX: [u8; 1] = [1];

    pub(crate) fn get_withdrawal_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&Self::WITHDRAW_DATA_PREFIX); // first byte is a bool variable 'addToPendingWithdrawalsQueue'
        data.extend_from_slice(&self.tx.to_chain_id.to_be_bytes());
        data.extend_from_slice(self.tx.to.as_bytes());
        data.extend_from_slice(&self.tx.sub_account_id.to_be_bytes());
        data.extend_from_slice(&(*self.tx.l2_source_token as u16).to_be_bytes());
        data.extend_from_slice(&self.tx.amount.to_u128().unwrap().to_be_bytes());
        data
    }

    pub fn from_public_data(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        ensure!(
            bytes.len() == Self::CHUNKS * CHUNK_BYTES,
            "Wrong bytes length for withdraw pubdata"
        );

        let chain_id_offset = 1;
        let account_offset = chain_id_offset + CHAIN_ID_BIT_WIDTH / 8;
        let sub_account_offset = account_offset + ACCOUNT_ID_BIT_WIDTH / 8;
        let l1_target_token_offset = sub_account_offset + SUB_ACCOUNT_ID_BIT_WIDTH / 8;
        let l2_source_token_offset = l1_target_token_offset + TOKEN_BIT_WIDTH / 8;
        let amount_offset = l2_source_token_offset + TOKEN_BIT_WIDTH / 8;
        let fee_offset = amount_offset + BALANCE_BIT_WIDTH / 8;
        let to_address_offset = fee_offset + FEE_BIT_WIDTH / 8;
        let nonce_offset = to_address_offset + ETH_ADDRESS_BIT_WIDTH / 8;
        let withdraw_fee_ratio_offset = nonce_offset + NONCE_BIT_WIDTH / 8;
        let end = withdraw_fee_ratio_offset + 2;

        let chain_id = bytes[chain_id_offset];
        let account_id = u32::from_bytes(&bytes[account_offset..sub_account_offset])
            .ok_or_else(|| format_err!("Cant get account id from withdraw pubdata"))?;
        let sub_account_id = bytes[sub_account_offset];
        let l1_target_token =
            u16::from_bytes(&bytes[l1_target_token_offset..l2_source_token_offset])
                .ok_or_else(|| format_err!("Cant get token id from withdraw pubdata"))?;
        let l2_source_token = u16::from_bytes(&bytes[l2_source_token_offset..amount_offset])
            .ok_or_else(|| format_err!("Cant get real token id from withdraw pubdata"))?;
        let amount = BigUint::from(
            u128::from_bytes(&bytes[amount_offset..fee_offset])
                .ok_or_else(|| format_err!("Cant get amount from withdraw pubdata"))?,
        );
        let fee = unpack_fee_amount(&bytes[fee_offset..to_address_offset])
            .ok_or_else(|| format_err!("Cant get fee from withdraw pubdata"))?;
        let to = ZkLinkAddress::from_slice(&bytes[to_address_offset..nonce_offset])?;
        let nonce = u32::from_bytes(&bytes[nonce_offset..withdraw_fee_ratio_offset])
            .ok_or_else(|| format_err!("Cant get nonce id from withdraw pubdata"))?;
        let withdraw_fee_ratio = u16::from_bytes(&bytes[withdraw_fee_ratio_offset..end])
            .ok_or_else(|| format_err!("Cant get withdraw_amount_in from withdraw pubdata"))?;

        // Check whether the mapping between l1_token and l2_token is correct
        let (is_required, l1_target_token_after_mapping) =
            check_source_token_and_target_token(l2_source_token.into(), l1_target_token.into());
        ensure!(
            is_required,
            "Source token or target token is mismatching in Withdraw Pubdata"
        );

        Ok(Self {
            tx: Withdraw::new(
                AccountId(account_id),
                SubAccountId(sub_account_id),
                ChainId(chain_id),
                to,
                TokenId(l2_source_token as u32),
                TokenId(l1_target_token as u32),
                amount,
                fee,
                Nonce(nonce),
                nonce > 0,
                withdraw_fee_ratio,
                None,
                Default::default(),
            ),
            account_id: AccountId(account_id),
            l1_target_token_after_mapping,
        })
    }

    pub fn get_updated_account_ids(&self) -> Vec<AccountId> {
        vec![self.account_id, GLOBAL_ASSET_ACCOUNT_ID]
    }
}
