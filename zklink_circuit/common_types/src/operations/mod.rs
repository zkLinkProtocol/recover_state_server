//! Set of all the operations supported by the zklink network.

use super::ZkLinkTx;
use anyhow::format_err;
use serde::{Deserialize, Serialize};
use zklink_crypto::params::CHUNK_BYTES;

mod change_pubkey_op;
mod deposit_op;
mod forced_exit;
mod full_exit_op;
mod noop_op;
mod order_matching_op;
mod transfer_op;
mod transfer_to_new_op;
mod withdraw_op;

pub mod basic_impl_and_params;

#[doc(hidden)]
pub use self::{
    basic_impl_and_params::{
        CHANGE_PUBKEY_CHUNK_FRS_NUMBER, DEPOSIT_CHUNK_FRS_NUMBER, FORCED_EXIT_CHUNK_FRS_NUMBER,
        FULL_EXIT_CHUNK_FRS_NUMBER, MAX_ZKLINK_TX_CHUNKS, MIN_ZKLINK_TX_CHUNKS,
        NON_PRIORITY_OP_TYPES, ORDER_MATCHING_CHUNK_FRS_NUMBER, PRIORITY_OP_TYPES,
        TRANSFER_CHUNK_FRS_NUMBER, TRANSFER_TO_NEW_CHUNK_FRS_NUMBER, WITHDRAW_CHUNK_FRS_NUMBER,
    },
    change_pubkey_op::ChangePubKeyOp,
    deposit_op::DepositOp,
    forced_exit::ForcedExitOp,
    full_exit_op::FullExitOp,
    noop_op::NoopOp,
    order_matching_op::{OrderContext, OrderMatchingOp},
    transfer_op::TransferOp,
    transfer_to_new_op::TransferToNewOp,
    withdraw_op::WithdrawOp,
};
use zklink_basic_types::{AccountId, ChainId};

pub trait GetPublicData {
    fn get_public_data(&self) -> Vec<u8>;
}

/// zklink network operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ZkLinkOp {
    Deposit(Box<DepositOp>),
    Transfer(Box<TransferOp>),
    /// Transfer to new operation is represented by `Transfer` transaction,
    /// same as `Transfer` operation. The difference is that for `TransferToNew` operation
    /// recipient account doesn't exist and has to be created.
    TransferToNew(Box<TransferToNewOp>),
    Withdraw(Box<WithdrawOp>),
    #[doc(hidden)]
    FullExit(Box<FullExitOp>),
    ChangePubKeyOffchain(Box<ChangePubKeyOp>),
    ForcedExit(Box<ForcedExitOp>),
    OrderMatching(Box<OrderMatchingOp>),
    /// `NoOp` operation cannot be directly created, but it's used to fill the block capacity.
    Noop(NoopOp),
}

impl ZkLinkOp {
    /// Returns the number of block chunks required for the operation.
    pub fn chunks(&self) -> usize {
        match self {
            ZkLinkOp::Noop(_) => NoopOp::CHUNKS,
            ZkLinkOp::Deposit(_) => DepositOp::CHUNKS,
            ZkLinkOp::TransferToNew(_) => TransferToNewOp::CHUNKS,
            ZkLinkOp::Withdraw(_) => WithdrawOp::CHUNKS,
            ZkLinkOp::Transfer(_) => TransferOp::CHUNKS,
            ZkLinkOp::FullExit(_) => FullExitOp::CHUNKS,
            ZkLinkOp::ChangePubKeyOffchain(_) => ChangePubKeyOp::CHUNKS,
            ZkLinkOp::ForcedExit(_) => ForcedExitOp::CHUNKS,
            ZkLinkOp::OrderMatching(_) => OrderMatchingOp::CHUNKS,
        }
    }

    /// Returns op_code for the operation.
    pub fn op_code(&self) -> usize {
        let op_code = match self {
            ZkLinkOp::Noop(_) => NoopOp::OP_CODE,
            ZkLinkOp::Deposit(_) => DepositOp::OP_CODE,
            ZkLinkOp::TransferToNew(_) => TransferToNewOp::OP_CODE,
            ZkLinkOp::Withdraw(_) => WithdrawOp::OP_CODE,
            ZkLinkOp::Transfer(_) => TransferOp::OP_CODE,
            ZkLinkOp::FullExit(_) => FullExitOp::OP_CODE,
            ZkLinkOp::ChangePubKeyOffchain(_) => ChangePubKeyOp::OP_CODE,
            ZkLinkOp::ForcedExit(_) => ForcedExitOp::OP_CODE,
            ZkLinkOp::OrderMatching(_) => OrderMatchingOp::OP_CODE,
        };
        op_code as usize
    }

    /// Returns the public data required for the Ethereum smart contract to commit the operation.
    pub fn public_data(&self) -> Vec<u8> {
        match self {
            ZkLinkOp::Noop(op) => op.get_public_data(),
            ZkLinkOp::Deposit(op) => op.get_public_data(),
            ZkLinkOp::TransferToNew(op) => op.get_public_data(),
            ZkLinkOp::Withdraw(op) => op.get_public_data(),
            ZkLinkOp::Transfer(op) => op.get_public_data(),
            ZkLinkOp::FullExit(op) => op.get_public_data(),
            ZkLinkOp::ChangePubKeyOffchain(op) => op.get_public_data(),
            ZkLinkOp::ForcedExit(op) => op.get_public_data(),
            ZkLinkOp::OrderMatching(op) => op.get_public_data(),
        }
    }

    /// Gets the witness required for the Ethereum smart contract.
    /// Unlike public data, some operations may not have a witness.
    ///
    /// Operations that have witness data:
    ///
    /// - `ChangePubKey`;
    pub fn eth_witness(&self) -> Option<Vec<u8>> {
        match self {
            ZkLinkOp::ChangePubKeyOffchain(op) => Some(op.get_eth_witness()),
            _ => None,
        }
    }

    /// Returns eth_witness data and data_size for operation, if any.
    ///
    /// Operations that have withdrawal data:
    ///
    /// - `Withdraw`;
    /// - `FullExit`;
    /// - `ForcedExit`.
    pub fn withdrawal_data(&self) -> Option<Vec<u8>> {
        match self {
            ZkLinkOp::Withdraw(op) => Some(op.get_withdrawal_data()),
            ZkLinkOp::FullExit(op) => Some(op.get_withdrawal_data()),
            ZkLinkOp::ForcedExit(op) => Some(op.get_withdrawal_data()),
            _ => None,
        }
    }

    /// Attempts to restore the operation from the public data committed on the Ethereum smart contract.
    pub fn from_public_data(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        let op_type: u8 = *bytes.first().ok_or_else(|| format_err!("Empty pubdata"))?;
        match op_type {
            NoopOp::OP_CODE => Ok(ZkLinkOp::Noop(NoopOp::from_public_data(bytes)?)),
            DepositOp::OP_CODE => Ok(ZkLinkOp::Deposit(Box::new(DepositOp::from_public_data(
                bytes,
            )?))),
            TransferToNewOp::OP_CODE => Ok(ZkLinkOp::TransferToNew(Box::new(
                TransferToNewOp::from_public_data(bytes)?,
            ))),
            TransferOp::OP_CODE => Ok(ZkLinkOp::Transfer(Box::new(TransferOp::from_public_data(
                bytes,
            )?))),
            WithdrawOp::OP_CODE => Ok(ZkLinkOp::Withdraw(Box::new(WithdrawOp::from_public_data(
                bytes,
            )?))),
            FullExitOp::OP_CODE => Ok(ZkLinkOp::FullExit(Box::new(FullExitOp::from_public_data(
                bytes,
            )?))),
            ChangePubKeyOp::OP_CODE => Ok(ZkLinkOp::ChangePubKeyOffchain(Box::new(
                ChangePubKeyOp::from_public_data(bytes)?,
            ))),
            ForcedExitOp::OP_CODE => Ok(ZkLinkOp::ForcedExit(Box::new(
                ForcedExitOp::from_public_data(bytes)?,
            ))),
            OrderMatchingOp::OP_CODE => Ok(ZkLinkOp::OrderMatching(Box::new(
                OrderMatchingOp::from_public_data(bytes)?,
            ))),
            _ => Err(format_err!("Wrong operation type: {}", &op_type)),
        }
    }

    /// Returns the expected number of chunks for a certain type of operation.
    pub fn public_data_length(op_type: u8) -> Result<usize, anyhow::Error> {
        match op_type {
            NoopOp::OP_CODE => Ok(NoopOp::CHUNKS),
            DepositOp::OP_CODE => Ok(DepositOp::CHUNKS),
            TransferToNewOp::OP_CODE => Ok(TransferToNewOp::CHUNKS),
            WithdrawOp::OP_CODE => Ok(WithdrawOp::CHUNKS),
            TransferOp::OP_CODE => Ok(TransferOp::CHUNKS),
            FullExitOp::OP_CODE => Ok(FullExitOp::CHUNKS),
            ChangePubKeyOp::OP_CODE => Ok(ChangePubKeyOp::CHUNKS),
            ForcedExitOp::OP_CODE => Ok(ForcedExitOp::CHUNKS),
            OrderMatchingOp::OP_CODE => Ok(OrderMatchingOp::CHUNKS),
            _ => Err(format_err!("Wrong operation type: {}", &op_type)),
        }
        .map(|chunks| chunks * CHUNK_BYTES)
    }

    /// Attempts to interpret the operation as the L2 transaction.
    pub fn try_get_tx(&self) -> Result<ZkLinkTx, anyhow::Error> {
        match self {
            ZkLinkOp::Deposit(op) => Ok(ZkLinkTx::Deposit(Box::new(op.tx.clone()))),
            ZkLinkOp::Transfer(op) => Ok(ZkLinkTx::Transfer(Box::new(op.tx.clone()))),
            ZkLinkOp::TransferToNew(op) => Ok(ZkLinkTx::Transfer(Box::new(op.tx.clone()))),
            ZkLinkOp::Withdraw(op) => Ok(ZkLinkTx::Withdraw(Box::new(op.tx.clone()))),
            ZkLinkOp::ChangePubKeyOffchain(op) => {
                Ok(ZkLinkTx::ChangePubKey(Box::new(op.tx.clone())))
            }
            ZkLinkOp::ForcedExit(op) => Ok(ZkLinkTx::ForcedExit(Box::new(op.tx.clone()))),
            ZkLinkOp::FullExit(op) => Ok(ZkLinkTx::FullExit(Box::new(op.tx.clone()))),
            ZkLinkOp::OrderMatching(op) => Ok(ZkLinkTx::OrderMatching(Box::new(op.tx.clone()))),
            _ => Err(format_err!("Wrong tx type")),
        }
    }

    /// Returns the list of account IDs affected by this operation.
    pub fn get_updated_account_ids(&self) -> Vec<AccountId> {
        match self {
            ZkLinkOp::Noop(op) => op.get_updated_account_ids(),
            ZkLinkOp::Deposit(op) => op.get_updated_account_ids(),
            ZkLinkOp::TransferToNew(op) => op.get_updated_account_ids(),
            ZkLinkOp::Withdraw(op) => op.get_updated_account_ids(),
            ZkLinkOp::Transfer(op) => op.get_updated_account_ids(),
            ZkLinkOp::FullExit(op) => op.get_updated_account_ids(),
            ZkLinkOp::ChangePubKeyOffchain(op) => op.get_updated_account_ids(),
            ZkLinkOp::ForcedExit(op) => op.get_updated_account_ids(),
            ZkLinkOp::OrderMatching(op) => op.get_updated_account_ids(),
        }
    }

    /// Keep same with `checkOnchainOp` in ZkLink.sol
    pub fn is_onchain_operation(&self) -> bool {
        matches!(
            self,
            &ZkLinkOp::Deposit(_)
                | &ZkLinkOp::Withdraw(_)
                | &ZkLinkOp::FullExit(_)
                | &ZkLinkOp::ChangePubKeyOffchain(_)
                | &ZkLinkOp::ForcedExit(_)
        )
    }

    pub fn is_priority_operation(&self) -> bool {
        matches!(self, &ZkLinkOp::Deposit(_) | &ZkLinkOp::FullExit(_))
    }

    /// Keep same with `checkOnchainOp` in ZkLink.sol
    pub fn is_local_onchain_operation(&self, chain_id: ChainId) -> bool {
        match self {
            ZkLinkOp::Deposit(op) => op.tx.from_chain_id == chain_id,
            ZkLinkOp::Withdraw(op) => op.tx.to_chain_id == chain_id,
            ZkLinkOp::FullExit(op) => op.tx.to_chain_id == chain_id,
            ZkLinkOp::ChangePubKeyOffchain(op) => op.tx.chain_id == chain_id,
            ZkLinkOp::ForcedExit(op) => op.tx.to_chain_id == chain_id,
            _ => false,
        }
    }

    /// Keep same with `checkOnchainOp` in ZkLink.sol
    pub fn get_onchain_operation_chain_id(&self) -> u8 {
        match self {
            ZkLinkOp::Deposit(op) => *op.tx.from_chain_id,
            ZkLinkOp::Withdraw(op) => *op.tx.to_chain_id,
            ZkLinkOp::FullExit(op) => *op.tx.to_chain_id,
            ZkLinkOp::ChangePubKeyOffchain(op) => *op.tx.chain_id,
            ZkLinkOp::ForcedExit(op) => *op.tx.to_chain_id,
            _ => 0, // 0 is a invalid chain id
        }
    }

    /// Keep same with `checkOnchainOp` in ZkLink.sol
    pub fn is_processable_onchain_operation(&self, chain_id: ChainId) -> bool {
        match self {
            ZkLinkOp::Withdraw(op) => op.tx.to_chain_id == chain_id,
            ZkLinkOp::FullExit(op) => op.tx.to_chain_id == chain_id,
            ZkLinkOp::ForcedExit(op) => op.tx.to_chain_id == chain_id,
            _ => false,
        }
    }
}
