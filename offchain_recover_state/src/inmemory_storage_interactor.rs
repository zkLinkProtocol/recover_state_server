use std::cmp::max;
use std::collections::HashMap;
use ethers::prelude::H256;
use zklink_types::block::Block;
use zklink_types::{Account, AccountId, AccountMap, AccountUpdate, Action, BlockNumber, ChainId, Operation, Token, TokenId, ZkLinkAddress};
use zklink_types::utils::calculate_actual_token;

use crate::{
    data_restore_driver::StorageUpdateState,
    events::{BlockEvent, EventType},
    events_state::RollUpEvents,
    rollup_ops::RollupOpsBlock,
    storage_interactor::StorageInteractor,
    storage_interactor::StoredTreeState,
};
use crate::contract::utils::NewToken;

pub struct InMemoryStorageInteractor {
    rollups: Vec<RollupOpsBlock>,
    storage_state: StorageUpdateState,
    tokens: HashMap<(TokenId, ChainId), Token>,
    events_state: Vec<BlockEvent>,
    last_watched_block: u64,
    #[allow(dead_code)]
    last_committed_block: BlockNumber,
    last_verified_block: BlockNumber,
    accounts: AccountMap,
}

impl Default for InMemoryStorageInteractor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl StorageInteractor for InMemoryStorageInteractor {
    async fn load_tokens(&mut self) -> HashMap<TokenId, Token> {
        todo!()
    }

    async fn store_tokens(&mut self, tokens: &[NewToken], chain_id: ChainId) {
        for token in tokens{
            let token = Token {
                id: token.id.into(),
                chains: vec![chain_id],
            };
            self.tokens.insert((token.id, chain_id), token);
        }

    }

    async fn save_rollup_ops(&mut self, blocks: &[RollupOpsBlock]) {
        self.rollups = blocks.to_vec();
        self.storage_state = StorageUpdateState::Operations
    }

    async fn update_tree_state(&mut self, block: Block, accounts_updated: &[(AccountId, AccountUpdate, H256)]){
        let commit_op = Operation {
            action: Action::Commit,
            block: block.clone(),
            id: None,
        };

        let verify_op = Operation {
            action: Action::Verify {
                proof: Box::new(Default::default()),
            },
            block: block.clone(),
            id: None,
        };

        self.last_committed_block = commit_op.block.block_number;
        self.last_verified_block = verify_op.block.block_number;

        self.commit_state_update(*block.block_number, accounts_updated);
        self.storage_state = StorageUpdateState::None
        // TODO save operations
    }

    async fn init_token_event_progress(&mut self, _chain_id: ChainId, _last_block_number: BlockNumber) {
        todo!()
    }

    async fn update_token_event_progress(&mut self, _chain_id: ChainId, _last_watched_block_number: u64) {
        todo!()
    }

    async fn init_block_events_state(&mut self, _chain_id: ChainId, _last_watched_block_number: u64) {
        todo!()
    }

    async fn update_block_events_state(
        &mut self,
        _chain_id: ChainId,
        block_events: &[BlockEvent],
        last_watched_block_number: u64,
    ) {
        self.events_state = block_events.to_vec();
        self.last_watched_block = last_watched_block_number;
        self.storage_state = StorageUpdateState::Events;
    }

    async fn save_genesis_tree_state(&mut self, genesis_updates: &[(AccountId, AccountUpdate, H256)]) {
        self.commit_state_update(0, genesis_updates);
    }

    async fn get_block_events_state_from_storage(&mut self, _chain_id: ChainId) -> RollUpEvents {
        let committed_events = self.load_committed_events_state();

        let verified_events = self.load_verified_events_state();

        RollUpEvents {
            committed_events,
            verified_events,
            last_watched_block_number: self.last_watched_block,
        }
    }

    async fn get_tree_state(&mut self) -> StoredTreeState {
        // TODO find a way how to get unprocessed_prior_ops and fee_acc_id
        StoredTreeState {
            last_block_number: self.last_verified_block,
            account_map: self.accounts.clone(),
            fee_acc_id: AccountId(0),
        }
    }

    async fn get_ops_blocks_from_storage(&mut self) -> Vec<RollupOpsBlock> {
        self.rollups.clone()
    }

    async fn get_storage_state(&mut self) -> StorageUpdateState {
        self.storage_state
    }
}

impl InMemoryStorageInteractor {
    pub fn new() -> Self {
        Self {
            rollups: vec![],
            storage_state: StorageUpdateState::None,
            tokens: Default::default(),
            events_state: vec![],
            last_watched_block: 0,
            last_committed_block: BlockNumber(0),
            last_verified_block: BlockNumber(0),
            accounts: Default::default(),
        }
    }

    pub fn insert_new_account(&mut self, id: AccountId, address: &ZkLinkAddress) {
        self.accounts
            .insert(id, Account::default_with_address(address));
    }

    pub fn get_account_by_address(&self, address: &ZkLinkAddress) -> Option<(AccountId, Account)> {
        let accounts: Vec<(AccountId, Account)> = self
            .accounts
            .iter()
            .filter(|(_, acc)| acc.address == *address)
            .map(|(acc_id, acc)| (*acc_id, acc.clone()))
            .collect();
        accounts.first().cloned()
    }

    fn load_verified_events_state(&self) -> Vec<BlockEvent> {
        self.events_state
            .clone()
            .into_iter()
            .filter(|event| event.block_type == EventType::Verified)
            .collect()
    }

    pub(crate) fn load_committed_events_state(&self) -> Vec<BlockEvent> {
        // TODO avoid clone
        self.events_state
            .clone()
            .into_iter()
            .filter(|event| event.block_type == EventType::Committed)
            .collect()
    }

    pub fn get_account(&self, id: &AccountId) -> Option<&Account> {
        self.accounts.get(id)
    }

    fn commit_state_update(
        &mut self,
        first_update_order_id: u32,
        accounts_updated: &[(AccountId, AccountUpdate, H256)]
    ) {
        let update_order_ids =
            first_update_order_id..first_update_order_id + accounts_updated.len() as u32;

        for (_, (id, upd, _hash)) in update_order_ids.zip(accounts_updated.iter()) {
            match upd {
                AccountUpdate::Create { ref address, nonce } => {
                    let (mut acc, _) = Account::create_account(*id, address.clone());
                    acc.nonce = *nonce;
                    self.accounts.insert(*id, acc);
                }
                AccountUpdate::Delete {
                    ref address,
                    nonce: _,
                } => {
                    let (acc_id, _) = self.get_account_by_address(address).unwrap();
                    self.accounts.remove(&acc_id);
                }
                AccountUpdate::UpdateBalance {
                    balance_update: (token, sub_account_id, _, new_balance),
                    old_nonce: _,
                    new_nonce,
                } => {
                    let account = self
                        .accounts
                        .get_mut(id)
                        .expect("In tests this account should be stored");
                    let real_token = calculate_actual_token(*sub_account_id, *token);
                    account.set_balance(real_token, new_balance.clone());
                    account.nonce = max(account.nonce, *new_nonce);
                }
                AccountUpdate::ChangePubKeyHash {
                    old_pub_key_hash: _,
                    ref new_pub_key_hash,
                    old_nonce: _,
                    new_nonce,
                } => {
                    let account = self
                        .accounts
                        .get_mut(id)
                        .expect("In tests this account should be stored");
                    account.nonce = max(account.nonce, *new_nonce);
                    account.pub_key_hash = *new_pub_key_hash;
                }
                AccountUpdate::UpdateTidyOrder { .. } => {}
            }
        }
    }
}
