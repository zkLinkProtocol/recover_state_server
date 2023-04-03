use crate::{FullExit, AccountId, TokenId, ZkLinkAddress};
use anyhow::{ensure, format_err};
use num::{BigUint, ToPrimitive};
use serde::{Deserialize, Serialize};
use zklink_basic_types::{ChainId, SubAccountId};
use zklink_crypto::params::{ACCOUNT_ID_BIT_WIDTH, BALANCE_BIT_WIDTH, CHAIN_ID_BIT_WIDTH, CHUNK_BYTES, ETH_ADDRESS_BIT_WIDTH, GLOBAL_ASSET_ACCOUNT_ID, SUB_ACCOUNT_ID_BIT_WIDTH, TOKEN_BIT_WIDTH};
use zklink_crypto::primitives::FromBytes;
use zklink_utils::BigUintSerdeAsRadix10Str;
use crate::operations::GetPublicData;
use crate::utils::check_source_token_and_target_token;

/// FullExit operation. For details, see the documentation of [`ZkLinkOp`](./operations/enum.ZkLinkOp.html).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullExitOp {
    pub tx: FullExit,
    /// None if withdraw was unsuccessful
    #[serde(with = "BigUintSerdeAsRadix10Str")]
    pub exit_amount: BigUint,
    pub l1_target_token_after_mapping: TokenId
}

impl GetPublicData for FullExitOp{
    fn get_public_data(&self) -> Vec<u8> {
        let withdraw_amount = self.exit_amount.clone();
        let mut data = Vec::new();
        data.push(Self::OP_CODE); // opcode
        data.extend_from_slice(&self.tx.to_chain_id.to_be_bytes());
        data.extend_from_slice(&self.tx.account_id.to_be_bytes());
        data.extend_from_slice(&self.tx.sub_account_id.to_be_bytes());
        data.extend_from_slice(self.tx.exit_address.as_bytes());
        data.extend_from_slice(&(*self.tx.l1_target_token as u16).to_be_bytes());
        data.extend_from_slice(&(*self.tx.l2_source_token as u16).to_be_bytes());
        data.extend_from_slice(
            &withdraw_amount
                .to_u128()
                .unwrap()
                .to_be_bytes()
        );
        data.resize(Self::CHUNKS * CHUNK_BYTES, 0x00);
        data
    }
}

impl FullExitOp {
    pub const WITHDRAW_DATA_PREFIX: [u8; 1] = [0];

    pub(crate) fn get_withdrawal_data(&self) -> Vec<u8> {
        let withdraw_amount = self.exit_amount.clone();
        let mut data = Vec::new();
        data.extend_from_slice(&Self::WITHDRAW_DATA_PREFIX); // first byte is a bool variable 'addToPendingWithdrawalsQueue'
        data.extend_from_slice(&self.tx.to_chain_id.to_be_bytes());
        data.extend_from_slice(self.tx.exit_address.as_bytes());
        data.extend_from_slice(&(*self.tx.l2_source_token as u16).to_be_bytes());
        data.extend_from_slice(
            &withdraw_amount
                .to_u128()
                .unwrap()
                .to_be_bytes()
        );
        data
    }

    pub fn from_public_data(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        ensure!(
            bytes.len() == Self::CHUNKS * CHUNK_BYTES,
            "Wrong bytes length for full exit pubdata"
        );

        let chain_id_offset = 1;
        let account_id_offset = chain_id_offset + CHAIN_ID_BIT_WIDTH / 8;
        let sub_account_id_offset = account_id_offset + ACCOUNT_ID_BIT_WIDTH / 8;
        let exit_address_offset = sub_account_id_offset + SUB_ACCOUNT_ID_BIT_WIDTH / 8;
        let l1_target_token_offset = exit_address_offset + ETH_ADDRESS_BIT_WIDTH / 8;
        let l2_source_token_offset = l1_target_token_offset + TOKEN_BIT_WIDTH / 8;
        let amount_offset = l2_source_token_offset + TOKEN_BIT_WIDTH / 8;
        let end = amount_offset + BALANCE_BIT_WIDTH / 8;

        let chain_id = bytes[chain_id_offset];
        let account_id = u32::from_bytes(&bytes[account_id_offset..sub_account_id_offset])
            .ok_or_else(|| format_err!("Cant get account id from full exit pubdata"))?;
        let sub_account_id = bytes[sub_account_id_offset];
        let exit_address = ZkLinkAddress::from_slice(&bytes[exit_address_offset..l1_target_token_offset])?;
        let l1_target_token = u16::from_bytes(&bytes[l1_target_token_offset..l2_source_token_offset])
            .ok_or_else(|| format_err!("Cant get token id from full exit pubdata"))?;
        let l2_source_token = u16::from_bytes(&bytes[l2_source_token_offset..amount_offset])
            .ok_or_else(|| format_err!("Cant get real token id from full exit pubdata"))?;
        let amount = BigUint::from(
            u128::from_bytes(&bytes[amount_offset..end])
                .ok_or_else(|| format_err!("Cant get amount from full exit pubdata"))?,
        );

        // Check whether the mapping between l1_token and l2_token is correct
        let (is_required, l1_target_token_after_mapping) =
            check_source_token_and_target_token(l2_source_token.into(), l1_target_token.into());
        ensure!(is_required, "Source token or target token is mismatching in FullExit pubdata");

        Ok(Self {
            tx: FullExit {
                to_chain_id: ChainId(chain_id),
                account_id: AccountId(account_id),
                sub_account_id: SubAccountId(sub_account_id),
                exit_address,
                l2_source_token: TokenId(l2_source_token as u32),
                l1_target_token: TokenId(l1_target_token as u32),
                serial_id: Default::default(),
                eth_hash: Default::default(),
            },
            exit_amount: amount,
            l1_target_token_after_mapping
        })
    }

    pub fn get_updated_account_ids(&self) -> Vec<AccountId> {
        vec![self.tx.account_id, GLOBAL_ASSET_ACCOUNT_ID]
    }
}
