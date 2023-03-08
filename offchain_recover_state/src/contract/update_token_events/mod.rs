pub mod evm;

use zklink_types::Token;
use tokio::sync::mpsc::Sender;
pub use evm::EvmTokenEvents;

#[async_trait::async_trait]
pub trait UpdateTokenEvents: Send + Sync {
    async fn update_token_events(&mut self, , token_sender: Sender<Token>) -> anyhow::Result<u64>;
}
