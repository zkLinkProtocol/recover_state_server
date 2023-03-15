pub mod evm;

pub use evm::EvmTokenEvents;

#[async_trait::async_trait]
pub trait UpdateTokenEvents: Send + Sync {
    fn reached_latest_block(&self, newer_block: u64) -> bool;
    async fn block_number(&self) -> anyhow::Result<u64>;
    async fn update_token_events(&mut self) -> anyhow::Result<u64>;
}
