pub mod evm;

pub use evm::EvmTokenEvents;

#[async_trait::async_trait]
pub trait UpdateTokenEvents: Send + Sync {
    /// Check that the latest block has been reached
    fn reached_latest_block(&self, latest_block: u64) -> bool;

    /// Get the newest block height of the layer1 at present
    async fn block_number(&self) -> anyhow::Result<u64>;

    /// Update all token events of the layer1
    async fn update_token_events(&mut self, latest_block: u64) -> anyhow::Result<u64>;
}

#[async_trait::async_trait]
pub trait UpdatePriorityRequest: Send + Sync {
    /// Update all token events of the layer1
    async fn update_priority_request(&mut self) -> anyhow::Result<u64>;
}
