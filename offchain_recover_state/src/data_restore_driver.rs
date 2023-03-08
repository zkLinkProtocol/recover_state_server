// External deps
// Workspace deps
use zklink_crypto::Fr;
use zklink_types::{Account, AccountId, AccountMap, AccountUpdate, BlockNumber, ChainId, H256, Token, ZkLinkAddress};

// Local deps
use crate::{error, events_state::RollUpEvents, rollup_ops::RollupOpsBlock, storage_interactor::StorageInteractor, tree_state::TreeState, VIEW_BLOCKS_STEP};

use std::marker::PhantomData;
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{debug, info};
use zklink_crypto::convert::FeConvert;
use zklink_crypto::params::{FEE_ACCOUNT_ID, GLOBAL_ASSET_ACCOUNT_ADDR, GLOBAL_ASSET_ACCOUNT_ID};
use recover_state_config::{ChainType, RecoverStateConfig};
use zklink_storage::ConnectionPool;
use crate::contract::update_token_events::{EvmTokenEvents, UpdateTokenEvents};
use crate::contract::ZkLinkContract;

/// Storage state update:
/// - None - The state is updated completely last time - start from fetching the new events
/// - Events - The events fetched and saved successfully - now get operations from them and update tree
/// - Operations - There are operations that are not presented in the tree state - update tree state
#[derive(Debug, Copy, Clone)]
pub enum StorageUpdateState {
    None,
    Events,
    Operations,
}

/// Recover state driver is a high level interface for all restoring components.
/// It is actually a finite state machine, that has following states:
/// - Empty - The state is new
/// - None - The state is completely updated last time, driver will load state from storage and fetch new events
/// - Events - The events has been fetched and saved successfully and firstly driver will load state from storage
///   and get new operation for last saved events
/// - Operations - The operations and events has been fetched and saved successfully and firstly driver will load
///   state from storage and update merkle tree by last saved operations
///
/// Driver can interact with other restoring components for their updating:
/// - Token Events
/// - Block Events
/// - Operations
/// - Tree
/// - Storage
pub struct RecoverStateDriver<I: StorageInteractor, T: ZkLinkContract> {
    /// Update the token events of all chains.
    pub update_token_events: Vec<(ChainId, Box<dyn UpdateTokenEvents>)>,
    /// Provides uncompressed(upload all pubdata) layer1 rollup contract interface.
    pub zklink_contract: T,
    /// Layer1 blocks heights that include correct UpgradeComplete events.
    /// Should be provided via config.
    pub contract_upgraded_blocks: Vec<u64>,
    /// The initial version of the deployed zkLink contract.
    pub init_contract_version: u32,
    /// Rollup contract events state
    pub rollup_events: RollUpEvents,
    /// Rollup accounts state
    pub tree_state: TreeState,
    /// The step distance of viewing events in the layer1 blocks
    pub view_blocks_step: u64,
    /// The distance to the last layer1 block
    pub end_block_offset: u64,
    /// Finite mode flag. In finite mode, driver will only work until
    /// amount of restored blocks will become equal to amount of known
    /// verified blocks. After that, it will stop.
    pub finite_mode: bool,
    /// Expected root hash to be observed after restoring process. Only
    /// available in finite mode, and intended for tests.
    pub final_hash: Option<Fr>,
    phantom_data: PhantomData<I>,
}

impl<T, I> RecoverStateDriver<I, T>
where
    T: ZkLinkContract,
    I: StorageInteractor,
{
    /// Returns new data restore driver with empty events and tree states.
    ///
    /// # Arguments
    ///
    /// * `zklink_contract` - Current deployed zklink contract
    /// * `config` - the config that RecoverState need.
    /// * `view_blocks_step` - The step distance of viewing events in the layer1 blocks
    /// * `end_block_offset` - The distance to the last layer1 block
    /// * `finite_mode` - Finite mode flag.
    /// * `final_hash` - Hash of the last block which we want to restore
    /// * `deploy_block_number` - the block number of deployed zklink contract
    /// * `connection_pool` - the connection pool of database
    ///
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        zklink_contract: T,
        config: &RecoverStateConfig,
        view_blocks_step: u64,
        end_block_offset: u64,
        finite_mode: bool,
        final_hash: Option<Fr>,
        deploy_block_number: u64,
        connection_pool: ConnectionPool,
    ) -> Self {
        let mut storage = connection_pool.access_storage().await.unwrap();
        let mut tree_state = TreeState::new();
        tree_state.state.token_by_id = storage.tokens_schema()
            .load_tokens_from_db()
            .await
            .expect("reload token from db failed");

        let mut events_state = RollUpEvents::default();
        events_state.last_watched_block_number = storage.recover_schema()
            .last_watched_block_number(*zklink_contract.layer2_chain_id() as i16, "block")
            .await
            .expect("load last watched block number failed")
            .map(|num|num as u64)
            .unwrap_or(deploy_block_number);

        let mut update_token_events = Vec::with_capacity(config.layer1.chain_configs.len());
        for config in &config.layer1.chain_configs {
            let token_events: Box<dyn UpdateTokenEvents> = match config.chain.chain_type{
                ChainType::EVM => Box::new(EvmTokenEvents::new(config,connection_pool.clone()).await),
                ChainType::STARKNET => panic!("supported chain type.")
            };
            update_token_events.push((config.chain.chain_id, token_events))
        }
        Self {
            update_token_events,
            contract_upgraded_blocks: config.upgrade_eth_blocks.clone(),
            init_contract_version: config.init_contract_version,
            zklink_contract,
            rollup_events: events_state,
            tree_state,
            view_blocks_step,
            end_block_offset,
            finite_mode,
            final_hash,
            phantom_data: Default::default(),
        }
    }

    pub async fn download_registered_tokens(&mut self, token_sender: Sender<Token>) {
        let cur_block_number = self.zklink_contract
            .block_number()
            .await
            .expect("Failed to get current block number");
        while let Some((chain_id, mut updating_event)) = self.update_token_events.pop(){
            tokio::spawn(async move {
                loop {
                    match updating_event.update_token_events(token_sender.clone()).await
                    {
                        Ok(last_sync_block_number) =>
                            if last_sync_block_number + VIEW_BLOCKS_STEP > cur_block_number {
                                info!("The update token events of {:?} client has completed!", chain_id);
                                break
                            } else {
                                info!("updating token events to block number:{}", last_sync_block_number);
                                continue
                            },
                        Err(err) => error!("Failed to update token events: {}", err)
                    }
                }
            });
        }
        info!("Let the updating token event run for a while, so that subsequent transactions can be executed.");
        tokio::time::sleep(Duration::from_secs(10)).await;
    }

    /// Sets the 'genesis' state.
    /// Tree with inserted genesis account will be created.
    /// Used when restore driver is restarted.
    ///
    /// # Arguments
    ///
    /// * `governance_contract_genesis_tx_hash` - Governance contract creation tx hash
    ///
    pub async fn set_genesis_state(&mut self, interactor: &mut I, config: RecoverStateConfig) {
        let full_pubdata_chain_config = config.layer1
            .chain_configs
            .iter()
            .find(|chain| !chain.chain.is_commit_compressed_blocks)
            .unwrap();
        let genesis_transaction = self.zklink_contract
            .get_transaction(full_pubdata_chain_config.contracts.genesis_tx_hash)
            .await
            .unwrap()
            .expect("Cant get zkLink genesis transaction");

        // Setting last watched block number for events state
        let last_watched_block_number = self
            .rollup_events
            .set_last_watched_block_number::<T>(&genesis_transaction)
            .expect("Cant set genesis block number for events state");
        info!("genesis_block_number: {:?}", &last_watched_block_number);

        let genesis_fee_account = self.zklink_contract
            .get_genesis_account(genesis_transaction)
            .expect("Cant get genesis account address");
        info!("genesis fee account address: {:?}", genesis_fee_account.address);

        // Init basic accounts.
        let mut account_map = AccountMap::default();
        // fee|validator account
        let fee_account = Account::default_with_address(&genesis_fee_account.address);
        let db_fee_account_update = AccountUpdate::Create {
            address: genesis_fee_account.address,
            nonce: fee_account.nonce,
        };
        account_map.insert(FEE_ACCOUNT_ID, fee_account);
        // black hole address, for global asset account
        let global_asset_account = Account::default_with_address(&GLOBAL_ASSET_ACCOUNT_ADDR.parse::<ZkLinkAddress>().unwrap());
        let db_global_account_update = AccountUpdate::Create {
            address: global_asset_account.address.clone(),
            nonce: global_asset_account.nonce,
        };
        account_map.insert(GLOBAL_ASSET_ACCOUNT_ID, global_asset_account);

        // Init state tree
        let tree_state = TreeState::load(
            BlockNumber(0),
            account_map,
            AccountId(0),
        );
        let state_root = tree_state.root_hash();
        info!("Genesis tree root: {:?}", state_root);
        debug!("Genesis accounts: {:?}", tree_state.get_accounts());
        let root_hash = H256::from_slice(&state_root.to_bytes());

        // init basic accounts updates
        let mut account_updates = Vec::with_capacity(2);
        account_updates.push((AccountId(0), db_fee_account_update, root_hash));
        account_updates.push((AccountId(1), db_global_account_update, root_hash));

        // Init last watched block number for database
        let chain_id = full_pubdata_chain_config.chain.chain_id;
        interactor.init_block_events_state(chain_id, last_watched_block_number).await;
        for chain_config in config.multi_chains_configs
            .chain_configs
            .iter()
        {
            interactor.init_token_event_progress(
                chain_config.chain.chain_id,
                chain_config.contracts.deployment_block.into()
            ).await;
        }
        // Init genesis tree state for database
        interactor.save_genesis_tree_state(&account_updates).await;

        info!("Saved genesis tree state\n");
        self.tree_state = tree_state;
    }

    /// Stops states from storage
    pub async fn load_state_from_storage(&mut self, interactor: &mut I) -> bool {
        info!("Loading state from storage");
        let state = interactor.get_storage_state().await;
        self.rollup_events = interactor.get_block_events_state_from_storage(
            self.zklink_contract.layer2_chain_id()
        ).await;
        let tree_state = interactor.get_tree_state().await;
        println!("account_map: {:?}", tree_state.account_map);
        self.tree_state = TreeState::load(
            tree_state.last_block_number,
            tree_state.account_map,
            tree_state.fee_acc_id,
        );
        let new_ops_blocks = match state {
            StorageUpdateState::Events => self.load_op_from_events_and_save_op(interactor).await,
            StorageUpdateState::Operations => interactor.get_ops_blocks_from_storage().await,
            StorageUpdateState::None => vec![]
        };
        self.update_tree_state(interactor, new_ops_blocks).await;

        let total_verified_blocks = self.zklink_contract.get_total_verified_blocks().await.unwrap();
        let last_verified_block = self.tree_state.state.block_number;
        info!(
            "State has been loaded\nProcessed {:?} blocks on contract\nRoot hash: {:?}\n",
            last_verified_block, self.tree_state.root_hash()
        );

        self.finite_mode && (total_verified_blocks == *last_verified_block)
    }

    /// Activates states updates
    pub async fn recover_state(&mut self, interactor: &mut I, mut token_receiver: Receiver<Token>) {
        let mut last_watched_block = self.rollup_events.last_watched_block_number;
        let mut final_hash_was_found = false;
        loop {
            info!("Last watched layer1 block: {:?}", last_watched_block);
            // Receive tokens generated by updating token events.
            while let Ok(token) = token_receiver.try_recv(){
                self.tree_state.state
                    .token_by_id
                    .entry(token.id)
                    .or_insert(token.clone())
                    .chains
                    .extend(token.chains);
            }

            // Update events
            if self.exist_events_state(interactor).await {
                // Update operations
                let new_ops_blocks = self.load_op_from_events_and_save_op(interactor).await;

                if !new_ops_blocks.is_empty() {
                    // Update tree
                    self.update_tree_state(interactor, new_ops_blocks).await;

                    let total_verified_blocks = self.zklink_contract.get_total_verified_blocks().await.unwrap();
                    let last_verified_block = self.tree_state.state.block_number;

                    // // We must update the Layer1 stats table to match the actual stored state
                    // // to keep the `state_keeper` consistent with the `sender`.
                    // interactor.update_eth_state().await;

                    info!(
                        "State updated\nProcessed {:?} blocks of total {:?} verified on contract\nRoot hash: {:?}\n",
                        last_verified_block, total_verified_blocks, self.tree_state.root_hash()
                    );

                    // If there is an expected root hash, check if current root hash matches the observed
                    // one.
                    // We check it after every block, since provided final hash may be not the latest hash
                    // by the time when it was processed.
                    if let Some(root_hash) = self.final_hash {
                        if root_hash == self.tree_state.root_hash() {
                            final_hash_was_found = true;
                            info!(
                                "Correct expected root hash was met on the block {} out of {}",
                                *last_verified_block, total_verified_blocks
                            );
                        }
                    }

                    if self.finite_mode && *last_verified_block == total_verified_blocks {
                        // Check if the final hash was found and panic otherwise.
                        if self.final_hash.is_some() && !final_hash_was_found {
                            panic!("Final hash was not met during the state restoring process");
                        }

                        // We've restored all the blocks, our job is done.
                        break;
                    }
                }
            }

            if last_watched_block == self.rollup_events.last_watched_block_number {
                info!("sleep block");
                tokio::time::sleep(Duration::from_secs(5)).await;
            } else {
                last_watched_block = self.rollup_events.last_watched_block_number;
            }
        }
    }

    /// Updates events state, saves new blocks, tokens events and the last watched block number in storage
    /// Returns bool flag, true if there are new block events
    async fn exist_events_state(&mut self, interactor: &mut I) -> bool {
        info!("Loading block events from zklink contract!");
        let (block_events, last_watched_eth_block_number) = self
            .rollup_events
            .update_block_events(
                &self.zklink_contract,
                &self.contract_upgraded_blocks,
                self.view_blocks_step,
                self.end_block_offset,
                self.init_contract_version,
            )
            .await
            .expect("Updating events state: cant update events state");
        interactor
            .update_block_events_state(
                self.zklink_contract.layer2_chain_id(),
                &block_events,
                last_watched_eth_block_number,
            )
            .await;
        info!("Updating block events: {:?}", block_events);
        info!("Updating block events to block_number: {}", last_watched_eth_block_number);

        !block_events.is_empty()
    }

    /// Updates tree state from the new Rollup operations blocks, saves it in storage
    ///
    /// # Arguments
    ///
    /// * `new_ops_blocks` - the new Rollup operations blocks
    ///
    async fn update_tree_state(&mut self, interactor: &mut I, new_ops_blocks: Vec<RollupOpsBlock>) {
        for op_block in new_ops_blocks {
            let (block, acc_updates) = self
                .tree_state
                .apply_ops_block(&op_block)
                .expect("Updating tree state: cant update tree from operations");
            interactor
                .update_tree_state(block, &acc_updates)
                .await;
        }

        debug!("Updated state");
    }

    /// Gets new operations blocks from events, updates rollup operations stored state.
    /// Returns new rollup operations blocks
    async fn load_op_from_events_and_save_op(&mut self, interactor: &mut I) -> Vec<RollupOpsBlock> {
        let new_blocks = self.get_new_operation_blocks_from_events().await;

        interactor.save_rollup_ops(&new_blocks).await;

        debug!("Updated operations storage");

        new_blocks
    }

    /// Returns verified committed operations blocks from verified op blocks events
    pub async fn get_new_operation_blocks_from_events(&mut self) -> Vec<RollupOpsBlock> {
        let mut blocks = Vec::new();

        let mut last_event_tx_hash = None;
        for event in self.rollup_events
            .get_only_verified_committed_events()
            .into_iter()
        {
            // We use an aggregated block in contracts, which means that several BlockEvent can include the same tx_hash,
            // but for correct restore we need to generate RollupBlocks from this tx only once.
            // These blocks go one after the other, and checking only the last transaction hash is safe
            if let Some(tx) = last_event_tx_hash {
                if tx == event.transaction_hash {
                    continue;
                }
            }

            let transaction_hash = event.transaction_hash;
            let rollup_blocks = RollupOpsBlock::get_rollup_ops_blocks(&self.zklink_contract, event)
                .await
                .expect("Cant get new operation blocks from events");

            blocks.extend(rollup_blocks);
            last_event_tx_hash = Some(transaction_hash);
        }

        blocks
    }
}
