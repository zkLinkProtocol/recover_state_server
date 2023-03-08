pub mod account;
pub mod block;
pub mod operations;
pub mod state;
pub mod proof;
use super::StorageProcessor;

/// `ChainIntermediator` is a structure providing methods to
/// obtain schemas declared in the `chain` module.
#[derive(Debug)]
pub struct ChainIntermediator<'a, 'c>(pub &'a mut StorageProcessor<'c>);

impl<'a, 'c> ChainIntermediator<'a, 'c> {
    pub fn account_schema(self) -> account::AccountSchema<'a, 'c> {
        account::AccountSchema(self.0)
    }

    pub fn block_schema(self) -> block::BlockSchema<'a, 'c> {
        block::BlockSchema(self.0)
    }

    pub fn operations_schema(self) -> operations::OperationsSchema<'a, 'c> {
        operations::OperationsSchema(self.0)
    }

    pub fn state_schema(self) -> state::StateSchema<'a, 'c> {
        state::StateSchema(self.0)
    }

    pub fn proof_schema(self) -> state::StateSchema<'a, 'c> {
        state::StateSchema(self.0)
    }

}
