pub(crate) mod events_state;

use crate::contract::ZkLinkContractVersion;
use std::cmp::Ordering;
use zklink_types::{BlockNumber, H256};

/// Rollup contract event type describing the state of the corresponding Rollup block
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EventType {
    /// Committed event
    Committed,
    /// Verified event
    Verified,
}

/// Rollup Contract event description
#[derive(Debug, Copy, Clone, Eq)]
pub struct BlockEvent {
    /// Start rollup block number
    pub start_block_num: BlockNumber,
    /// End rollup block number
    pub end_block_num: BlockNumber,
    /// Layer1 transaction hash
    pub transaction_hash: H256,
    /// Rollup block type
    pub block_type: EventType,
    /// Version of ZkLink contract
    pub contract_version: ZkLinkContractVersion,
}

impl BlockEvent {
    pub fn blocks_num(&self) -> usize {
        (*self.end_block_num - *self.start_block_num + 1) as usize
    }
}

impl PartialOrd for BlockEvent {
    fn partial_cmp(&self, other: &BlockEvent) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BlockEvent {
    fn cmp(&self, other: &BlockEvent) -> Ordering {
        self.end_block_num.cmp(&other.end_block_num)
    }
}

impl PartialEq for BlockEvent {
    fn eq(&self, other: &BlockEvent) -> bool {
        self.end_block_num == other.end_block_num
    }
}
