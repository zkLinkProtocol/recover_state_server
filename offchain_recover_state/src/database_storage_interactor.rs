use std::collections::HashMap;
// Built-in deps
use chrono::{DateTime, NaiveDateTime, Utc};
use ethers::prelude::H256;
use num::One;
// Workspace deps
use zklink_storage::{recover_state::records::{NewRollupOpsBlock}, StorageProcessor};
use zklink_crypto::convert::FeConvert;
use zklink_crypto::params::{USD_SYMBOL, USD_TOKEN_ID};
use zklink_types::{AccountId, {block::Block, AccountUpdate}, ChainId, TokenId, Token, BlockNumber};
use zklink_storage::chain::operations::records::{AggType, StoredAggregatedOperation};
use zklink_storage::tokens::records::{DbToken, DbTokenOfChain};
// Local deps
use crate::storage_interactor::StoredTreeState;
use crate::{
    data_restore_driver::StorageUpdateState,
    events::BlockEvent,
    events_state::RollUpEvents,
    rollup_ops::RollupOpsBlock,
    storage_interactor::{
        block_event_into_stored_block_event, stored_block_event_into_block_event,
        stored_ops_block_into_ops_block, StorageInteractor,
    },
};
use crate::contract::utils::NewToken;

pub struct DatabaseStorageInteractor<'a> {
    storage: StorageProcessor<'a>,
}

impl<'a> DatabaseStorageInteractor<'a> {
    pub fn new(storage: StorageProcessor<'a>) -> Self {
        Self { storage }
    }

    pub fn storage(&mut self) -> &mut StorageProcessor<'a> {
        &mut self.storage
    }
}

#[async_trait::async_trait]
impl StorageInteractor for DatabaseStorageInteractor<'_> {
    async fn load_tokens(&mut self) -> HashMap<TokenId, Token>{
        self.storage
            .tokens_schema()
            .load_tokens_from_db()
            .await
            .expect("reload token from db failed")
    }

    async fn store_tokens(&mut self, tokens: &[NewToken], chain_id: ChainId) {
        for token in tokens.iter(){
            let db_token = DbToken{
                token_id: token.id as i32,
                symbol: "".to_string(),
                price_id: "".to_string(),
                usd_price: Default::default(),
                last_update_time: Default::default(),
            };
            self.storage
                .tokens_schema()
                .store_token_price(db_token)
                .await
                .expect("failed to store token");
        }
        let tokens = tokens.iter()
            .map(|token_event|
                DbTokenOfChain {
                    id: token_event.id as i32,
                    chain_id: *chain_id as i16,
                    address: token_event.address.as_bytes().to_vec(),
                    decimals: 18,
                    fast_withdraw: false
                }
            )
            .collect();
        self.storage
            .tokens_schema()
            .save_tokens(tokens)
            .await
            .expect("failed to store token");
    }

    async fn save_rollup_ops(&mut self, blocks: &[RollupOpsBlock]) {
        let mut ops = Vec::with_capacity(blocks.len());

        for block in blocks {
            let timestamp = block.timestamp.map(|timestamp|
                DateTime::from_utc(NaiveDateTime::from_timestamp_millis(timestamp as i64).unwrap(), Utc)
            );

            ops.push(NewRollupOpsBlock {
                block_num: block.block_num,
                ops: block.ops.as_slice(),
                fee_account: block.fee_account,
                timestamp,
                previous_block_root_hash: block.previous_block_root_hash,
                contract_version: block.contract_version.unwrap().into(),
            });
        }

        self.storage
            .recover_schema()
            .save_rollup_ops(ops.as_slice())
            .await
            .expect("Cant update rollup operations");
    }

    async fn store_blocks_and_updates(&mut self, blocks_and_updates: Vec<(Block, Vec<(AccountId, AccountUpdate, H256)>)>) {
        let new_state = self.storage.recover_schema().new_storage_state("None");
        let mut transaction = self
            .storage
            .start_transaction()
            .await
            .expect("Failed initializing a DB transaction");
        for (block, accounts_updated) in blocks_and_updates {
            let block_number = *block.block_number;
            let commit_aggregated_operation = StoredAggregatedOperation {
                id: 0,
                action_type: AggType::CommitBlocks,
                from_block: block_number.into(),
                to_block: block_number.into(),
                created_at: Utc::now(),
                confirmed: true
            };
            let execute_aggregated_operation = StoredAggregatedOperation {
                id: 0,
                action_type: AggType::ExecuteBlocks,
                from_block: block_number.into(),
                to_block: block_number.into(),
                created_at: Utc::now(),
                confirmed: true
            };

            transaction
                .chain()
                .state_schema()
                .commit_state_update(block.block_number, &accounts_updated)
                .await
                .expect("Cant execute verify operation");

            transaction
                .recover_schema()
                .save_block_operations(&commit_aggregated_operation, &execute_aggregated_operation)
                .await
                .expect("Cant execute verify operation");

            transaction
                .chain()
                .block_schema()
                .save_block(block)
                .await
                .expect("Unable to save block");
        }
        transaction
            .recover_schema()
            .update_storage_state(new_state)
            .await
            .expect("Unable to update storage state");
        transaction
            .commit()
            .await
            .expect("Unable to commit DB transaction");
    }

    async fn init_token_event_progress(&mut self, chain_id: ChainId, last_block_number: BlockNumber) {
        self.storage
            .recover_schema()
            .insert_last_watched_block_number(*chain_id as i16, "token", *last_block_number as i64)
            .await
            .expect("failed to initialize last watched block number");
    }

    async fn update_token_event_progress(
        &mut self,
        chain_id: ChainId,
        last_watched_block_number: u64,
    ) {
        // add USD token to token_price table
        self.storage.tokens_schema()
            .store_token_price(DbToken {
                token_id: USD_TOKEN_ID as i32,
                symbol: String::from(USD_SYMBOL),
                price_id: "".to_string(),
                usd_price: One::one(),
                last_update_time: Utc::now(),
            })
            .await
            .expect("failed to add USD token");

        self.storage
            .recover_schema()
            .update_last_watched_block_number(*chain_id as i16, "token", last_watched_block_number as i64)
            .await
            .expect("failed to update last_watched_block_number");
    }

    async fn init_block_events_state(
        &mut self,
        chain_id: ChainId,
        last_watched_block_number: u64,
    ) {
        self.storage
            .recover_schema()
            .insert_block_events_state(chain_id, last_watched_block_number)
            .await
            .expect("Cant update events state");
    }

    async fn update_block_events_state(
        &mut self,
        chain_id: ChainId,
        block_events: &[BlockEvent],
        last_watched_block_number: u64,
    ) -> anyhow::Result<()> {
        let block_events= block_events
            .iter()
            .map(block_event_into_stored_block_event)
            .collect::<Vec<_>>();

        self.storage
            .recover_schema()
            .update_block_events_state(chain_id, &block_events, last_watched_block_number)
            .await?;
        Ok(())
    }

    async fn save_genesis_tree_state(&mut self, genesis_updates: &[(AccountId, AccountUpdate, H256)]) {
        let root_hash = FeConvert::from_bytes(genesis_updates.first().unwrap().2.as_bytes()).unwrap();
        let (last_committed, accounts) = self
            .storage
            .chain()
            .state_schema()
            .load_committed_state(None)
            .await
            .expect("Cant load committed state");
        assert!(
            last_committed == 0 && accounts.is_empty(),
            "db should be empty"
        );
        self.storage
            .recover_schema()
            .save_genesis_state(genesis_updates)
            .await
            .expect("Cant update genesis state");
        self.storage
            .chain()
            .block_schema()
            .save_genesis_block(root_hash)
            .await
            .expect("Cant update genesis block");
    }

    async fn get_block_events_state_from_storage(&mut self, chain_id: ChainId) -> RollUpEvents {
        let last_watched_eth_block_number = self
            .storage
            .recover_schema()
            .last_watched_block_number(*chain_id as i16, "block")
            .await
            .expect("Cant load last watched block number")
            .unwrap() as u64;

        let committed = self
            .storage
            .recover_schema()
            .load_committed_events_state()
            .await
            .expect("Cant load committed state");

        let mut committed_events: Vec<BlockEvent> = vec![];
        for event in committed {
            let block_event = stored_block_event_into_block_event(event.clone());
            committed_events.push(block_event);
        }

        let verified = self
            .storage
            .recover_schema()
            .load_verified_events_state()
            .await
            .expect("Cant load verified state");
        let mut verified_events: Vec<BlockEvent> = vec![];
        for event in verified {
            let block_event = stored_block_event_into_block_event(event.clone());
            verified_events.push(block_event);
        }

        RollUpEvents {
            committed_events,
            verified_events,
            last_watched_block_number: last_watched_eth_block_number,
        }
    }

    async fn get_tree_state(&mut self) -> StoredTreeState {
        let (last_block, account_map) = self
            .storage
            .chain()
            .state_schema()
            .load_last_state()
            .await
            .expect("There are no last verified state in storage");

        let block = self
            .storage
            .chain()
            .block_schema()
            .get_block(last_block)
            .await
            .expect("Cant get the last block from storage")
            .expect("There are no last block in storage - restart driver");

        StoredTreeState {
            last_block_number: last_block.into(),
            account_map,
            fee_acc_id: block.fee_account,
        }
    }

    async fn get_ops_blocks_from_storage(&mut self) -> Vec<RollupOpsBlock> {
        self.storage
            .recover_schema()
            .load_rollup_ops_blocks()
            .await
            .expect("Cant load operation blocks")
            .into_iter()
            .map(stored_ops_block_into_ops_block)
            .collect()
    }

    async fn get_storage_state(&mut self) -> StorageUpdateState {
        let storage_state_string = self
            .storage
            .recover_schema()
            .load_storage_state()
            .await
            .expect("Cant load storage state")
            .storage_state;

        match storage_state_string.as_ref() {
            "Events" => StorageUpdateState::Events,
            "Operations" => StorageUpdateState::Operations,
            "None" => StorageUpdateState::None,
            _ => panic!("Unknown storage state"),
        }
    }
}
