use zklink_types::block::Block;
use zklink_storage::chain::operations::records::AggType;

#[derive(Debug, Clone, Default)]
pub struct BlocksCommitOperation {
    pub last_committed_block: Block,
    pub blocks: Vec<Block>,
}

impl BlocksCommitOperation {
    pub(crate) const ACTION_TYPE: AggType = AggType::CommitBlocks;
}