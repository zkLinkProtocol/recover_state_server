use crate::helpers::{pack_fee_amount, unpack_fee_amount};
use crate::tx::ChangePubKey;
use crate::{PubKeyHash, AccountId, Nonce, TokenId, ZkLinkAddress};
use anyhow::{ensure, format_err};
use serde::{Deserialize, Serialize};
use zklink_crypto::params::{ACCOUNT_ID_BIT_WIDTH, ETH_ADDRESS_BIT_WIDTH, CHUNK_BYTES, NONCE_BIT_WIDTH, TOKEN_BIT_WIDTH, SUB_ACCOUNT_ID_BIT_WIDTH, CHAIN_ID_BIT_WIDTH, FEE_BIT_WIDTH, NEW_PUBKEY_HASH_WIDTH};
use zklink_crypto::primitives::FromBytes;
use super::GetPublicData;

/// ChangePubKey operation. For details, see the documentation of [`ZkLinkOp`](./operations/enum.ZkLinkOp.html).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePubKeyOp {
    pub tx: ChangePubKey,
    pub account_id: AccountId,
    pub address: ZkLinkAddress,
}

impl GetPublicData for ChangePubKeyOp{
    fn get_public_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.push(Self::OP_CODE); // opcode
        data.push(*self.tx.chain_id);
        data.extend_from_slice(&self.tx.account_id.to_be_bytes());
        data.extend_from_slice(&self.tx.sub_account_id.to_be_bytes());
        data.extend_from_slice(&self.tx.new_pk_hash.data);
        data.extend_from_slice(&self.address.as_bytes());
        data.extend_from_slice(&self.tx.nonce.to_be_bytes());
        data.extend_from_slice(&(*self.tx.fee_token as u16).to_be_bytes());
        data.extend_from_slice(&pack_fee_amount(&self.tx.fee));
        data.resize(Self::CHUNKS * CHUNK_BYTES, 0x00);
        data
    }
}

impl ChangePubKeyOp {
    pub fn get_eth_witness(&self) -> Vec<u8> {
        self.tx.eth_auth_data.get_eth_witness()
    }

    pub fn from_public_data(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        ensure!(
            bytes.len() == Self::CHUNKS * CHUNK_BYTES,
            "Wrong bytes length for changepubkey pubdata"
        );

        let chain_id_offset = 1;
        let account_id_offset = chain_id_offset + CHAIN_ID_BIT_WIDTH / 8;
        let sub_account_id_offset = account_id_offset + ACCOUNT_ID_BIT_WIDTH / 8;
        let pk_hash_offset = sub_account_id_offset + SUB_ACCOUNT_ID_BIT_WIDTH / 8;
        let address_offset = pk_hash_offset + NEW_PUBKEY_HASH_WIDTH / 8;
        let nonce_offset = address_offset + ETH_ADDRESS_BIT_WIDTH / 8;
        let fee_token_offset = nonce_offset + NONCE_BIT_WIDTH / 8;
        let fee_offset = fee_token_offset + TOKEN_BIT_WIDTH / 8;
        let end = fee_offset + FEE_BIT_WIDTH / 8;

        let chain_id = bytes[chain_id_offset];
        let account_id = u32::from_bytes(&bytes[account_id_offset..sub_account_id_offset])
            .ok_or_else(|| format_err!("Change pubkey offchain, fail to get account id"))?;
        let sub_account_id = bytes[sub_account_id_offset];
        let new_pk_hash = PubKeyHash::from_bytes(&bytes[pk_hash_offset..address_offset])?;
        let address = ZkLinkAddress::from_slice(&bytes[address_offset..nonce_offset])?;
        let nonce = u32::from_bytes(&bytes[nonce_offset..fee_token_offset])
            .ok_or_else(|| format_err!("Change pubkey offchain, fail to get nonce"))?;
        let fee_token = u16::from_bytes(&bytes[fee_token_offset..fee_offset])
            .ok_or_else(|| format_err!("Change pubkey offchain, fail to get fee token id"))?;
        let fee = unpack_fee_amount(&bytes[fee_offset..end])
            .ok_or_else(|| format_err!("Change pubkey offchain, fail to get fee"))?;

        Ok(ChangePubKeyOp {
            tx: ChangePubKey::new(
                chain_id.into(),
                account_id.into(),
                sub_account_id.into(),
                new_pk_hash,
                TokenId(fee_token as u32),
                fee,
                Nonce(nonce),
                Default::default(),
                None,
                Default::default(),
            ),
            account_id: AccountId(account_id),
            address
        })
    }

    pub fn get_updated_account_ids(&self) -> Vec<AccountId> {
        vec![self.account_id]
    }
}
