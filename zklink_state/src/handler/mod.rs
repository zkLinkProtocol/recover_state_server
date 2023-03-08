use crate::state::OpSuccess;
use zklink_types::{AccountUpdates, ZkLinkOp};

mod change_pubkey;
mod deposit;
mod forced_exit;
mod full_exit;
mod transfer;
mod withdraw;
mod order_matching;


/// TxHandler trait encapsulates the logic of each individual transaction
/// handling. By transactions we assume both zkLink network transactions,
/// and priority operations (initiated by invoking the Ethereum smart contract
/// methods).
///
/// Template parameter `Tx` represents a type of transaction being handled.
/// It has to be a template parameter rather than an associated type, so
/// there may be more than one trait implementation for a structure.
///
/// We need to use `assert!` to check l1 transaction condition and panic if condition is not satisfied.
/// And on the contrary, we need to use `ensure!` to check l2 transaction and mark it to be failed if condition is not satisfied.
pub trait TxHandler<Tx> {
    /// Operation wrapper for the transaction.
    type Op: Into<ZkLinkOp>;

    /// Creates an operation wrapper from the given transaction.
    fn create_op(&self, tx: Tx) -> Result<Self::Op, anyhow::Error>;

    fn apply_tx(&mut self, tx: Tx) -> Result<OpSuccess, anyhow::Error> {
        let mut op = self.create_op(tx)?;

        let updates = self.apply_op(&mut op)?;
        Ok(OpSuccess {
            updates,
            executed_op: op.into(),
        })
    }

    /// Applies the operation.
    fn apply_op(
        &mut self,
        op: &mut Self::Op,
    ) -> Result<AccountUpdates, anyhow::Error>;

    /// Applies the operation unsafely for recovering state.
    fn unsafe_apply_op(
        &mut self,
        _op: &mut Self::Op,
    ) -> Result<AccountUpdates, anyhow::Error>{
        Ok(vec![])
    }
}
