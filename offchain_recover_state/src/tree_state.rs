use crate::rollup_ops::RollupOpsBlock;
use anyhow::format_err;
use std::collections::HashMap;
use tracing::info;
use zklink_crypto::convert::FeConvert;
use zklink_crypto::Fr;
use zklink_state::state::TransferOutcome;
use zklink_state::{
    handler::TxHandler,
    state::{OpSuccess, ZkLinkState},
};
use zklink_types::block::{Block, ExecutedTx};
use zklink_types::operations::ZkLinkOp;
use zklink_types::{
    Account, AccountId, AccountMap, AccountUpdate, BlockNumber, ChainId, ChangePubKey, Deposit,
    ForcedExit, FullExit, OrderMatching, Transfer, Withdraw, ZkLinkAddress, H256,
};

type BlockAndUpdates = (Block, Vec<(AccountId, AccountUpdate, H256)>);

/// Rollup accounts states
pub struct TreeState {
    /// Accounts stored in a spase merkle tree
    pub state: ZkLinkState,
    /// The last fee account address
    pub last_fee_account_address: ZkLinkAddress,
    /// the current serial id of priority op of all chain.
    pub last_serial_ids: HashMap<ChainId, i64>,
}

impl Default for TreeState {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeState {
    /// Returns empty self state
    pub fn new() -> Self {
        Self {
            state: ZkLinkState::empty(),
            last_fee_account_address: ZkLinkAddress::default(),
            last_serial_ids: HashMap::new(),
        }
    }

    /// Returns the loaded state
    ///
    /// # Arguments
    ///
    /// * `current_block` - The current block number
    /// * `last_serial_ids` - the current serial id of priority op of all chain.
    /// * `accounts` - Accounts stored in a spase merkle tree
    /// * `fee_account` - The last fee account address
    ///
    pub fn load(
        current_block: BlockNumber,
        last_serial_ids: HashMap<ChainId, i64>,
        accounts: AccountMap,
        fee_account: AccountId,
    ) -> Self {
        let state = ZkLinkState::from_acc_map(accounts, current_block);
        let last_fee_account_address = state
            .get_account(fee_account)
            .expect("Cant get fee account from tree state")
            .address;
        Self {
            state,
            last_fee_account_address,
            last_serial_ids,
        }
    }

    /// Updates Rollup accounts states from Rollup operations block
    /// Returns current rollup block and updated accounts
    pub fn apply_ops_block(
        &mut self,
        ops_block: &RollupOpsBlock,
    ) -> Result<BlockAndUpdates, anyhow::Error> {
        info!("Applying ops_block[{:?}]", ops_block.block_num);
        assert_eq!(self.state.block_number + 1, ops_block.block_num);
        assert_eq!(
            ops_block.previous_block_root_hash,
            H256::from_slice(&self.root_hash().to_bytes()),
            "There was an error in processing the last block[{:?}] ",
            ops_block.block_num - 1
        );
        let operations = ops_block.ops.clone();

        let mut accounts_updated = Vec::new();
        let mut ops = Vec::new();
        let mut current_op_block_index = 0u32;

        for (index, operation) in operations.into_iter().enumerate() {
            match operation {
                ZkLinkOp::Deposit(op) => {
                    let mut op = <ZkLinkState as TxHandler<Deposit>>::create_op(&self.state, op.tx)
                        .map_err(|e| format_err!("Create Deposit fail: {}", e))?;
                    let updates =
                        <ZkLinkState as TxHandler<Deposit>>::apply_op(&mut self.state, &mut op)
                            .map_err(|e| format_err!("Apply Deposit fail: {}", e))?;
                    let tx_result = OpSuccess {
                        updates,
                        executed_op: op.into(),
                    };

                    current_op_block_index = self.update_from_tx(
                        tx_result,
                        &mut accounts_updated,
                        current_op_block_index,
                        &mut ops,
                    );
                }
                ZkLinkOp::TransferToNew(mut op) => {
                    let from = self
                        .state
                        .get_account(op.from)
                        .ok_or_else(|| format_err!("TransferToNew fail: Nonexistent account"))?;
                    op.tx.nonce = from.nonce;

                    let mut op = TransferOutcome::TransferToNew(*op);
                    let updates =
                        <ZkLinkState as TxHandler<Transfer>>::apply_op(&mut self.state, &mut op)
                            .map_err(|e| format_err!("TransferToNew fail: {}", e))?;
                    let tx_result = OpSuccess {
                        updates,
                        executed_op: op.into_franklin_op(),
                    };

                    current_op_block_index = self.update_from_tx(
                        tx_result,
                        &mut accounts_updated,
                        current_op_block_index,
                        &mut ops,
                    );
                }
                ZkLinkOp::Transfer(mut op) => {
                    let from = self
                        .state
                        .get_account(op.from)
                        .ok_or_else(|| format_err!("Transfer Fail: Nonexistent account"))?;
                    let to = self
                        .state
                        .get_account(op.to)
                        .ok_or_else(|| format_err!("Transfer Fail: Nonexistent account"))?;
                    // op.tx.from = from.address;
                    op.tx.to = to.address;
                    op.tx.nonce = from.nonce;

                    let mut op = TransferOutcome::Transfer(*op);
                    let updates =
                        <ZkLinkState as TxHandler<Transfer>>::apply_op(&mut self.state, &mut op)
                            .map_err(|e| format_err!("Transfer fail: {}", e))?;
                    let tx_result = OpSuccess {
                        updates,
                        executed_op: op.into_franklin_op(),
                    };

                    current_op_block_index = self.update_from_tx(
                        tx_result,
                        &mut accounts_updated,
                        current_op_block_index,
                        &mut ops,
                    );
                }
                ZkLinkOp::Withdraw(mut op) => {
                    // Withdraw op comes with empty Account ZkLinkAddress and Nonce fields
                    let account = self
                        .state
                        .get_account(op.account_id)
                        .ok_or_else(|| format_err!("Withdraw fail: Nonexistent account"))?;
                    op.tx.nonce = account.nonce;

                    let updates =
                        <ZkLinkState as TxHandler<Withdraw>>::apply_op(&mut self.state, &mut op)
                            .map_err(|e| format_err!("Withdraw fail: {}", e))?;
                    let tx_result = OpSuccess {
                        updates,
                        executed_op: (*op).into(),
                    };

                    current_op_block_index = self.update_from_tx(
                        tx_result,
                        &mut accounts_updated,
                        current_op_block_index,
                        &mut ops,
                    );
                }
                ZkLinkOp::ForcedExit(mut op) => {
                    // Withdraw op comes with empty Account ZkLinkAddress and Nonce fields
                    let initiator_account = self
                        .state
                        .get_account(op.tx.initiator_account_id)
                        .ok_or_else(|| {
                            format_err!("ForcedExit fail: Nonexistent initiator account")
                        })?;

                    // Set the fields unknown from the pubdata.
                    op.tx.nonce = initiator_account.nonce;

                    let updates =
                        <ZkLinkState as TxHandler<ForcedExit>>::apply_op(&mut self.state, &mut op)
                            .map_err(|e| format_err!("ForcedExit fail: {}", e))?;
                    let tx_result = OpSuccess {
                        updates,
                        executed_op: (*op).into(),
                    };

                    current_op_block_index = self.update_from_tx(
                        tx_result,
                        &mut accounts_updated,
                        current_op_block_index,
                        &mut ops,
                    );
                }
                ZkLinkOp::FullExit(op) => {
                    let mut op =
                        <ZkLinkState as TxHandler<FullExit>>::create_op(&self.state, op.tx)
                            .map_err(|e| format_err!("Create FullExit fail: {}", e))?;
                    let updates =
                        <ZkLinkState as TxHandler<FullExit>>::apply_op(&mut self.state, &mut op)
                            .map_err(|e| format_err!("FullExit fail: {}", e))?;
                    let tx_result = OpSuccess {
                        updates,
                        executed_op: op.into(),
                    };

                    current_op_block_index = self.update_from_tx(
                        tx_result,
                        &mut accounts_updated,
                        current_op_block_index,
                        &mut ops,
                    );
                }
                ZkLinkOp::ChangePubKeyOffchain(mut op) => {
                    let account = self.state.get_account(op.account_id).ok_or_else(|| {
                        format_err!("ChangePubKeyOffChain fail: Nonexistent account")
                    })?;
                    op.tx.nonce = account.nonce;

                    let updates = <ZkLinkState as TxHandler<ChangePubKey>>::apply_op(
                        &mut self.state,
                        &mut op,
                    )
                    .map_err(|e| format_err!("ChangePubKeyOffchain fail: {}", e))?;
                    let tx_result = OpSuccess {
                        updates,
                        executed_op: (*op).into(),
                    };

                    current_op_block_index = self.update_from_tx(
                        tx_result,
                        &mut accounts_updated,
                        current_op_block_index,
                        &mut ops,
                    );
                }
                ZkLinkOp::OrderMatching(mut op) => {
                    let updates = <ZkLinkState as TxHandler<OrderMatching>>::unsafe_apply_op(
                        &mut self.state,
                        &mut op,
                    )
                    .map_err(|e| format_err!("OrderMatching fail: {}", e))?;
                    if ops_block.block_num == 1770.into() {
                        info!("block_index: {}, \nupdates :{:?}", index, updates);
                    }
                    let tx_result = OpSuccess {
                        updates,
                        executed_op: (*op).into(),
                    };

                    current_op_block_index = self.update_from_tx(
                        tx_result,
                        &mut accounts_updated,
                        current_op_block_index,
                        &mut ops,
                    );
                }
                ZkLinkOp::Noop(_) => {}
            }
        }

        let fee_account_address = self
            .get_account(ops_block.fee_account)
            .ok_or_else(|| format_err!("Nonexistent fee account"))?
            .address;

        self.last_fee_account_address = fee_account_address;

        // As we restoring an already executed block, this value isn't important.
        let gas_limit = 0.into();

        // Take the contract version into account when choosing block chunk sizes.
        let available_block_chunk_sizes = ops_block
            .contract_version
            .expect("contract version must be set")
            .available_block_chunk_sizes();
        let support_ops_numbers = ops_block
            .contract_version
            .expect("contract version must be set")
            .supported_ops_numbers();
        let available_chain_ids = ops_block
            .contract_version
            .expect("contract version must be set")
            .available_chain_ids();

        let block = Block::new_from_available_block_sizes(
            ops_block.block_num,
            self.root_hash(),
            ops_block.fee_account,
            ops,
            *available_block_chunk_sizes.first().unwrap(),
            support_ops_numbers,
            available_chain_ids,
            gas_limit,
            gas_limit,
            ops_block.previous_block_root_hash,
            ops_block.timestamp.unwrap_or_default(),
        );

        *self.state.block_number += 1;

        Ok((block, accounts_updated))
    }

    /// Updates the list of accounts that has been updated, aggregates fees, updates blocks operations list from Rollup transaction
    /// Returns current operation index
    ///
    /// # Arguments
    ///
    /// * `op_result` - Rollup transaction execution result
    /// * `accounts_updated` - Updated accounts
    /// * `current_op_block_index` - Current operation index
    /// * `ops` - Current block operations list
    ///
    fn update_from_tx(
        &mut self,
        tx_result: OpSuccess,
        accounts_updated: &mut Vec<(AccountId, AccountUpdate, H256)>,
        current_op_block_index: u32,
        ops: &mut Vec<ExecutedTx>,
    ) -> u32 {
        let OpSuccess {
            updates,
            mut executed_op,
            ..
        } = tx_result;
        let tx_hash = executed_op.try_get_tx().unwrap().hash();
        let mut updates = updates
            .into_iter()
            .map(|update| (update.0, update.1, H256::from_slice(tx_hash.as_ref())))
            .collect::<Vec<_>>();
        accounts_updated.append(&mut updates);
        match &mut executed_op {
            ZkLinkOp::Deposit(op) => {
                *self.last_serial_ids.get_mut(&op.tx.from_chain_id).unwrap() += 1;
                op.tx.serial_id = self.last_serial_ids[&op.tx.from_chain_id] as u64;
            }
            ZkLinkOp::FullExit(op) => {
                *self.last_serial_ids.get_mut(&op.tx.to_chain_id).unwrap() += 1;
                op.tx.serial_id = self.last_serial_ids[&op.tx.to_chain_id] as u64;
            }
            _ => {}
        }

        let block_index = current_op_block_index;
        let exec_result = ExecutedTx {
            tx: executed_op.try_get_tx().unwrap(),
            success: true,
            op: executed_op,
            fail_reason: None,
            block_index: Some(block_index),
            created_at: chrono::Utc::now(),
        };
        ops.push(exec_result);
        current_op_block_index + 1
    }

    /// Returns map of ZkLink accounts ids and their descriptions
    pub fn get_accounts(&self) -> Vec<(u32, Account)> {
        self.state.get_accounts()
    }

    /// Returns sparse Merkle tree root hash
    pub fn root_hash(&self) -> Fr {
        self.state.root_hash()
    }

    /// Returns ZkLink Account id and description by its address
    pub fn get_account_by_address(&self, address: &ZkLinkAddress) -> Option<(AccountId, Account)> {
        self.state.get_account_by_address(address)
    }

    /// Returns ZkLink Account description by its id
    pub fn get_account(&self, account_id: AccountId) -> Option<Account> {
        self.state.get_account(account_id)
    }
}

#[cfg(test)]
mod test {
    use crate::contract::default::get_rollup_ops_from_data;
    use crate::contract::utils::get_rollup_ops_from_data;
    use crate::rollup_ops::RollupOpsBlock;
    use crate::tree_state::TreeState;
    use num::BigUint;
    use zklink_types::{
        AccountId, BlockNumber, ChainId, ChangePubKey, ChangePubKeyOp, Deposit, DepositOp,
        ForcedExit, ForcedExitOp, FullExit, FullExitOp, Nonce, Order, OrderMatching,
        OrderMatchingOp, PubKeyHash, SlotId, SubAccountId, TokenId, Transfer, TransferOp,
        TransferToNewOp, Withdraw, WithdrawOp, ZkLinkOp,
    };

    const ZKL_TOKEN: TokenId = TokenId(32);
    const USD_TOKEN: TokenId = TokenId(1);

    #[test]
    fn test_update_tree_with_one_tx_per_block() {
        // Deposit 1000 to 7
        let tx1 = Deposit {
            from_chain_id: Default::default(),
            from: vec![1u8; 20].into(),
            sub_account_id: Default::default(),
            l1_source_token: Default::default(),
            amount: BigUint::from(1000u32),
            to: vec![7u8; 20].into(),
            serial_id: 0,
            l2_target_token: Default::default(),
            eth_hash: Default::default(),
        };
        let op1 = ZkLinkOp::Deposit(Box::new(DepositOp {
            tx: tx1,
            account_id: AccountId(0),
            l1_source_token_after_mapping: Default::default(),
        }));
        let pub_data1 = op1.public_data();
        let ops1 = get_rollup_ops_from_data(&pub_data1).expect("cant get ops from data 1");
        let block1 = RollupOpsBlock {
            block_num: BlockNumber(1),
            ops: ops1,
            fee_account: AccountId(0),
            timestamp: None,
            previous_block_root_hash: Default::default(),
            contract_version: None,
        };

        // Withdraw 20 with 1 fee from 7 to 10
        let tx2 = Withdraw::new(
            AccountId(0),
            SubAccountId(0),
            ChainId(1),
            vec![9u8; 20].into(),
            ZKL_TOKEN,
            ZKL_TOKEN,
            BigUint::from(20u32),
            BigUint::from(1u32),
            Nonce(1),
            false,
            1,
            None,
            Default::default(),
        );
        let op2 = ZkLinkOp::Withdraw(Box::new(WithdrawOp {
            tx: tx2,
            account_id: AccountId(0),
            l1_target_token_after_mapping: Default::default(),
        }));
        let pub_data2 = op2.public_data();
        let ops2 = get_rollup_ops_from_data(&pub_data2).expect("cant get ops from data 2");
        let block2 = RollupOpsBlock {
            block_num: BlockNumber(2),
            ops: ops2,
            fee_account: AccountId(0),
            timestamp: None,
            previous_block_root_hash: Default::default(),
            contract_version: None,
        };

        // Transfer 40 with 1 fee from 7 to 8
        let tx3 = Transfer::new(
            AccountId(0),
            vec![7u8; 20].into(),
            SubAccountId(0),
            SubAccountId(0),
            ZKL_TOKEN,
            BigUint::from(40u32),
            BigUint::from(1u32),
            Nonce(3),
            None,
            Default::default(),
        );
        let op3 = ZkLinkOp::TransferToNew(Box::new(TransferToNewOp {
            tx: tx3,
            from: AccountId(0),
            to: AccountId(2),
        }));
        let pub_data3 = op3.public_data();
        let ops3 = get_rollup_ops_from_data(&pub_data3).expect("cant get ops from data 3");
        let block3 = RollupOpsBlock {
            block_num: BlockNumber(3),
            ops: ops3,
            fee_account: AccountId(0),
            timestamp: None,
            previous_block_root_hash: Default::default(),
            contract_version: None,
        };

        // Transfer 19 with 1 fee from 8 to 7
        let tx4 = Transfer::new(
            AccountId(1),
            vec![8u8; 20].into(),
            SubAccountId(0),
            SubAccountId(0),
            ZKL_TOKEN,
            BigUint::from(19u32),
            BigUint::from(1u32),
            Nonce(1),
            None,
            Default::default(),
        );
        let op4 = ZkLinkOp::Transfer(Box::new(TransferOp {
            tx: tx4,
            from: AccountId(1),
            to: AccountId(0),
        }));
        let pub_data4 = op4.public_data();
        let ops4 = get_rollup_ops_from_data(&pub_data4).expect("cant get ops from data 4");
        let block4 = RollupOpsBlock {
            block_num: BlockNumber(4),
            ops: ops4,
            fee_account: AccountId(0),
            timestamp: None,
            previous_block_root_hash: Default::default(),
            contract_version: None,
        };

        let pub_key_hash_7 = PubKeyHash::from_hex("sync:8888888888888888888888888888888888888888")
            .expect("Correct pub key hash");
        let tx5 = ChangePubKey::new(
            ChainId(1),
            AccountId(0),
            SubAccountId(0),
            pub_key_hash_7,
            ZKL_TOKEN,
            BigUint::from(1u32),
            Nonce(2),
            None,
            None,
            Default::default(),
        );
        let op5 = ZkLinkOp::ChangePubKeyOffchain(Box::new(ChangePubKeyOp {
            tx: tx5,
            account_id: AccountId(0),
            address: Default::default(),
        }));
        let pub_data5 = op5.public_data();
        let ops5 = get_rollup_ops_from_data(&pub_data5).expect("cant get ops from data 5");
        let block5 = RollupOpsBlock {
            block_num: BlockNumber(5),
            ops: ops5,
            fee_account: AccountId(0),
            timestamp: None,
            previous_block_root_hash: Default::default(),
            contract_version: None,
        };

        // Full exit for 8
        let tx6 = FullExit {
            to_chain_id: Default::default(),
            account_id: AccountId(1),
            sub_account_id: Default::default(),
            exit_address: Default::default(),
            l2_source_token: Default::default(),
            l1_target_token: Default::default(),
            serial_id: 0,
            eth_hash: Default::default(),
        };
        let op6 = ZkLinkOp::FullExit(Box::new(FullExitOp {
            tx: tx6,
            exit_amount: Default::default(),
            l1_target_token_after_mapping: Default::default(),
        }));
        let pub_data6 = op6.public_data();
        let ops6 = get_rollup_ops_from_data(&pub_data6).expect("cant get ops from data 5");
        let block6 = RollupOpsBlock {
            block_num: BlockNumber(5),
            ops: ops6,
            fee_account: AccountId(0),
            timestamp: None,
            previous_block_root_hash: Default::default(),
            contract_version: None,
        };

        // Forced exit for 7
        let tx7 = ForcedExit::new(
            ChainId(1),
            AccountId(0),
            SubAccountId(0),
            vec![7u8; 20].into(),
            SubAccountId(0),
            ZKL_TOKEN,
            ZKL_TOKEN,
            ZKL_TOKEN,
            BigUint::from(1u32),
            Nonce(1),
            None,
            Default::default(),
        );
        let op7 = ZkLinkOp::ForcedExit(Box::new(ForcedExitOp {
            tx: tx7,
            target_account_id: AccountId(0),
            withdraw_amount: BigUint::from(960u32).into(),
            l1_target_token_after_mapping: Default::default(),
        }));
        let pub_data7 = op7.public_data();
        let ops7 = get_rollup_ops_from_data(&pub_data7).expect("cant get ops from data 5");
        let block7 = RollupOpsBlock {
            block_num: BlockNumber(7),
            ops: ops7,
            fee_account: AccountId(1),
            timestamp: None,
            previous_block_root_hash: Default::default(),
            contract_version: None,
        };

        // OrderMatching for 8
        let maker_order = Order::new(
            AccountId(0),
            SubAccountId(0),
            SlotId(0),
            Nonce(0),
            ZKL_TOKEN,
            USD_TOKEN,
            BigUint::from(1000000000000000000u128),
            BigUint::from(2000000000000000000u128),
            true,
            5,
            10,
            None,
        );
        let taker_order = Order::new(
            AccountId(0),
            SubAccountId(0),
            SlotId(1),
            Nonce(0),
            ZKL_TOKEN,
            USD_TOKEN,
            BigUint::from(1000000000000000000u128),
            BigUint::from(2000000000000000000u128),
            true,
            5,
            10,
            None,
        );
        let tx8 = OrderMatching::new(
            AccountId(0),
            SubAccountId(0),
            maker_order,
            taker_order,
            BigUint::from(1u32),
            TokenId(1),
            Default::default(),
            Default::default(),
            None,
        );
        let op8 = ZkLinkOp::OrderMatching(Box::new(OrderMatchingOp {
            tx: tx8,
            submitter: Default::default(),
            maker: Default::default(),
            taker: Default::default(),
            maker_context: Default::default(),
            taker_context: Default::default(),
        }));
        let pub_data8 = op8.public_data();
        let ops8 = get_rollup_ops_from_data(&pub_data8).expect("cant get ops from data 5");
        let block8 = RollupOpsBlock {
            block_num: BlockNumber(8),
            ops: ops8,
            fee_account: AccountId(1),
            timestamp: None,
            previous_block_root_hash: Default::default(),
            contract_version: None,
        };

        // let available_block_chunk_sizes = vec![10, 32, 72, 156, 322, 654];
        // let mut tree = TreeState::new();
        // tree.apply_ops_block(&block1)
        //     .expect("Cant update state from block 1");
        // let zero_acc = tree.get_account(AccountId(0)).expect("Cant get 0 account");
        // assert_eq!(zero_acc.address, vec![7u8; 20].into());
        // assert_eq!(zero_acc.get_balance(TokenId(1)), BigUint::from(1000u32));
        //
        // tree.apply_ops_block(&block2)
        //     .expect("Cant update state from block 2");
        // let zero_acc = tree.get_account(AccountId(0)).expect("Cant get 0 account");
        // assert_eq!(zero_acc.get_balance(TokenId(1)), BigUint::from(980u32));
        //
        // tree.apply_ops_block(&block3)
        //     .expect("Cant update state from block 3");
        // // Verify creating accounts
        // assert_eq!(tree.get_accounts().len(), 2);
        //
        // let zero_acc = tree.get_account(AccountId(0)).expect("Cant get 0 account");
        // let first_acc = tree.get_account(AccountId(1)).expect("Cant get 0 account");
        // assert_eq!(first_acc.address, vec![8u8; 20].into());
        //
        // assert_eq!(zero_acc.get_balance(TokenId(1)), BigUint::from(940u32));
        // assert_eq!(first_acc.get_balance(TokenId(1)), BigUint::from(40u32));
        //
        // tree.apply_ops_block(&block4)
        //     .expect("Cant update state from block 4");
        // let zero_acc = tree.get_account(AccountId(0)).expect("Cant get 0 account");
        // let first_acc = tree.get_account(AccountId(1)).expect("Cant get 0 account");
        // assert_eq!(zero_acc.get_balance(TokenId(1)), BigUint::from(960u32));
        // assert_eq!(first_acc.get_balance(TokenId(1)), BigUint::from(20u32));
        //
        // assert_eq!(zero_acc.pub_key_hash, PubKeyHash::zero());
        // tree.apply_ops_block(&block5)
        //     .expect("Cant update state from block 5");
        // let zero_acc = tree.get_account(AccountId(0)).expect("Cant get 0 account");
        // assert_eq!(zero_acc.pub_key_hash, pub_key_hash_7);
        //
        // tree.apply_ops_block(&block6)
        //     .expect("Cant update state from block 6");
        // let zero_acc = tree.get_account(AccountId(0)).expect("Cant get 0 account");
        // let first_acc = tree.get_account(AccountId(1)).expect("Cant get 0 account");
        // assert_eq!(zero_acc.get_balance(TokenId(1)), BigUint::from(960u32));
        // assert_eq!(first_acc.get_balance(TokenId(1)), BigUint::from(0u32));
        //
        // tree.apply_ops_block(&block7)
        //     .expect("Cant update state from block 7");
        // let zero_acc = tree.get_account(AccountId(0)).expect("Cant get 0 account");
        // let first_acc = tree.get_account(AccountId(1)).expect("Cant get 0 account");
        // assert_eq!(zero_acc.get_balance(TokenId(1)), BigUint::from(0u32));
        // assert_eq!(first_acc.get_balance(TokenId(1)), BigUint::from(1u32));
        //
        // tree.apply_ops_block(&block8)
        //     .expect("Cant update state from block 8");
        // let zero_acc = tree.get_account(AccountId(0)).expect("Cant get 0 account");
        // let first_acc = tree.get_account(AccountId(1)).expect("Cant get 0 account");
        // assert_eq!(zero_acc.get_balance(TokenId(1)), BigUint::from(0u32));
        // assert_eq!(first_acc.get_balance(TokenId(1)), BigUint::from(1u32));
    }

    #[test]
    fn test_update_tree_with_multiple_txs_per_block() {
        let tx1 = Deposit {
            from_chain_id: ChainId(1),
            from: vec![1u8; 20].into(),
            sub_account_id: Default::default(),
            l1_source_token: ZKL_TOKEN,
            amount: BigUint::from(1000u32),
            to: vec![7u8; 20].into(),
            serial_id: 0,
            l2_target_token: ZKL_TOKEN,
            eth_hash: Default::default(),
        };
        let op1 = ZkLinkOp::Deposit(Box::new(DepositOp {
            tx: Deposit {
                from_chain_id: ChainId(1),
                from: Default::default(),
                sub_account_id: Default::default(),
                l1_source_token: ZKL_TOKEN,
                l2_target_token: ZKL_TOKEN,
                amount: Default::default(),
                to: Default::default(),
                serial_id: 0,
                eth_hash: Default::default(),
            },
            account_id: AccountId(0),
            l1_source_token_after_mapping: Default::default(),
        }));
        let pub_data1 = op1.public_data();

        let tx2 = Withdraw::new(
            AccountId(0),
            SubAccountId(0),
            ChainId(1),
            vec![9u8; 20].into(),
            ZKL_TOKEN,
            ZKL_TOKEN,
            BigUint::from(20u32),
            BigUint::from(1u32),
            Nonce(1),
            false,
            10,
            None,
            Default::default(),
        );
        let op2 = ZkLinkOp::Withdraw(Box::new(WithdrawOp {
            tx: tx2,
            account_id: AccountId(0),
            l1_target_token_after_mapping: Default::default(),
        }));
        let pub_data2 = op2.public_data();

        let tx3 = Transfer::new(
            AccountId(0),
            vec![8u8; 20].into(),
            SubAccountId(0),
            SubAccountId(0),
            ZKL_TOKEN,
            BigUint::from(40u32),
            BigUint::from(1u32),
            Nonce(3),
            None,
            Default::default(),
        );
        let op3 = ZkLinkOp::TransferToNew(Box::new(TransferToNewOp {
            tx: tx3,
            from: AccountId(0),
            to: AccountId(1),
        }));
        let pub_data3 = op3.public_data();

        let tx4 = Transfer::new(
            AccountId(1),
            vec![7u8; 20].into(),
            SubAccountId(0),
            SubAccountId(0),
            ZKL_TOKEN,
            BigUint::from(19u32),
            BigUint::from(1u32),
            Nonce(1),
            None,
            Default::default(),
        );
        let op4 = ZkLinkOp::Transfer(Box::new(TransferOp {
            tx: tx4,
            from: AccountId(1),
            to: AccountId(0),
        }));
        let pub_data4 = op4.public_data();

        let pub_key_hash_7 = PubKeyHash::from_hex("sync:8888888888888888888888888888888888888888")
            .expect("Correct pub key hash");
        let tx5 = ChangePubKey::new(
            ChainId(1),
            AccountId(0),
            SubAccountId(0),
            pub_key_hash_7,
            ZKL_TOKEN,
            BigUint::from(1u32),
            Nonce(2),
            None,
            None,
            Default::default(),
        );
        let op5 = ZkLinkOp::ChangePubKeyOffchain(Box::new(ChangePubKeyOp {
            tx: tx5,
            account_id: AccountId(0),
            address: Default::default(),
        }));
        let pub_data5 = op5.public_data();

        let tx6 = FullExit {
            to_chain_id: Default::default(),
            account_id: AccountId(1),
            sub_account_id: Default::default(),
            exit_address: vec![8u8; 20].into(),
            l2_source_token: Default::default(),
            l1_target_token: Default::default(),
            serial_id: 0,
            eth_hash: Default::default(),
        };
        let op6 = ZkLinkOp::FullExit(Box::new(FullExitOp {
            tx: FullExit {
                to_chain_id: Default::default(),
                account_id: Default::default(),
                sub_account_id: Default::default(),
                exit_address: Default::default(),
                l2_source_token: Default::default(),
                l1_target_token: Default::default(),
                serial_id: 0,
                eth_hash: Default::default(),
            },
            exit_amount: Default::default(),
            l1_target_token_after_mapping: Default::default(),
        }));
        let pub_data6 = op6.public_data();

        let tx7 = ForcedExit::new(
            ChainId(1),
            AccountId(0),
            SubAccountId(0),
            vec![7u8; 20].into(),
            SubAccountId(0),
            ZKL_TOKEN,
            ZKL_TOKEN,
            ZKL_TOKEN,
            BigUint::from(1u32),
            Nonce(1),
            None,
            Default::default(),
        );
        let op7 = ZkLinkOp::ForcedExit(Box::new(ForcedExitOp {
            tx: tx7,
            target_account_id: AccountId(0),
            withdraw_amount: BigUint::from(956u32).into(),
            l1_target_token_after_mapping: Default::default(),
        }));
        let pub_data7 = op7.public_data();

        let mut pub_data = Vec::new();
        pub_data.extend_from_slice(&pub_data1);
        pub_data.extend_from_slice(&pub_data2);
        pub_data.extend_from_slice(&pub_data3);
        pub_data.extend_from_slice(&pub_data4);
        pub_data.extend_from_slice(&pub_data5);
        pub_data.extend_from_slice(&pub_data6);
        pub_data.extend_from_slice(&pub_data7);

        let ops = get_rollup_ops_from_data(pub_data.as_slice()).expect("cant get ops from data 1");
        let block = RollupOpsBlock {
            block_num: BlockNumber(1),
            ops,
            fee_account: AccountId(0),
            timestamp: None,
            previous_block_root_hash: Default::default(),
            contract_version: None,
        };

        let mut tree = TreeState::new();
        let available_block_chunk_sizes = vec![10, 32, 72, 156, 322, 654];
        tree.apply_ops_block(&block)
            .expect("Cant update state from block");

        assert_eq!(tree.get_accounts().len(), 2);

        let zero_acc = tree.get_account(AccountId(0)).expect("Cant get 0 account");
        assert_eq!(zero_acc.address, vec![7u8; 20].into());
        assert_eq!(zero_acc.get_balance(TokenId(1)), BigUint::from(5u32));
        assert_eq!(zero_acc.pub_key_hash, pub_key_hash_7);

        let first_acc = tree.get_account(AccountId(1)).expect("Cant get 0 account");
        assert_eq!(first_acc.address, vec![8u8; 20].into());
        assert_eq!(first_acc.get_balance(TokenId(1)), BigUint::from(0u32));
    }
}
