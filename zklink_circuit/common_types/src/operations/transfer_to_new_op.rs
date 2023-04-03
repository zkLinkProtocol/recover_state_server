use crate::{helpers::{pack_fee_amount, pack_token_amount, unpack_fee_amount, unpack_token_amount}, Transfer, ZkLinkAddress};
use crate::{AccountId, Nonce, TokenId};
use anyhow::{ensure, format_err};
use serde::{Deserialize, Serialize};
use zklink_basic_types::SubAccountId;
use zklink_crypto::params::{ACCOUNT_ID_BIT_WIDTH, AMOUNT_BIT_WIDTH, CHUNK_BYTES, ETH_ADDRESS_BIT_WIDTH, FEE_BIT_WIDTH, SUB_ACCOUNT_ID_BIT_WIDTH, TOKEN_BIT_WIDTH};
use zklink_crypto::primitives::FromBytes;
use crate::operations::GetPublicData;

/// TransferToNew operation. For details, see the documentation of [`ZkLinkOp`](./operations/enum.ZkLinkOp.html).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferToNewOp {
    pub tx: Transfer,
    pub from: AccountId,
    pub to: AccountId,
}

impl GetPublicData for TransferToNewOp{
    fn get_public_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.push(Self::OP_CODE); // opcode
        data.extend_from_slice(&self.from.to_be_bytes());
        data.extend_from_slice(&self.tx.from_sub_account_id.to_be_bytes());
        data.extend_from_slice(&(*self.tx.token as u16).to_be_bytes());
        data.extend_from_slice(&pack_token_amount(&self.tx.amount));
        data.extend_from_slice(&self.tx.to.as_bytes());
        data.extend_from_slice(&self.to.to_be_bytes());
        data.extend_from_slice(&self.tx.to_sub_account_id.to_be_bytes());
        data.extend_from_slice(&pack_fee_amount(&self.tx.fee));

        data.resize(Self::CHUNKS * CHUNK_BYTES, 0x00);
        data
    }
}

impl TransferToNewOp {
    pub fn from_public_data(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        ensure!(
            bytes.len() == Self::CHUNKS * CHUNK_BYTES,
            "Wrong bytes length for transfer to new pubdata"
        );

        let from_offset = 1;
        let from_sub_account_id_offset = from_offset + ACCOUNT_ID_BIT_WIDTH / 8;
        let token_id_offset = from_sub_account_id_offset + SUB_ACCOUNT_ID_BIT_WIDTH / 8;
        let amount_offset = token_id_offset + TOKEN_BIT_WIDTH / 8;
        let to_address_offset = amount_offset + AMOUNT_BIT_WIDTH / 8;
        let to_id_offset = to_address_offset + ETH_ADDRESS_BIT_WIDTH / 8;
        let to_sub_account_id_offset = to_id_offset + ACCOUNT_ID_BIT_WIDTH / 8;
        let fee_offset = to_sub_account_id_offset + SUB_ACCOUNT_ID_BIT_WIDTH / 8;
        let end = fee_offset + FEE_BIT_WIDTH / 8;

        let from_id = u32::from_bytes(&bytes[from_offset..from_sub_account_id_offset])
            .ok_or_else(|| format_err!("Cant get from account id from transfer to new pubdata"))?;
        let from_sub_acount_id = bytes[from_sub_account_id_offset];
        let token = u16::from_bytes(&bytes[token_id_offset..amount_offset])
            .ok_or_else(|| format_err!("Cant get token id from transfer to new pubdata"))?;
        let amount = unpack_token_amount(&bytes[amount_offset..to_address_offset])
            .ok_or_else(|| format_err!("Cant get amount from transfer to new pubdata"))?;
        let to = ZkLinkAddress::from_slice(&bytes[to_address_offset..to_id_offset])?;
        let to_id = u32::from_bytes(&bytes[to_id_offset..to_sub_account_id_offset])
            .ok_or_else(|| format_err!("Cant get to account id from transfer to new pubdata"))?;
        let to_sub_acount_id = bytes[to_sub_account_id_offset];
        let fee = unpack_fee_amount(&bytes[fee_offset..end])
            .ok_or_else(|| format_err!("Cant get fee from transfer to new pubdata"))?;

        let nonce = 0; // It is unknown from pubdata
        let ts = Default::default();

        Ok(Self {
            tx: Transfer::new(
                    AccountId(from_id),
                    to,
                    SubAccountId(from_sub_acount_id),
                    SubAccountId(to_sub_acount_id),
                    TokenId(token as u32),
                    amount,
                    fee,
                    Nonce(nonce),
                    None,
                    ts,
            ),
            from: AccountId(from_id),
            to: AccountId(to_id),
        })
    }

    pub fn get_updated_account_ids(&self) -> Vec<AccountId> {
        vec![self.from, self.to]
    }
}
