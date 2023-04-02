use crate::{Deposit, ChainId, ZkLinkAddress};
use crate::{AccountId, TokenId, SubAccountId};
use anyhow::{ensure, format_err};
use num::{BigUint, ToPrimitive};
use serde::{Deserialize, Serialize};
use zklink_crypto::params::{ACCOUNT_ID_BIT_WIDTH, BALANCE_BIT_WIDTH, CHUNK_BYTES, TOKEN_BIT_WIDTH, GLOBAL_ASSET_ACCOUNT_ID, CHAIN_ID_BIT_WIDTH, SUB_ACCOUNT_ID_BIT_WIDTH, ETH_ADDRESS_BIT_WIDTH};
use zklink_crypto::primitives::FromBytes;
use crate::operations::GetPublicData;
use crate::utils::check_source_token_and_target_token;

/// Deposit operation. For details, see the documentation of [`ZkLinkOp`](./operations/enum.ZkLinkOp.html).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositOp {
    pub tx: Deposit,
    pub account_id: AccountId,
    pub l1_source_token_after_mapping: TokenId
}

impl GetPublicData for DepositOp{
    fn get_public_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.push(Self::OP_CODE); // opcode
        data.extend_from_slice(&self.tx.from_chain_id.to_be_bytes());
        data.extend_from_slice(&self.account_id.to_be_bytes());
        data.extend_from_slice(&self.tx.sub_account_id.to_be_bytes());
        data.extend_from_slice(&(*self.tx.l1_source_token as u16).to_be_bytes());
        data.extend_from_slice(&(*self.tx.l2_target_token as u16).to_be_bytes());
        data.extend_from_slice(&self.tx.amount.to_u128().unwrap().to_be_bytes());
        data.extend_from_slice(&self.tx.to.as_bytes());
        data.resize(Self::CHUNKS * CHUNK_BYTES, 0x00);
        data
    }
}

impl DepositOp {
    pub fn from_public_data(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        ensure!(
            bytes.len() == Self::CHUNKS * CHUNK_BYTES,
            "Wrong bytes length for deposit pubdata"
        );

        let chain_id_offset = 1;
        let account_id_offset = chain_id_offset + CHAIN_ID_BIT_WIDTH / 8;
        let sub_account_id_offset = account_id_offset + ACCOUNT_ID_BIT_WIDTH / 8;
        let l1_source_token_offset = sub_account_id_offset + SUB_ACCOUNT_ID_BIT_WIDTH / 8;
        let l2_target_token_offset = l1_source_token_offset + TOKEN_BIT_WIDTH / 8;
        let amount_offset = l2_target_token_offset + TOKEN_BIT_WIDTH / 8;
        let to_address_offset = amount_offset + BALANCE_BIT_WIDTH / 8;
        let end = to_address_offset + ETH_ADDRESS_BIT_WIDTH / 8;

        let from_chain_id = bytes[chain_id_offset];
        let account_id = u32::from_bytes(
            &bytes[account_id_offset..sub_account_id_offset],
        ).ok_or_else(|| format_err!("Cant get account id from deposit pubdata"))?;
        let sub_account_id = bytes[sub_account_id_offset];
        let l1_source_token = u16::from_bytes(&bytes[l1_source_token_offset..l2_target_token_offset])
            .ok_or_else(|| format_err!("Cant get token id from deposit pubdata"))?;
        let l2_target_token = u16::from_bytes(&bytes[l2_target_token_offset..amount_offset])
            .ok_or_else(|| format_err!("Cant get real token id from deposit pubdata"))?;
        let amount = BigUint::from(
            u128::from_bytes(&bytes[amount_offset..to_address_offset])
                .ok_or_else(|| format_err!("Cant get amount from deposit pubdata"))?,
        );
        let to = ZkLinkAddress::from_slice(&bytes[to_address_offset..end])?;

        // Check whether the mapping between l1_token and l2_token is correct
        let (is_required, l1_source_token_after_mapping) =
            check_source_token_and_target_token(l2_target_token.into(), l1_source_token.into());
        ensure!(is_required, "Source token or target token is mismatching in Deposit Pubdata");

        Ok(Self {
            tx: Deposit {
                from_chain_id: ChainId(from_chain_id),
                from: Default::default(), // unknown from pubdata.
                sub_account_id: SubAccountId(sub_account_id),
                l2_target_token: TokenId(l2_target_token as u32),
                l1_source_token: TokenId(l1_source_token as u32),
                amount,
                to,
                serial_id: Default::default(),
                eth_hash: Default::default(),
            },
            account_id: AccountId(account_id),
            l1_source_token_after_mapping
        })
    }

    pub fn get_updated_account_ids(&self) -> Vec<AccountId> {
        vec![self.account_id, GLOBAL_ASSET_ACCOUNT_ID]
    }
}
