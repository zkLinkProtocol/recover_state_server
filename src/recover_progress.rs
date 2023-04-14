use offchain_recover_state::contract::ZkLinkContract;
use offchain_recover_state::get_fully_on_chain_zklink_contract;
use recover_state_config::RecoverStateConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use zklink_types::BlockNumber;

#[derive(Debug, Clone)]
pub struct RecoverProgress {
    current_sync_height: Arc<RwLock<BlockNumber>>,
    total_verified_block: BlockNumber,
}

impl RecoverProgress {
    pub async fn new(config: &RecoverStateConfig) -> Self {
        let (_, zklink_contract) = get_fully_on_chain_zklink_contract(config);
        let total_verified_block = zklink_contract
            .get_total_verified_blocks()
            .await
            .unwrap()
            .into();
        Self {
            current_sync_height: Arc::new(RwLock::new(0.into())),
            total_verified_block,
        }
    }

    pub(crate) async fn update_progress(&self, block_height: BlockNumber) {
        *self.current_sync_height.write().await = block_height;
    }

    pub(crate) async fn is_completed_state(&self) -> bool {
        *self.current_sync_height.read().await == self.total_verified_block
    }

    pub(crate) async fn get_progress(&self) -> Progress {
        let current_block = *self.current_sync_height.read().await;
        Progress {
            current_block,
            total_verified_block: self.total_verified_block,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Progress {
    current_block: BlockNumber,
    total_verified_block: BlockNumber,
}
