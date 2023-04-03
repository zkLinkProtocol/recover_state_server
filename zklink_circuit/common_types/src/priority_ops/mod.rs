//! Definition of zklink network priority operations: operations initiated from the L1.

use std::convert::{TryInto};

use anyhow::{bail, ensure};
use num::{BigUint};
use serde::{Deserialize, Serialize};
use zklink_basic_types::{H256, SubAccountId};
use zklink_crypto::params::{
    ACCOUNT_ID_BIT_WIDTH, BALANCE_BIT_WIDTH, CHAIN_ID_BIT_WIDTH, ETH_ADDRESS_BIT_WIDTH, SUB_ACCOUNT_ID_BIT_WIDTH,
    TOKEN_BIT_WIDTH, TX_TYPE_BIT_WIDTH
};
use zklink_crypto::primitives::FromBytes;
use zklink_utils::BigUintSerdeAsRadix10Str;

use super::{
    AccountId,
    operations::{DepositOp, FullExitOp}, SerialId, TokenId, ZkLinkAddress
};

#[cfg(test)]
mod tests;

/// Deposit priority operation transfers funds from the L1 account to the desired L2 account.
/// If the target L2 account didn't exist at the moment of the operation execution, a new
/// account will be created.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct PriorityDeposit {
    /// Chain Id of the Deposit.
    pub chain_id: u8,
    /// Address of the transaction initiator's L1 account.
    pub from: ZkLinkAddress,
    /// Source token and target token of deposited from l1 to l2.
    pub l2_target_token: TokenId,
    pub l1_source_token: TokenId,
    /// The target sub-account id of depositing amount.
    pub sub_account_id: SubAccountId,
    /// Amount of tokens deposited.
    #[serde(with = "BigUintSerdeAsRadix10Str")]
    pub amount: BigUint,
    /// Address of L2 account to deposit funds to.
    pub to: ZkLinkAddress,
    /// serial id for unique tx_hash
    pub serial_id: u64,
    /// eth_hash for broker_ack
    pub tx_hash: H256,
}

/// Performs a withdrawal of funds without direct interaction with the L2 network.
/// All the balance of the desired token will be withdrawn to the provided L1 address.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct PriorityFullExit {
    pub chain_id: u8,
    pub account_id: AccountId,
    pub sub_account_id: SubAccountId,
    pub initiator: ZkLinkAddress,
    pub exit_address: ZkLinkAddress,
    pub l2_source_token: TokenId,
    pub l1_target_token: TokenId,
    pub serial_id:u64,
    /// eth_hash for broker_ack
    pub tx_hash: H256,
}

/// A set of L1 priority operations supported by the zklink network.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum ZkLinkPriorityOp {
    Deposit(PriorityDeposit),
    FullExit(PriorityFullExit),
}

impl ZkLinkPriorityOp {

    /// Parses priority operation from the Evm logs.
    pub fn parse_from_priority_queue_logs(
        pub_data: &[u8],
        op_type_id: u8,
        sender: ZkLinkAddress,
        serial_id: u64,
        tx_hash: H256,
    ) -> Result<Self, anyhow::Error> {
        // see zklink-contracts/Operations.sol
        match op_type_id {
            DepositOp::OP_CODE => {
                let pub_data_left = pub_data;

                ensure!(
                    pub_data_left.len() >= TX_TYPE_BIT_WIDTH / 8,
                    "DepositOp PubData length mismatch"
                );
                let (_, pub_data_left) = pub_data_left.split_at(TX_TYPE_BIT_WIDTH / 8);

                // chain_id
                ensure!(
                    pub_data_left.len() >= CHAIN_ID_BIT_WIDTH / 8,
                    "DepositOp PubData length mismatch"
                );
                let (chain_id, pub_data_left) = pub_data_left.split_at(CHAIN_ID_BIT_WIDTH / 8);

                // account_id
                ensure!(
                    pub_data_left.len() >= ACCOUNT_ID_BIT_WIDTH / 8,
                    "DepositOp PubData length mismatch"
                );
                let (_, pub_data_left) = pub_data_left.split_at(ACCOUNT_ID_BIT_WIDTH / 8);

                // sub_account_id
                ensure!(
                    pub_data_left.len() >= SUB_ACCOUNT_ID_BIT_WIDTH / 8,
                    "DepositOp PubData length mismatch"
                );
                let (sub_account_id, pub_data_left) = pub_data_left.split_at(SUB_ACCOUNT_ID_BIT_WIDTH / 8);

                // l1_source_token
                let (real_token, pub_data_left) = {
                    ensure!(
                        pub_data_left.len() >= TOKEN_BIT_WIDTH / 8,
                        "DepositOp PubData length mismatch"
                    );
                    let (token, left) = pub_data_left.split_at(TOKEN_BIT_WIDTH / 8);
                    (u16::from_be_bytes(token.try_into().unwrap()), left)
                };

                // l2_target_token
                let (user_token, pub_data_left) = {
                    ensure!(
                        pub_data_left.len() >= TOKEN_BIT_WIDTH / 8,
                        "DepositOp PubData length mismatch"
                    );
                    let (token, left) = pub_data_left.split_at(TOKEN_BIT_WIDTH / 8);
                    (u16::from_be_bytes(token.try_into().unwrap()), left)
                };


                // amount
                let (amount, pub_data_left) = {
                    ensure!(
                        pub_data_left.len() >= BALANCE_BIT_WIDTH / 8,
                        "PubData length mismatch"
                    );
                    let (amount, left) = pub_data_left.split_at(BALANCE_BIT_WIDTH / 8);
                    let amount = u128::from_be_bytes(amount.try_into().unwrap());
                    (BigUint::from(amount), left)
                };

                // account
                let (account, pub_data_left) = {
                    ensure!(
                        pub_data_left.len() >= ETH_ADDRESS_BIT_WIDTH / 8,
                        "DepositOp PubData length mismatch"
                    );
                    let (account, left) = pub_data_left.split_at(ETH_ADDRESS_BIT_WIDTH / 8);
                    (ZkLinkAddress::from_slice(account)?, left)
                };

                ensure!(
                    pub_data_left.is_empty(),
                    "DepositOp parse failed: input too big"
                );

                Ok(Self::Deposit(PriorityDeposit {
                    chain_id: u8::from_be_bytes(chain_id.try_into().unwrap()),
                    from: sender,
                    l1_source_token: TokenId(real_token as u32),
                    l2_target_token: TokenId(user_token as u32),
                    sub_account_id: SubAccountId(sub_account_id[0]),
                    amount,
                    to: account,
                    serial_id,
                    tx_hash,
                }))
            }
            FullExitOp::OP_CODE => {
                ensure!(
                    pub_data.len() >= TX_TYPE_BIT_WIDTH / 8,
                    "FullExitOp PubData length mismatch"
                );
                let (_, pub_data_left) = pub_data.split_at(TX_TYPE_BIT_WIDTH / 8);

                // chain_id
                ensure!(
                    pub_data_left.len() >= CHAIN_ID_BIT_WIDTH / 8,
                    "FullExitOp PubData length mismatch"
                );
                let (chain_id, pub_data_left) = pub_data_left.split_at(CHAIN_ID_BIT_WIDTH / 8);

                // account_id
                let (account_id, pub_data_left) = {
                    ensure!(
                        pub_data_left.len() >= ACCOUNT_ID_BIT_WIDTH / 8,
                        "FullExitOp PubData length mismatch"
                    );
                    let (account_id, left) = pub_data_left.split_at(ACCOUNT_ID_BIT_WIDTH / 8);
                    (u32::from_bytes(account_id).unwrap(), left)
                };

                // sub_account_id
                ensure!(
                    pub_data_left.len() >= SUB_ACCOUNT_ID_BIT_WIDTH / 8,
                    "FullExitOp PubData length mismatch"
                );
                let (sub_account_id, pub_data_left) = pub_data_left.split_at(SUB_ACCOUNT_ID_BIT_WIDTH / 8);

                // owner
                let (exit_address, pub_data_left) = {
                    ensure!(
                        pub_data_left.len() >= ETH_ADDRESS_BIT_WIDTH / 8,
                        "FullExitOp PubData length mismatch"
                    );
                    let (exit_address, left) = pub_data_left.split_at(ETH_ADDRESS_BIT_WIDTH / 8);
                    (ZkLinkAddress::from_slice(exit_address)?, left)
                };

                // l1_source_token
                let (real_token, pub_data_left) = {
                    ensure!(
                        pub_data_left.len() >= TOKEN_BIT_WIDTH / 8,
                        "FullExitOp PubData length mismatch"
                    );
                    let (token, left) = pub_data_left.split_at(TOKEN_BIT_WIDTH / 8);
                    (u16::from_be_bytes(token.try_into().unwrap()), left)
                };

                // l2_target_token
                let (user_token, pub_data_left) = {
                    ensure!(
                        pub_data_left.len() >= TOKEN_BIT_WIDTH / 8,
                        "FullExitOp PubData length mismatch"
                    );
                    let (token, left) = pub_data_left.split_at(TOKEN_BIT_WIDTH / 8);
                    (u16::from_be_bytes(token.try_into().unwrap()), left)
                };


                // amount
                ensure!(
                    pub_data_left.len() == BALANCE_BIT_WIDTH / 8,
                    "FullExitOp parse failed: input too big: {:?}",
                    pub_data_left
                );

                Ok(Self::FullExit(PriorityFullExit {
                    chain_id: u8::from_be_bytes(chain_id.try_into().unwrap()),
                    account_id: AccountId(account_id),
                    sub_account_id: SubAccountId(sub_account_id[0]),
                    initiator: sender,
                    exit_address,
                    l1_target_token: TokenId(real_token as u32),
                    l2_source_token: TokenId(user_token as u32),
                    serial_id,
                    tx_hash,
                }))
            }
            _ => {
                bail!("Unsupported priority op type");
            }
        }
    }
}

/// Priority operation description with the metadata required for server to process it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityOp {
    /// Unique ID of the priority operation.
    pub serial_id: SerialId,
    /// Priority operation.
    pub data: ZkLinkPriorityOp,
    /// Ethereum deadline block until which operation must be processed.
    pub deadline_block: u64,
    /// Block in which Ethereum transaction was included.
    pub eth_block: u64,
}
