//! Module with additional conversion methods for the storage records.
//! These methods are only needed for the `block` module, so they're kept in a
//! private module.
// External imports
use num::bigint::ToBigInt;
use sqlx::types::BigDecimal;
// Workspace imports
use zklink_types::block::FailedExecutedTx;
use zklink_types::{block::ExecutedTx, BlockNumber, ZkLinkOp, ZkLinkTx};
// Local imports
use crate::chain::operations::records::StoredAggregatedOperation;
use crate::chain::operations::records::{NewExecutedTransaction, StoredExecutedTransaction};

impl StoredExecutedTransaction {
    pub fn into_executed_tx(self) -> Result<ExecutedTx, anyhow::Error> {
        let tx: ZkLinkTx = serde_json::from_value(self.tx_data).expect("Unparsable tx in tx_data");
        let franklin_op: Option<ZkLinkOp> =
            serde_json::from_value(self.operation.unwrap()).expect("Unparsable ZkLinkOp in db");
        Ok(ExecutedTx {
            tx,
            success: self.success,
            // the sql has set condition success=true, which ensure op must exist
            op: franklin_op.unwrap(),
            fail_reason: self.fail_reason,
            block_index: Some(self.block_index as u32),
            created_at: self.created_at,
        })
    }
}

impl NewExecutedTransaction {
    pub fn prepare_stored_priority_tx(exec_tx: ExecutedTx, block: BlockNumber) -> Self {
        let tx_data = serde_json::to_value(exec_tx.tx.clone()).unwrap();
        let operation = serde_json::to_value(exec_tx.op.clone()).unwrap();
        let op = exec_tx.op;
        let op_type = op.op_code() as i16;
        let chain_id = op.get_onchain_operation_chain_id() as i16;
        let nonce = if matches!(op, ZkLinkOp::Deposit(_) | ZkLinkOp::FullExit(_)) {
            match &op {
                ZkLinkOp::Deposit(op) => op.tx.serial_id as i64,
                ZkLinkOp::FullExit(op) => op.tx.serial_id as i64,
                _ => unreachable!(),
            }
        } else {
            *exec_tx.tx.nonce() as i64
        };
        let amount = match op {
            ZkLinkOp::Deposit(op) => op.tx.amount,
            ZkLinkOp::FullExit(op) => op.exit_amount,
            _ => unreachable!()
        };

        let amount = BigDecimal::from(amount.to_bigint().unwrap());
        let mut block_index = exec_tx.block_index.map(|idx| idx as i32);
        if block_index.is_none() {
            block_index = Some(0);
        }
        Self {
            op_type,
            chain_id,
            block_number: i64::from(*block),
            tx_hash: exec_tx.tx.hash().as_ref().to_vec(),
            operation,
            success: exec_tx.success,
            fail_reason: exec_tx.fail_reason,
            block_index,
            nonce,
            amount,
            tx_data,
        }
    }

    pub fn prepare_stored_tx(exec_tx: ExecutedTx, block: BlockNumber) -> Self {
        let tx_data = serde_json::to_value(exec_tx.tx.clone()).unwrap();
        let operation = serde_json::to_value(exec_tx.op.clone()).unwrap();
        let op = exec_tx.op;
        let op_type = op.op_code() as i16;
        let chain_id = op.get_onchain_operation_chain_id() as i16;
        let amount = match op {
            ZkLinkOp::Deposit(op) => op.tx.amount,
            ZkLinkOp::Transfer(op) => op.tx.amount,
            ZkLinkOp::TransferToNew(op) => op.tx.amount,
            ZkLinkOp::Withdraw(op) => op.tx.amount,
            ZkLinkOp::FullExit(op) => op.exit_amount,
            ZkLinkOp::ChangePubKeyOffchain(_) => Default::default(),
            ZkLinkOp::ForcedExit(op) => op.withdraw_amount,
            ZkLinkOp::OrderMatching(op) => op.tx.expect_base_amount,
            ZkLinkOp::Noop(_) => Default::default(),
        };

        let amount = BigDecimal::from(amount.to_bigint().unwrap());
        let mut block_index = exec_tx.block_index.map(|idx| idx as i32);
        if block_index.is_none() {
            block_index = Some(0);
        }
        Self {
            op_type,
            chain_id,
            block_number: i64::from(*block),
            tx_hash: exec_tx.tx.hash().as_ref().to_vec(),
            operation,
            success: exec_tx.success,
            fail_reason: exec_tx.fail_reason,
            block_index,
            nonce: *exec_tx.tx.nonce() as i64,
            amount,
            tx_data,
        }
    }

    pub fn prepare_stored_failed_tx(exec_tx: FailedExecutedTx, block: BlockNumber) -> Self {
        let amount: BigDecimal = Default::default();
        let op: Option<ZkLinkOp> = None;
        let tx_data = serde_json::to_value(exec_tx.tx.clone()).unwrap();
        let operation = serde_json::to_value(op).unwrap();

        let block_index = Some(0);
        Self {
            op_type: Default::default(),
            chain_id: Default::default(),
            block_number: i64::from(*block),
            tx_hash: exec_tx.tx.hash().as_ref().to_vec(),
            operation,
            success: exec_tx.success,
            fail_reason: exec_tx.fail_reason,
            block_index,
            nonce: *exec_tx.tx.nonce() as i64,
            amount,
            tx_data,
        }
    }
}

impl StoredAggregatedOperation {
    pub fn get_aggregate_operation_info(self) -> (i64, (i64, i64)) {
        (self.id, (self.from_block, self.to_block))
    }
}
