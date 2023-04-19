use offchain_recover_state::contract::ZkLinkContract;
use offchain_recover_state::get_fully_on_chain_zklink_contract;
use recover_state_config::RecoverStateConfig;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, warn};
use zklink_storage::ConnectionPool;
use zklink_types::BlockNumber;

#[derive(Debug)]
pub struct RecoverProgress {
    pub(crate) current_sync_height: AtomicU32,
    pub(crate) total_verified_block: BlockNumber,
}

impl RecoverProgress {
    pub async fn from_config(config: &RecoverStateConfig) -> Self {
        let conn_pool = ConnectionPool::new(config.db.url.clone(), config.db.pool_size);
        let mut storage = conn_pool.access_storage_with_retry().await;
        let verified_block_num = storage
            .chain()
            .block_schema()
            .get_last_block_number()
            .await
            .expect("Failed to get last verified block number from database");

        let (_, zklink_contract) = get_fully_on_chain_zklink_contract(config);
        let total_verified_block = zklink_contract
            .get_total_verified_blocks()
            .await
            .expect("Failed to get total verified blocks from zklink contract")
            .into();

        Self {
            current_sync_height: AtomicU32::new(verified_block_num as u32),
            total_verified_block,
        }
    }

    pub async fn sync_from_database(&self, conn_pool: &ConnectionPool) {
        let mut ticker = interval(Duration::from_secs(1));
        let mut storage = conn_pool.access_storage_with_retry().await;
        info!("Sync recovering state started!");
        loop {
            if self.is_completed() {
                break;
            }

            ticker.tick().await;

            match storage.chain().block_schema().get_last_block_number().await {
                Ok(verified_block_num) => self.update_progress(verified_block_num.into()),
                Err(e) => warn!("Failed to get last block number:{}", e),
            }
        }
        info!("Recovering state completed!");
    }

    pub(crate) fn update_progress(&self, block_height: BlockNumber) {
        self.current_sync_height
            .store(block_height.into(), Ordering::Relaxed);
    }

    pub(crate) fn is_completed(&self) -> bool {
        let current_height = self.current_sync_height.load(Ordering::Relaxed);
        current_height == *self.total_verified_block
    }

    pub(crate) fn get_progress(&self) -> Progress {
        let current_block = self.current_sync_height.load(Ordering::Relaxed).into();
        Progress {
            current_block,
            total_verified_block: self.total_verified_block,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Progress {
    pub(crate) current_block: BlockNumber,
    pub(crate) total_verified_block: BlockNumber,
}
