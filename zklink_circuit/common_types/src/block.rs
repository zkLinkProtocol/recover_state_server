//! zklink network block definition.

use std::convert::TryInto;
use chrono::DateTime;
use chrono::Utc;
use parity_crypto::digest::sha256;
use parity_crypto::Keccak256;
use serde::{Deserialize, Serialize};
use zklink_basic_types::{ChainId, H256, U256};
use zklink_crypto::franklin_crypto::bellman::pairing::ff::{PrimeField, PrimeFieldRepr};
use zklink_crypto::params::{ALL_DIFFERENT_TRANSACTIONS_TYPE_NUMBER, CHUNK_BYTES};
use zklink_crypto::serialization::FrSerde;

use crate::ZkLinkTx;

use super::ZkLinkOp;
use super::{AccountId, BlockNumber, Fr};

/// Executed L2 transaction.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutedTx {
    pub tx: ZkLinkTx,
    pub success: bool,
    pub op: ZkLinkOp,
    pub fail_reason: Option<String>,
    pub block_index: Option<u32>,
    pub created_at: DateTime<Utc>,
}

/// Executed L2 transaction.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FailedExecutedTx {
    pub tx: ZkLinkTx,
    pub success: bool,
    pub fail_reason: Option<String>,
    pub block_index: Option<u32>,
    pub created_at: DateTime<Utc>,
}

impl ExecutedTx {
    /// Returns the `ZkLinkOp` object associated with the operation, if any.
    pub fn get_executed_op(&self) -> &ZkLinkOp {
        &self.op
    }
}

/// zklink network block.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Block {
    /// Block ID.
    pub block_number: BlockNumber,
    /// Chain root hash obtained after executing this block.
    #[serde(with = "FrSerde")]
    pub new_root_hash: Fr,
    /// ID of the zklink account to which fees are collected.
    pub fee_account: AccountId,
    /// List of operations executed in the block. Includes both L1 and L2 operations.
    pub block_transactions: Vec<ExecutedTx>,
    /// Actual block chunks amount that will be used on contract, such that `block_chunks_sizes >= block.chunks_used()`.
    /// Server and provers may support blocks of several different sizes, and this value must be equal to one of the
    /// supported size values.
    pub block_chunks_size: usize,
    pub ops_composition_number: usize,

    /// Gas limit to be set for the Commit Ethereum transaction.
    pub commit_gas_limit: U256,
    /// Gas limit to be set for the Verify Ethereum transaction.
    pub verify_gas_limit: U256,
    /// Commitment
    pub block_commitment: H256,
    /// Sync hash
    pub sync_hash: H256,
    /// Timestamp
    pub timestamp: u64,
}

/// StoredBlockInfo is defined in Storage.sol
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoredBlockInfo {
    pub block_number: BlockNumber,             // Rollup block number
    pub priority_operations: u64,              // Number of priority operations processed
    pub pending_onchain_operations_hash: H256, // Hash of all operations that must be processed after verify
    pub timestamp: U256, // Rollup block timestamp, have the same format as Ethereum block constant
    pub state_hash: H256, // Root hash of the rollup state
    pub commitment: H256, // Verified input for the ZkLink circuit
    pub sync_hash: H256, // Used for cross chain block verify
}

/// OnchainOperationsBlockInfo is defined in ZkLink.sol
#[derive(Debug, Clone)]
pub struct OnchainOperationsBlockInfo {
    pub public_data_offset: u32, // Byte offset in public data for onchain operation
    pub eth_witness: Vec<u8>,    // Some external data that can be needed for operation processing
}

/// CommitBlockInfo is defined in ZkLink.sol
#[derive(Debug, Clone)]
pub struct CommitBlockInfo {
    pub new_state_hash: H256, // Root hash of the rollup state
    pub public_data: Vec<u8>, // Contain pubdata of all chains when compressed is disabled or only current chain if compressed is enable
    pub timestamp: U256,      // Rollup block timestamp
    pub onchain_operations: Vec<OnchainOperationsBlockInfo>, // Contain onchain ops of all chains when compressed is disabled or only current chain if compressed is enable
    pub block_number: BlockNumber,                           // Rollup block number
    pub fee_account: AccountId,                              // Fee account id
}

/// CompressedBlockExtraInfo is defined in ZkLink.sol
#[derive(Debug, Clone)]
pub struct CompressedBlockExtraInfo {
    pub public_data_hash: H256,       // Pubdata hash of all chains
    pub offset_commitment_hash: H256, // All chains pubdata offset commitment hash
    pub onchain_operation_pubdata_hashs: Vec<H256>, // Onchain operation pubdata hash of the all other chains
}

/// ExecuteBlockInfo is defined in ZkLink.sol
pub struct ExecuteBlockInfo {
    pub stored_block: StoredBlockInfo, // The block info that will be executed
    pub pending_onchain_ops_pubdata: Vec<Vec<u8>>, // Onchain ops(e.g. Withdraw, ForcedExit, FullExit) that will be executed
}

impl Block {
    /// Creates a new `Block` object.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        block_number: BlockNumber,
        new_root_hash: Fr,
        fee_account: AccountId,
        block_transactions: Vec<ExecutedTx>,
        block_chunks_size: usize,
        ops_composition_number: usize,
        commit_gas_limit: U256,
        verify_gas_limit: U256,
        block_commitment: H256,
        sync_hash: H256,
        timestamp: u64,
    ) -> Self {
        Self {
            block_number,
            new_root_hash,
            fee_account,
            block_transactions,
            ops_composition_number,
            block_chunks_size,
            commit_gas_limit,
            verify_gas_limit,
            block_commitment,
            sync_hash,
            timestamp,
        }
    }

    /// Creates a new block, choosing the smallest supported block size which will fit
    /// all the executed transactions.
    ///
    /// # Panics
    ///
    /// Panics if there is no supported block size to fit all the transactions.
    #[allow(clippy::too_many_arguments)]
    pub fn new_from_available_block_sizes(
        block_number: BlockNumber,
        new_root_hash: Fr,
        fee_account: AccountId,
        block_transactions: Vec<ExecutedTx>,
        block_chunks_size: usize,
        support_ops_numbers: &[usize],
        available_chain_ids: &[ChainId],
        commit_gas_limit: U256,
        verify_gas_limit: U256,
        previous_block_root_hash: H256,
        previous_block_sync_hash: H256,
        timestamp: u64,
    ) -> Self {
        let mut block = Self {
            block_number,
            new_root_hash,
            fee_account,
            block_transactions,
            block_chunks_size,
            ops_composition_number: 0,
            commit_gas_limit,
            verify_gas_limit,
            block_commitment: H256::default(),
            sync_hash: H256::default(),
            timestamp,
        };
        block.ops_composition_number = block.ops_composition_number(support_ops_numbers);
        // caculate_block_accumulators(&mut block);
        let block_commitment = Block::get_commitment(
            block_number,
            fee_account,
            previous_block_root_hash,
            block.get_eth_encoded_root(),
            block.timestamp,
            &block.get_onchain_op_commitment(),
            &block.get_eth_public_data(),
        );
        block.block_commitment = block_commitment;
        // cal sync hash depends on block_commitment
        block.sync_hash = block.get_sync_hash(previous_block_sync_hash, available_chain_ids);
        block
    }

    /// Returns the new state root hash encoded for the Ethereum smart contract.
    pub fn get_eth_encoded_root(&self) -> H256 {
        let mut be_bytes = [0u8; 32];
        self.new_root_hash
            .into_repr()
            .write_be(be_bytes.as_mut())
            .expect("Write commit bytes");
        H256::from(be_bytes)
    }

    pub fn get_block_commitment(&self, old_state_hash: H256) -> H256 {
        Block::get_commitment(
            self.block_number,
            self.fee_account,
            old_state_hash,
            self.get_eth_encoded_root(),
            self.timestamp,
            &self.get_onchain_op_commitment(),
            &self.get_eth_public_data(),
        )
    }

    /// Returns the public data for the Ethereum Commit operation.
    pub fn get_eth_public_data(&self) -> Vec<u8> {
        let mut executed_tx_pub_data = self
            .block_transactions
            .iter()
            .map(ExecutedTx::get_executed_op)
            .flat_map(ZkLinkOp::public_data)
            .collect::<Vec<_>>();

        // Pad block with noops.
        executed_tx_pub_data.resize(self.block_chunks_size * CHUNK_BYTES, 0x00);
        executed_tx_pub_data
    }

    /// Returns the public data for the Ethereum CommitCompressed operation.
    pub fn get_eth_public_data_with_compress(&self, chain_id: ChainId) -> Vec<u8> {
        let executed_tx_pub_data = self
            .block_transactions
            .iter()
            .filter_map(|tx| {
                let op = tx.get_executed_op();
                if op.is_local_onchain_operation(chain_id) {
                    Some(op)
                } else {
                    None
                }
            })
            .flat_map(ZkLinkOp::public_data)
            .collect::<Vec<_>>();

        executed_tx_pub_data
    }

    /// Returns eth_witness data and data_size for each operation that has it.
    pub fn get_eth_witness_data(&self) -> (Vec<u8>, Vec<u64>) {
        let mut eth_witness = Vec::new();
        let mut used_bytes = Vec::new();

        for block_tx in &self.block_transactions {
            let franklin_op = block_tx.get_executed_op();
            if let Some(witness_bytes) = franklin_op.eth_witness() {
                used_bytes.push(witness_bytes.len() as u64);
                eth_witness.extend(witness_bytes.into_iter());
            }
        }

        (eth_witness, used_bytes)
    }

    /// Returns the number of priority operations processed in this block.
    ///
    /// Keep same with `checkOnchainOp` in ZkLink.sol
    pub fn number_of_processed_prior_ops(&self, chain_id: ChainId) -> u64 {
        let mut count = 0u64;
        for tx in &self.block_transactions {
            match &tx.tx {
                ZkLinkTx::FullExit(tx) => {
                    if tx.to_chain_id == chain_id {
                        count += 1;
                    }
                }
                ZkLinkTx::Deposit(tx) => {
                    if tx.from_chain_id == chain_id {
                        count += 1;
                    }
                }

                _ => {}
            }
        }
        count
    }

    fn ops_composition_number(&self, support_ops_numbers: &[usize]) -> usize {
        let mut contains_ops = [false; ALL_DIFFERENT_TRANSACTIONS_TYPE_NUMBER];
        self.block_transactions
            .iter()
            .map(ExecutedTx::get_executed_op)
            .for_each(|op| contains_ops[op.op_code()] = true);
        find_exec_ops_number(contains_ops, support_ops_numbers)
    }

    /// Returns the number of Withdrawal and ForcedExit in a block.
    pub fn get_withdrawals_count(&self) -> usize {
        let mut withdrawals_count = 0;

        for block_tx in &self.block_transactions {
            let sync_op = block_tx.get_executed_op();
            if sync_op.withdrawal_data().is_some() {
                withdrawals_count += 1;
            }
        }

        withdrawals_count
    }

    /// Returns the data about withdrawals required for the Ethereum smart contract.
    pub fn get_withdrawals_data(&self) -> Vec<u8> {
        let mut withdrawals_data = Vec::new();

        for block_tx in &self.block_transactions {
            let franklin_op = block_tx.get_executed_op();
            if let Some(withdrawal_data) = franklin_op.withdrawal_data() {
                withdrawals_data.extend(&withdrawal_data);
            }
        }

        withdrawals_data
    }

    /// Get onchain ops of all chain
    pub fn get_onchain_ops(&self) -> Vec<OnchainOperationsBlockInfo> {
        let mut onchain_ops = Vec::new();
        let mut public_data_offset = 0;
        for op in &self.block_transactions {
            let executed_op = op.get_executed_op();
            if executed_op.is_onchain_operation() {
                onchain_ops.push(OnchainOperationsBlockInfo {
                    public_data_offset,
                    eth_witness: executed_op.eth_witness().unwrap_or_default(),
                });
            }

            public_data_offset += (CHUNK_BYTES * executed_op.chunks()) as u32;
        }
        onchain_ops
    }

    /// Get onchain ops of a chain
    pub fn get_onchain_ops_of_chain(&self, chain_id: ChainId) -> Vec<OnchainOperationsBlockInfo> {
        let mut onchain_ops = Vec::new();
        let mut public_data_offset = 0;

        for op in &self.block_transactions {
            let executed_op = op.get_executed_op();
            if executed_op.is_local_onchain_operation(chain_id) {
                onchain_ops.push(OnchainOperationsBlockInfo {
                    public_data_offset,
                    eth_witness: executed_op.eth_witness().unwrap_or_default(),
                });

                public_data_offset += (CHUNK_BYTES * executed_op.chunks()) as u32;
            }
        }
        onchain_ops
    }

    /// Get hash of processable operations commit to a chain
    pub fn get_processable_operations_hash_of_chain(&self, chain_id: ChainId) -> H256 {
        let mut processable_operations_hash = Vec::new().keccak256();
        for op in &self.block_transactions {
            let executed_op = op.get_executed_op();
            if executed_op.is_processable_onchain_operation(chain_id) {
                processable_operations_hash = [
                    processable_operations_hash.to_vec(),
                    executed_op.public_data().as_slice().to_vec(),
                ]
                .concat()
                .keccak256();
            }
        }
        H256::from(processable_operations_hash)
    }

    /// Get onchain op pubdata hash of each chain
    pub fn get_onchain_op_pubdata_hashs(&self, max_chain_id: &ChainId) -> Vec<H256> {
        let mut onchain_operation_pubdata_hashs =
            vec![Vec::new().keccak256(); max_chain_id.0 as usize + 1];
        for op in &self.block_transactions {
            let executed_op = op.get_executed_op();
            if executed_op.is_onchain_operation() {
                let op_chain_id = executed_op.get_onchain_operation_chain_id() as usize;
                onchain_operation_pubdata_hashs[op_chain_id] = [
                    onchain_operation_pubdata_hashs[op_chain_id].to_vec(),
                    executed_op.public_data().as_slice().to_vec(),
                ]
                .concat()
                .keccak256();
            }
        }
        let onchain_op_pubdata_hashs = onchain_operation_pubdata_hashs
            .iter()
            .map(|h| H256::from(*h))
            .collect();
        onchain_op_pubdata_hashs
    }

    /// Returns the public data for the Ethereum Commit operation.
    pub fn get_onchain_op_commitment(&self) -> Vec<u8> {
        let mut res = vec![0u8; self.block_chunks_size];
        for op in self.get_onchain_ops() {
            res[op.public_data_offset as usize / CHUNK_BYTES] = 0x01;
        }
        res
    }

    pub fn get_commitment(
        block_number: BlockNumber,
        fee_account: AccountId,
        old_state_hash: H256,
        new_state_hash: H256,
        timestamp: u64,
        onchain_op_commitment: &[u8],
        public_data: &[u8],
    ) -> H256 {
        let mut hash_arg = vec![0u8; 160];
        U256::from(*block_number).to_big_endian(&mut hash_arg[0..32]);
        U256::from(*fee_account).to_big_endian(&mut hash_arg[32..64]);
        hash_arg[64..96].copy_from_slice(old_state_hash.as_bytes());
        hash_arg[96..128].copy_from_slice(new_state_hash.as_bytes());

        U256::from(timestamp).to_big_endian(&mut hash_arg[128..]);

        hash_arg.extend_from_slice(&sha256(public_data));
        hash_arg.extend_from_slice(&sha256(onchain_op_commitment));

        hash_arg = sha256(&hash_arg).to_vec();
        H256::from_slice(&hash_arg)
    }

    pub fn processable_ops_pubdata(&self, chain_id: ChainId) -> Vec<Vec<u8>> {
        self.block_transactions
            .iter()
            .map(|tx| tx.get_executed_op())
            .filter_map(|op| {
                if op.is_processable_onchain_operation(chain_id) {
                    Some(op.public_data())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_sync_hash(
        &self,
        previous_block_sync_hash: H256,
        available_chain_ids: &[ChainId],
    ) -> H256 {
        let sync_hash = if self.block_number.0 == 0 {
            Vec::new().keccak256().to_vec()
        } else {
            let max_chain_id = available_chain_ids.iter().max().unwrap();
            let onchain_op_pubdata_hashs = self.get_onchain_op_pubdata_hashs(max_chain_id);
            let mut sync_hash_tmp = [
                previous_block_sync_hash.as_bytes().to_vec(),
                self.block_commitment.as_bytes().to_vec(),
            ]
            .concat()
            .keccak256()
            .to_vec();
            for i in available_chain_ids.iter() {
                sync_hash_tmp = [
                    sync_hash_tmp,
                    onchain_op_pubdata_hashs[i.0 as usize].as_bytes().to_vec(),
                ]
                .concat()
                .keccak256()
                .to_vec();
            }
            sync_hash_tmp
        };
        H256::from_slice(&sync_hash)
    }

    pub fn stored_block_info(&self, chain_id: ChainId) -> StoredBlockInfo {
        StoredBlockInfo {
            block_number: self.block_number,
            priority_operations: self.number_of_processed_prior_ops(chain_id),
            pending_onchain_operations_hash: self
                .get_processable_operations_hash_of_chain(chain_id),
            timestamp: self.timestamp.into(),
            state_hash: self.get_eth_encoded_root(),
            commitment: self.block_commitment,
            sync_hash: self.sync_hash,
        }
    }

    pub fn uncompressed_commit_block_info(&self) -> CommitBlockInfo {
        CommitBlockInfo {
            new_state_hash: self.get_eth_encoded_root(),
            public_data: self.get_eth_public_data(),
            timestamp: self.timestamp.into(),
            onchain_operations: self.get_onchain_ops(),
            block_number: self.block_number,
            fee_account: self.fee_account,
        }
    }

    pub fn compressed_commit_block_info(&self, chain_id: ChainId) -> CommitBlockInfo {
        CommitBlockInfo {
            new_state_hash: self.get_eth_encoded_root(),
            public_data: self.get_eth_public_data_with_compress(chain_id),
            timestamp: self.timestamp.into(),
            onchain_operations: self.get_onchain_ops_of_chain(chain_id),
            block_number: self.block_number,
            fee_account: self.fee_account,
        }
    }

    pub fn compressed_commit_extra_info(
        &self,
        available_chain_ids: &[ChainId],
    ) -> CompressedBlockExtraInfo {
        let public_data_all = self.get_eth_public_data();
        let public_data_hash = H256::from_slice(sha256(&public_data_all).to_vec().as_slice());
        let offset_commitment = self.get_onchain_op_commitment();
        let offset_commitment_hash =
            H256::from_slice(sha256(&offset_commitment).to_vec().as_slice());
        let max_chain_id = available_chain_ids.iter().max().unwrap();
        CompressedBlockExtraInfo {
            public_data_hash,
            offset_commitment_hash,
            onchain_operation_pubdata_hashs: self.get_onchain_op_pubdata_hashs(max_chain_id),
        }
    }

    pub fn execute_info(&self, chain_id: ChainId) -> ExecuteBlockInfo {
        ExecuteBlockInfo {
            stored_block: self.stored_block_info(chain_id),
            pending_onchain_ops_pubdata: self.processable_ops_pubdata(chain_id),
        }
    }
}

/// find the number of ops composition
pub fn find_exec_ops_number(
    contains_ops: [bool; ALL_DIFFERENT_TRANSACTIONS_TYPE_NUMBER],
    support_ops_numbers: &[usize],
) -> usize {
    let exec_ops_number = convert_ops_compositions_to_number(contains_ops.iter());
    *support_ops_numbers
        .iter()
        .find(|&num| *num | exec_ops_number == *num)
        .unwrap()
}

pub fn convert_number_to_ops_compositions(
    num: usize,
) -> [bool; ALL_DIFFERENT_TRANSACTIONS_TYPE_NUMBER] {
    (0..ALL_DIFFERENT_TRANSACTIONS_TYPE_NUMBER)
        .map(|i| num >> i & 1 == 1)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

pub fn convert_ops_compositions_to_number<'a, T: Iterator<Item = &'a bool>>(
    contains_ops: T,
) -> usize {
    contains_ops
        .enumerate()
        .map(|(i, &contain)| if contain { 1 << i } else { 0 })
        .sum()
}
