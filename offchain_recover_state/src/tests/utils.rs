use crate::contract::ZkLinkContract;
use anyhow::Error;
use async_trait::async_trait;
use ethers::prelude::{Bytes, Log};
use std::future::Future;
use zklink_types::{Account, BlockNumber, ChainId, H160, H256};

// #[derive(Debug, Clone)]
// pub(crate) struct FakeContract;
//
// #[async_trait]
// impl ZkLinkContract for FakeContract {
//     type Log = ();
//     type Transaction = ();
//
//     fn layer2_chain_id(&self) -> ChainId {
//         todo!()
//     }
//
//     fn get_event_signature(&self, name: &str) -> H256 {
//         todo!()
//     }
//
//     fn get_genesis_account(&self, genesis_tx: Self::Transaction) -> anyhow::Result<Account> {
//         todo!()
//     }
//
//     async fn get_transaction(&self, hash: H256) -> anyhow::Result<Option<Self::Transaction>> {
//         todo!()
//     }
//
//     async fn get_total_verified_blocks(&self) -> anyhow::Result<u32> {
//         todo!()
//     }
//
//     async fn get_block_logs(&self, from: BlockNumber, to: BlockNumber) -> Result<Vec<Self::Log>, Error> {
//         todo!()
//     }
//
//     async fn get_gatekeeper_logs(&self) -> anyhow::Result<Vec<Self::Log>> {
//         todo!()
//     }
//
//     async fn block_number(&self) -> anyhow::Result<u64> {
//         todo!()
//     }
// }

pub(crate) fn u32_to_32bytes(value: u32) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    let bytes_value = value.to_be_bytes();
    // Change only the last 4 bytes, which are represent u32
    bytes[28..32].clone_from_slice(&bytes_value);
    bytes
}

pub(crate) fn create_log(
    address: H160,
    topic: H256,
    additional_topics: Vec<H256>,
    data: Bytes,
    block_number: u32,
    transaction_hash: H256,
) -> Log {
    let mut topics = vec![topic];
    topics.extend(additional_topics);
    Log {
        address,
        topics,
        data,
        block_hash: None,
        block_number: Some(block_number.into()),
        transaction_hash: Some(transaction_hash),
        transaction_index: Some(0.into()),
        log_index: Some(0.into()),
        transaction_log_index: Some(0.into()),
        log_type: Some("mined".into()),
        removed: None,
    }
}
