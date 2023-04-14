use crate::*;

// tx
impl Deposit {
    pub const TX_TYPE: u8 = 0x01;
}
impl Withdraw {
    pub const TX_TYPE: u8 = 0x03;
}
impl Transfer {
    pub const TX_TYPE: u8 = 0x04;
}
impl FullExit {
    pub const TX_TYPE: u8 = 0x05;
}
impl ChangePubKey {
    pub const TX_TYPE: u8 = 0x06;
}
impl ForcedExit {
    pub const TX_TYPE: u8 = 0x07;
}
impl OrderMatching {
    pub const TX_TYPE: u8 = 0x08;
}

// op
impl DepositOp {
    pub const OP_CODE: u8 = Deposit::TX_TYPE;
    pub const CHUNKS: usize = 3;
}
impl TransferToNewOp {
    pub const OP_CODE: u8 = 0x02;
    pub const CHUNKS: usize = 3;
}
impl WithdrawOp {
    pub const OP_CODE: u8 = Withdraw::TX_TYPE;
    pub const CHUNKS: usize = 3;
}
impl TransferOp {
    pub const OP_CODE: u8 = Transfer::TX_TYPE;
    pub const CHUNKS: usize = 2;
}
impl FullExitOp {
    pub const OP_CODE: u8 = FullExit::TX_TYPE;
    pub const CHUNKS: usize = 3;
}
impl ChangePubKeyOp {
    pub const OP_CODE: u8 = ChangePubKey::TX_TYPE;
    pub const CHUNKS: usize = 3;
}
impl ForcedExitOp {
    pub const OP_CODE: u8 = ForcedExit::TX_TYPE;
    pub const CHUNKS: usize = 3;
}
impl OrderMatchingOp {
    pub const OP_CODE: u8 = OrderMatching::TX_TYPE;
    pub const CHUNKS: usize = 4;
}

// The number of Fr's required for pubdata of every op
pub const DEPOSIT_CHUNK_FRS_NUMBER: usize = 2;
pub const TRANSFER_TO_NEW_CHUNK_FRS_NUMBER: usize = 2;
pub const WITHDRAW_CHUNK_FRS_NUMBER: usize = 2;
pub const TRANSFER_CHUNK_FRS_NUMBER: usize = 2;
pub const FULL_EXIT_CHUNK_FRS_NUMBER: usize = 2;
pub const CHANGE_PUBKEY_CHUNK_FRS_NUMBER: usize = 2;
pub const FORCED_EXIT_CHUNK_FRS_NUMBER: usize = 2;
pub const ORDER_MATCHING_CHUNK_FRS_NUMBER: usize = 3;

// SHOULD be reset on change of tx.chunks
pub const MAX_ZKLINK_TX_CHUNKS: usize = 4;
pub const MIN_ZKLINK_TX_CHUNKS: usize = 2;

// SHOULD be reset on change of ops
pub const PRIORITY_OP_TYPES: [u8; 2] = [DepositOp::OP_CODE, FullExitOp::OP_CODE];
pub const NON_PRIORITY_OP_TYPES: [u8; 6] = [
    ChangePubKeyOp::OP_CODE,
    TransferOp::OP_CODE,
    TransferToNewOp::OP_CODE,
    WithdrawOp::OP_CODE,
    ForcedExitOp::OP_CODE,
    OrderMatchingOp::OP_CODE,
];

impl From<NoopOp> for ZkLinkOp {
    fn from(op: NoopOp) -> Self {
        Self::Noop(op)
    }
}

impl From<DepositOp> for ZkLinkOp {
    fn from(op: DepositOp) -> Self {
        Self::Deposit(Box::new(op))
    }
}

impl From<WithdrawOp> for ZkLinkOp {
    fn from(op: WithdrawOp) -> Self {
        Self::Withdraw(Box::new(op))
    }
}

impl From<TransferOp> for ZkLinkOp {
    fn from(op: TransferOp) -> Self {
        Self::Transfer(Box::new(op))
    }
}

impl From<TransferToNewOp> for ZkLinkOp {
    fn from(op: TransferToNewOp) -> Self {
        Self::TransferToNew(Box::new(op))
    }
}

impl From<FullExitOp> for ZkLinkOp {
    fn from(op: FullExitOp) -> Self {
        Self::FullExit(Box::new(op))
    }
}

impl From<ChangePubKeyOp> for ZkLinkOp {
    fn from(op: ChangePubKeyOp) -> Self {
        Self::ChangePubKeyOffchain(Box::new(op))
    }
}

impl From<ForcedExitOp> for ZkLinkOp {
    fn from(op: ForcedExitOp) -> Self {
        Self::ForcedExit(Box::new(op))
    }
}

impl From<OrderMatchingOp> for ZkLinkOp {
    fn from(op: OrderMatchingOp) -> Self {
        Self::OrderMatching(Box::new(op))
    }
}

pub fn priority_op_types() -> Vec<i16> {
    PRIORITY_OP_TYPES.iter().map(|e| *e as i16).collect()
}

pub fn non_priority_op_types() -> Vec<i16> {
    NON_PRIORITY_OP_TYPES.iter().map(|e| *e as i16).collect()
}
