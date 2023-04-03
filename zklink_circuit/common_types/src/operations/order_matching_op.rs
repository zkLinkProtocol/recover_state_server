use anyhow::{ensure, format_err};
use num::{BigUint, ToPrimitive, One, CheckedMul, CheckedDiv};
use serde::{Deserialize, Serialize};

use zklink_basic_types::{AccountId, Nonce, SlotId, SubAccountId, TokenId};
use zklink_crypto::params::{
    ACCOUNT_ID_BIT_WIDTH, AMOUNT_BIT_WIDTH, BALANCE_BIT_WIDTH, CHUNK_BYTES, FEE_BIT_WIDTH, ORDER_NONCE_BIT_WIDTH,
    SLOT_BIT_WIDTH, TOKEN_BIT_WIDTH, FEE_RATIO_BIT_WIDTH, SUB_ACCOUNT_ID_BIT_WIDTH, precision_magnified
};
use zklink_crypto::primitives::FromBytes;

use crate::{Order, OrderMatching};
use crate::helpers::{pack_fee_amount, pack_token_amount, unpack_fee_amount, unpack_token_amount};
use crate::operations::GetPublicData;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct OrderMatchingOp {
    pub tx: OrderMatching,
    pub maker_sell_amount: BigUint,
    pub taker_sell_amount: BigUint,
    pub maker_context: OrderContext,
    pub taker_context: OrderContext,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrderContext{
    pub residue: BigUint,
}

impl GetPublicData for OrderMatchingOp{
    fn get_public_data(&self) -> Vec<u8> {
        let (maker_sell_token, taker_sell_token) =
            if self.tx.maker.is_sell.is_one() {
                (self.tx.maker.base_token_id, self.tx.maker.quote_token_id)
            } else {
                (self.tx.maker.quote_token_id, self.tx.maker.base_token_id)
            };

        let mut data = vec![Self::OP_CODE]; // opcode
        data.extend_from_slice(&self.tx.sub_account_id.to_be_bytes()); // 2
        data.extend_from_slice(&self.tx.maker.account_id.to_be_bytes());
        data.extend_from_slice(&self.tx.taker.account_id.to_be_bytes());
        data.extend_from_slice(&self.tx.account_id.to_be_bytes()); // 14
        data.extend_from_slice(&(*self.tx.maker.slot_id as u16).to_be_bytes());
        data.extend_from_slice(&(*self.tx.taker.slot_id as u16).to_be_bytes()); // 18
        data.extend_from_slice(&(*maker_sell_token as u16).to_be_bytes());
        data.extend_from_slice(&(*taker_sell_token as u16).to_be_bytes());
        data.extend_from_slice(&(*self.tx.fee_token as u16).to_be_bytes()); // 24
        data.extend_from_slice(&pack_token_amount(&self.tx.maker.amount));
        data.extend_from_slice(&pack_token_amount(&self.tx.taker.amount)); //34
        data.extend_from_slice(&pack_fee_amount(&self.tx.fee)); // 36
        data.extend_from_slice(&self.tx.maker.fee_ratio1.to_be_bytes()); //
        data.extend_from_slice(&self.tx.taker.fee_ratio2.to_be_bytes()); // 38
        data.extend_from_slice(&self.maker_sell_amount.to_u128().unwrap().to_be_bytes());
        data.extend_from_slice(&self.taker_sell_amount.to_u128().unwrap().to_be_bytes()); // 70
        data.extend_from_slice(&self.tx.maker.nonce.to_be_bytes()[1..]);
        data.extend_from_slice(&self.tx.taker.nonce.to_be_bytes()[1..]); // 76
        data.resize(Self::CHUNKS * CHUNK_BYTES, 0x00);
        data
    }
}

impl OrderMatchingOp {
    pub fn from_public_data(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        ensure!(
            bytes.len() == Self::CHUNKS * CHUNK_BYTES,
            "Wrong bytes length for remove liquidity pubdata"
        );

        let sub_account_id_offset = 1;
        let accounts_offset = sub_account_id_offset + SUB_ACCOUNT_ID_BIT_WIDTH / 8;
        let slots_offset = accounts_offset + ACCOUNT_ID_BIT_WIDTH * 3 / 8;
        let tokens_offset = slots_offset + SLOT_BIT_WIDTH * 2 / 8;
        let amounts_offset = tokens_offset + TOKEN_BIT_WIDTH * 3 / 8;
        let fee_offset = amounts_offset + AMOUNT_BIT_WIDTH * 2 / 8;
        let fee_ratios_offset = fee_offset + FEE_BIT_WIDTH / 8;
        let expect_amounts_offset = fee_ratios_offset + FEE_RATIO_BIT_WIDTH * 2 / 8;
        let nonces_offset = expect_amounts_offset + BALANCE_BIT_WIDTH * 2 / 8;

        let read_slot = |offset| {
            u16::from_bytes(&bytes[offset..offset + SLOT_BIT_WIDTH / 8])
                .ok_or_else(||format_err!("Cant get slot from OrderMatching pubdata"))
                .map(|b|b as u32)
        };
        let read_token = |offset| {
            u32::from_bytes(&bytes[offset..offset + TOKEN_BIT_WIDTH / 8])
                .ok_or_else(||format_err!("Cant get token from OrderMatching pubdata"))
        };
        let read_account = |offset| {
            u32::from_bytes(&bytes[offset..offset + ACCOUNT_ID_BIT_WIDTH / 8])
                .ok_or_else(|| format_err!("Cant get from account id from OrderMatching pubdata"))
        };
        let read_amount = |offset| {
            unpack_token_amount(&bytes[offset..offset + AMOUNT_BIT_WIDTH / 8])
                .ok_or_else(||format_err!("Cant get amount from OrderMatching pubdata"))
        };
        let read_expect_amount = |offset| {
            BigUint::from_bytes_be(&bytes[offset..offset + BALANCE_BIT_WIDTH / 8])
        };
        let read_nonce = |offset| {
            u32::from_bytes(&bytes[offset..offset + ORDER_NONCE_BIT_WIDTH / 8])
                .ok_or_else(|| format_err!("Cant get from nonce from OrderMatching pubdata"))
        };

        let sub_account_id = SubAccountId(bytes[sub_account_id_offset]);
        let maker_account_id = AccountId(read_account(accounts_offset)?);
        let taker_account_id = AccountId(read_account(accounts_offset + ACCOUNT_ID_BIT_WIDTH / 8)?);
        let submitter_account_id = AccountId(read_account(accounts_offset + ACCOUNT_ID_BIT_WIDTH * 2 / 8)?);
        let maker_slot_id = SlotId(read_slot(slots_offset)?);
        let taker_slot_id = SlotId(read_slot(slots_offset + SLOT_BIT_WIDTH / 8)?);
        let maker_sell_token = TokenId(read_token(tokens_offset)?);
        let taker_sell_token = TokenId(read_token(tokens_offset + TOKEN_BIT_WIDTH / 8)?);
        let fee_token = TokenId(read_token(tokens_offset + TOKEN_BIT_WIDTH * 2 / 8)?);
        let fee = unpack_fee_amount(&bytes[fee_offset..fee_offset + FEE_BIT_WIDTH / 8])
            .ok_or_else(||format_err!("OrderMatchingOpError::CannotGetFee"))?;
        let fee_ratio1 = bytes[fee_ratios_offset];
        let fee_ratio2 = bytes[fee_ratios_offset + FEE_RATIO_BIT_WIDTH / 8];
        let maker_sell_amount = read_expect_amount(expect_amounts_offset);
        let taker_sell_amount = read_expect_amount(expect_amounts_offset + BALANCE_BIT_WIDTH / 8);
        let maker_nonce = Nonce(read_nonce(nonces_offset)?);
        let taker_nonce = Nonce(read_nonce(nonces_offset + ORDER_NONCE_BIT_WIDTH / 8)?);
        let maker_amount = read_amount(amounts_offset)?;
        let taker_amount = read_amount(amounts_offset + AMOUNT_BIT_WIDTH / 8)?;

        // Todo: add 1 bit for selecting base token and quote token in the public data of OrderMatchingOp.
        // Assume the maker is Seller, take is Buyer, either assumption is the same for state changes.
        let (
            base_token_id,quote_token_id,
            expect_base_amount,expect_quote_amount
        ) = if maker_sell_token < taker_sell_token{
            (
                taker_sell_token, maker_sell_token,
                taker_sell_amount.clone(), maker_sell_amount.clone()
            )
        } else {
            (
                maker_sell_token, taker_sell_token,
                maker_sell_amount.clone(), taker_sell_amount.clone()
            )
        };
        let maker_is_sell = maker_sell_token > taker_sell_token;
        let taker_is_sell = !maker_is_sell;
        let price = expect_quote_amount
            .checked_mul(&precision_magnified())
            .unwrap()
            .checked_div(&expect_base_amount)
            .unwrap_or_default();
        let maker = Order {
            account_id: maker_account_id,
            sub_account_id,
            slot_id: maker_slot_id,
            nonce: maker_nonce,
            amount: maker_amount,
            base_token_id,
            quote_token_id,
            price: price.clone(),
            is_sell: maker_is_sell as u8,
            fee_ratio1,
            ..Default::default()
        };

        let taker = Order {
            account_id: taker_account_id,
            sub_account_id,
            slot_id: taker_slot_id,
            nonce: taker_nonce,
            amount: taker_amount,
            base_token_id,
            quote_token_id,
            price,
            is_sell: taker_is_sell as u8,
            fee_ratio2,
            ..Default::default()
        };

        Ok(Self {
            tx: OrderMatching::new(
                submitter_account_id,
                sub_account_id,
                taker,
                maker,
                fee,
                fee_token,
                expect_base_amount,
                expect_quote_amount,
                None,
            ),
            maker_sell_amount,
            taker_sell_amount,
            ..Default::default()
        })
    }

    pub fn get_updated_account_ids(&self) -> Vec<AccountId> {
        vec![
            self.tx.account_id,
            self.tx.maker.account_id,
            self.tx.taker.account_id,
        ]
    }

}
