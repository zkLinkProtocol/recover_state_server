//! This module provides utilities for estimating the gas costs for
//! the transactions that server sends to the Ethereum network.
//! Server uses this module to ensure that generated transactions
//! won't run out of the gas and won't trespass the block gas limit.
// Workspace deps
use zklink_basic_types::*;
// Local deps
use crate::{Block, ZkLinkOp};

/// These values are estimated by execute the `npx hardhat run script/gas_estimate.js` in `zklink-contracts`.
/// More details see https://github.com/zkLinkProtocol/zklink-tech-docs/blob/main/development/gas_estimate.md
///
/// When commit block in compressed mode, cost of some op may be cheaper.
/// But we still use the test result of non-compressed to define the COST of these ops.
/// Because there will always be a chain that needs to make a non-compressed commit.
/// We must ensure that transactions submitted to any chain do not exceed the gas limit.
///
/// tx_estimate_gas = BASE_BLOCKS_TX_COST + sum(block_estimate_cost)
///
/// block_estimate_cost = BASE_COST + sum(op_cost)

#[derive(Debug)]
pub struct CommitCost;

impl CommitCost {
    pub const BASE_COST: u64 = 32334;
    pub const DEPOSIT_COST: u64 = 9024;
    /// Use the max cost of (FullExitERC20, FullExitETH)
    pub const FULL_EXIT_COST: u64 = 9894;
    pub const CHANGE_PUBKEY_COST_ECDSA: u64 = 12260;
    pub const NOT_EXEC_CHAIN_CHANGE_PUBKEY_COST_ECDSA: u64 = 5593;
    pub const CHANGE_PUBKEY_COST_CREATE2: u64 = 7806;
    pub const NOT_EXEC_CHAIN_CHANGE_PUBKEY_COST_CREATE2: u64 = 5077;
    pub const CHANGE_PUBKEY_COST_ONCHAIN: u64 = 6057;
    pub const NOT_EXEC_CHAIN_CHANGE_PUBKEY_COST_ONCHAIN: u64 = 4358;
    pub const TRANSFER_COST: u64 = 691;
    pub const TRANSFER_TO_NEW_COST: u64 = 1069;
    /// Use the max cost of (NormalWithdrawERC20, NormalWithdrawETH, FastWithdraw)
    pub const WITHDRAW_COST: u64 = 4875;
    pub const NOT_EXEC_CHAIN_WITHDRAW_COST: u64 = 4230;
    pub const FORCED_EXIT_COST: u64 = Self::WITHDRAW_COST;
    pub const NOT_EXEC_CHAIN_FORCED_EXIT_COST: u64 = Self::NOT_EXEC_CHAIN_WITHDRAW_COST;
    pub const ORDER_MATCHING_COST: u64 = 1306;
    /// Make sure we can contain at least one transaction
    pub const MIN_GAS: u64 = Self::BASE_COST + Self::CHANGE_PUBKEY_COST_ECDSA;

    pub fn base_cost() -> U256 {
        U256::from(Self::BASE_COST)
    }

    pub fn op_cost_with_exec_chain(op: &ZkLinkOp) -> U256 {
        let cost = match op {
            ZkLinkOp::Noop(_) => 0,
            ZkLinkOp::Deposit(_) => Self::DEPOSIT_COST,
            ZkLinkOp::ChangePubKeyOffchain(change_pubkey) => {
                if change_pubkey.tx.is_onchain() {
                    Self::CHANGE_PUBKEY_COST_ONCHAIN
                } else if change_pubkey.tx.eth_auth_data.is_eth_ecdsa(){
                    Self::CHANGE_PUBKEY_COST_ECDSA
                } else {
                    Self::CHANGE_PUBKEY_COST_CREATE2
                }
            }
            ZkLinkOp::Transfer(_) => Self::TRANSFER_COST,
            ZkLinkOp::TransferToNew(_) => Self::TRANSFER_TO_NEW_COST,
            ZkLinkOp::FullExit(_) => Self::FULL_EXIT_COST,
            ZkLinkOp::Withdraw(_) => Self::WITHDRAW_COST,
            ZkLinkOp::ForcedExit(_) => Self::FORCED_EXIT_COST,
            ZkLinkOp::OrderMatching(_) => Self::ORDER_MATCHING_COST,
        };

        U256::from(cost)
    }
}

#[derive(Debug)]
pub struct VerifyCost;

impl VerifyCost {
    pub const BASE_COST: u64 = 7168;
    pub const DEPOSIT_COST: u64 = 127;

    /// The max test result of (NormalWithdrawERC20, NormalWithdrawETH, FastWithdraw) is 12752
    ///
    /// The max test result of (FullExitERC20, FullExitETH) is 12796
    ///
    /// In zkLink contract WITHDRAWAL_GAS_LIMIT(100000) will be send when call erc20 transfer
    /// and the final cost of transfer is unknown, when it need more than WITHDRAWAL_GAS_LIMIT call will be failed
    /// ```js
    /// let BLOCK_GAS_LIMIT = 12500000
    /// let MAX_WITHDRAW_TXS = BLOCK_GAS_LIMIT/WITHDRAWAL_GAS_LIMIT = 125
    /// let MAX_BLOCK_CHUNKS = MAX_WITHDRAW_TXS*WITHDRAW_CHUNK_SIZE
    /// = 125 * 3 = 375
    /// ```
    /// Mostly a transfer cost 20000-50000(not sure for now), the WITHDRAWAL_GAS_LIMIT is more larger than really cost
    /// the less withdraw txs we can package in a block
    pub const WITHDRAWAL_GAS_LIMIT: u64 = 100000;
    pub const FULL_EXIT_COST: u64 = Self::WITHDRAWAL_GAS_LIMIT;
    pub const NOT_EXEC_CHAIN_FULL_EXIT_COST: u64 = 71;
    pub const CHANGE_PUBKEY_COST: u64 = 71;
    pub const TRANSFER_COST: u64 = 71;
    pub const TRANSFER_TO_NEW_COST: u64 = 71;

    pub const WITHDRAW_COST: u64 = Self::WITHDRAWAL_GAS_LIMIT;
    pub const NOT_EXEC_CHAIN_WITHDRAW_COST: u64 = 71;
    pub const FORCED_EXIT_COST: u64 = Self::FULL_EXIT_COST;
    pub const NOT_EXEC_CHAIN_FORCED_EXIT_COST: u64 = Self::NOT_EXEC_CHAIN_FULL_EXIT_COST;
    pub const ORDER_MATCHING_COST: u64 = 71;
    /// Make sure we can contain at least one transaction
    pub const MIN_GAS: u64 = Self::BASE_COST + Self::WITHDRAWAL_GAS_LIMIT;

    pub fn base_cost() -> U256 {
        U256::from(Self::BASE_COST)
    }

    pub fn op_cost_with_exec_chain(op: &ZkLinkOp) -> U256 {
        let cost = match op {
            ZkLinkOp::Noop(_) => 0,
            ZkLinkOp::Deposit(_) => Self::DEPOSIT_COST,
            ZkLinkOp::ChangePubKeyOffchain(_) => Self::CHANGE_PUBKEY_COST,
            ZkLinkOp::Transfer(_) => Self::TRANSFER_COST,
            ZkLinkOp::TransferToNew(_) => Self::TRANSFER_TO_NEW_COST,
            ZkLinkOp::FullExit(_) => Self::FULL_EXIT_COST,
            ZkLinkOp::Withdraw(_) => Self::WITHDRAW_COST,
            ZkLinkOp::ForcedExit(_) => Self::FORCED_EXIT_COST,
            ZkLinkOp::OrderMatching(_) => Self::ORDER_MATCHING_COST,
        };

        U256::from(cost)
    }
}

/// `GasCounter` is an entity capable of counting the estimated gas cost of an
/// upcoming transaction. It watches for the total gas cost of either commit
/// or withdraw operation to not exceed the reasonable gas limit amount.
/// It is used by `state_keeper` module to seal the block once we're not able
/// to safely insert any more transactions.
///
/// The estimation process is based on the pre-calculated "base cost" of operation
/// (basically, cost of processing an empty block), and the added cost of all the
/// operations in that block.
///
/// These estimated costs were calculated using the `gas_price_test` from `testkit`.
#[derive(Debug, Clone)]
pub struct GasCounter {
    commit_cost: U256,
    verify_cost: U256,
}

impl Default for GasCounter {
    fn default() -> Self {
        Self {
            commit_cost: CommitCost::base_cost(),
            verify_cost: VerifyCost::base_cost(),
        }
    }
}

#[derive(Debug)]
pub struct WrongTransaction;

impl std::fmt::Display for WrongTransaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wrong transaction in gas counter")
    }
}

impl std::error::Error for WrongTransaction {}

impl GasCounter {

    pub fn new() -> Self {
        Self::default()
    }

    /// Adds the cost of the operation to the gas counter.
    ///
    /// Returns `Ok(())` if transaction fits, and returns `Err(())` if
    /// the block must be sealed without this transaction.
    pub fn try_add_op(&mut self, op: &ZkLinkOp, tx_gas_limit: u64) -> Result<(), WrongTransaction> {
        let new_commit_cost = self.commit_cost + CommitCost::op_cost_with_exec_chain(op);
        if new_commit_cost > U256::from(tx_gas_limit) {
            return Err(WrongTransaction);
        }

        let new_verify_cost = self.verify_cost + VerifyCost::op_cost_with_exec_chain(op);
        if new_verify_cost > U256::from(tx_gas_limit) {
            return Err(WrongTransaction);
        }

        Ok(())
    }

    pub fn add_op(&mut self, op: &ZkLinkOp) {
        self.commit_cost += CommitCost::op_cost_with_exec_chain(op);
        self.verify_cost += VerifyCost::op_cost_with_exec_chain(op);
    }

    pub fn commit_gas_limit(&self) -> U256 {
        self.commit_cost
    }

    pub fn verify_gas_limit(&self) -> U256 {
        self.verify_cost
    }
}

pub const BASE_COMMIT_BLOCKS_TX_COST: usize = 50240;
pub const BASE_EXECUTE_BLOCKS_TX_COST: usize = 44873;

/// Evil bridge cost is abount 61000
pub const EVIL_BRIDGE_BLOCKS_TX_COST: usize = 61000;
/// Test result is 47060, but really cost is abount 63000
pub const SYNC_BLOCKS_TX_COST: usize = 63000;
/// Prove block base cost is refer from zklink
pub const BASE_PROOF_BLOCKS_TX_COST: usize = 600000;
/// Use our test result to define prove cost per block
pub const PROVE_COST_PER_BLOCK: usize = 6759;

pub fn commit_gas_limit_aggregated(blocks: &[Block]) -> U256 {
    U256::from(BASE_COMMIT_BLOCKS_TX_COST)
        + blocks
        .iter()
        .fold(U256::zero(), |acc, block| acc + block.commit_gas_limit)
}

pub fn prove_gas_limit_aggregated(block_num: usize) -> U256 {
    U256::from(BASE_PROOF_BLOCKS_TX_COST) + U256::from(block_num * PROVE_COST_PER_BLOCK)
}

pub fn execute_gas_limit_aggregated(blocks: &[Block]) -> U256 {
    U256::from(BASE_EXECUTE_BLOCKS_TX_COST)
        + blocks
        .iter()
        .fold(U256::zero(), |acc, block| acc + block.verify_gas_limit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{operations::{
        ChangePubKeyOp, DepositOp, ForcedExitOp, FullExitOp, NoopOp, TransferOp,
        TransferToNewOp, WithdrawOp,
    }, tx::{ChangePubKey, ForcedExit, Transfer, Withdraw}, Deposit, FullExit};

    #[test]
    fn commit_and_verify_cost() {
        let change_pubkey_op = ChangePubKeyOp {
            tx: ChangePubKey::new(
                Default::default(),
                AccountId(1),
                Default::default(),
                Default::default(),
                TokenId(0),
                Default::default(),
                Default::default(),
                None,
                None,
                Default::default(),
            ),
            account_id: AccountId(1),
            address: Default::default()
        };
        let deposit_op = DepositOp {
            tx: Deposit {
                from_chain_id: Default::default(),
                from: Default::default(),
                sub_account_id: Default::default(),
                l2_target_token: TokenId(0),
                l1_source_token: Default::default(),
                amount: Default::default(),
                to: Default::default(),
                serial_id: Default::default(),
                eth_hash: Default::default(),
            },
            account_id: AccountId(1),
            l1_source_token_after_mapping: Default::default()
        };
        let transfer_op = TransferOp {
            tx: Transfer::new(
                AccountId(1),
                Default::default(),
                Default::default(),
                Default::default(),
                TokenId(0),
                Default::default(),
                Default::default(),
                Nonce(0),
                None,
                Default::default(),
            ),
            from: AccountId(1),
            to: AccountId(1),
        };
        let transfer_to_new_op = TransferToNewOp {
            tx: Transfer::new(
                AccountId(1),
                Default::default(),
                Default::default(),
                Default::default(),
                TokenId(0),
                Default::default(),
                Default::default(),
                Nonce(0),
                None,
                Default::default(),
            ),
            from: AccountId(1),
            to: AccountId(1),
        };
        let noop_op = NoopOp {};
        let full_exit_op = FullExitOp {
            tx: FullExit {
                to_chain_id: Default::default(),
                account_id: AccountId(0),
                sub_account_id: Default::default(),
                exit_address: Default::default(),
                l2_source_token: TokenId(0),
                l1_target_token: Default::default(),
                serial_id: 0,
                eth_hash: Default::default(),
            },
            exit_amount: Default::default(),
            l1_target_token_after_mapping: Default::default()
        };
        let forced_exit_op = ForcedExitOp {
            tx: ForcedExit::new(
                Default::default(),
                AccountId(1),
                Default::default(),
                Default::default(),
                Default::default(),
                TokenId(0),
                Default::default(),
                Default::default(),
                Default::default(),
                Nonce(0),
                None,
                Default::default()
            ),
            target_account_id: AccountId(1),
            withdraw_amount: 0u8.into(),
            l1_target_token_after_mapping: Default::default()
        };
        let withdraw_op = WithdrawOp {
            tx: Withdraw::new(
                AccountId(1),
                Default::default(),
                Default::default(),
                Default::default(),
                TokenId(0),
                Default::default(),
                Default::default(),
                Default::default(),
                Nonce(0),
                false,
                0,
                None,
                Default::default(),
            ),
            account_id: AccountId(1),
            l1_target_token_after_mapping: Default::default()
        };

        let test_vector_commit = vec![
            (
                ZkLinkOp::from(change_pubkey_op.clone()),
                CommitCost::CHANGE_PUBKEY_COST_ONCHAIN,
            ),
            (ZkLinkOp::from(deposit_op.clone()), CommitCost::DEPOSIT_COST),
            (
                ZkLinkOp::from(transfer_op.clone()),
                CommitCost::TRANSFER_COST,
            ),
            (
                ZkLinkOp::from(transfer_to_new_op.clone()),
                CommitCost::TRANSFER_TO_NEW_COST,
            ),
            (ZkLinkOp::from(noop_op.clone()), 0),
            (
                ZkLinkOp::from(full_exit_op.clone()),
                CommitCost::FULL_EXIT_COST,
            ),
            (
                ZkLinkOp::from(forced_exit_op.clone()),
                CommitCost::FORCED_EXIT_COST,
            ),
            (
                ZkLinkOp::from(withdraw_op.clone()),
                CommitCost::WITHDRAW_COST,
            ),
        ];
        let test_vector_verify = vec![
            (
                ZkLinkOp::from(change_pubkey_op),
                VerifyCost::CHANGE_PUBKEY_COST,
            ),
            (ZkLinkOp::from(deposit_op), VerifyCost::DEPOSIT_COST),
            (ZkLinkOp::from(transfer_op), VerifyCost::TRANSFER_COST),
            (
                ZkLinkOp::from(transfer_to_new_op),
                VerifyCost::TRANSFER_TO_NEW_COST,
            ),
            (ZkLinkOp::from(noop_op), 0),
            (ZkLinkOp::from(full_exit_op), VerifyCost::FULL_EXIT_COST),
            (ZkLinkOp::from(forced_exit_op), VerifyCost::FORCED_EXIT_COST),
            (ZkLinkOp::from(withdraw_op), VerifyCost::WITHDRAW_COST),
        ];

        for (op, expected_cost) in test_vector_commit {
            assert_eq!(CommitCost::op_cost_with_exec_chain(&op), U256::from(expected_cost));
        }
        for (op, expected_cost) in test_vector_verify {
            assert_eq!(VerifyCost::op_cost_with_exec_chain(&op), U256::from(expected_cost));
        }
    }

    #[test]
    fn gas_counter() {
        let change_pubkey_op = ChangePubKeyOp {
            tx: ChangePubKey::new(
                Default::default(),
                AccountId(1),
                Default::default(),
                Default::default(),
                TokenId(0),
                Default::default(),
                Default::default(),
                Default::default(),
                None,
                Default::default(),
            ),
            account_id: AccountId(1),
            address: Default::default()
        };
        let zklink_op = ZkLinkOp::from(change_pubkey_op);

        let mut gas_counter = GasCounter::new();

        assert_eq!(gas_counter.commit_cost, U256::from(CommitCost::BASE_COST));
        assert_eq!(gas_counter.verify_cost, U256::from(VerifyCost::BASE_COST));

        // Verify cost is 0, thus amount of operations is determined by the commit cost.
        let tx_gas_limit : u64 = 4_000_000;
        let amount_ops_in_block = U256::from(tx_gas_limit)
            - gas_counter.commit_gas_limit()
            / U256::from(CommitCost::CHANGE_PUBKEY_COST_ONCHAIN);

        for _ in 0..amount_ops_in_block.as_u64() {
            gas_counter.add_op(&zklink_op);
        }

        // Expected gas limit is (base_cost + n_ops * op_cost) * 1.3
        let expected_commit_limit = (U256::from(CommitCost::BASE_COST)
            + amount_ops_in_block * U256::from(CommitCost::CHANGE_PUBKEY_COST_ONCHAIN))
            * U256::from(130u16)
            / U256::from(100u16);
        let expected_verify_limit = (U256::from(VerifyCost::BASE_COST)
            + amount_ops_in_block * U256::from(VerifyCost::CHANGE_PUBKEY_COST))
            * U256::from(130u16)
            / U256::from(100u16);
        assert_eq!(gas_counter.commit_gas_limit(), expected_commit_limit);
        assert_eq!(gas_counter.verify_gas_limit(), expected_verify_limit);

        // Attempt to add one more operation (it should fail).
        gas_counter.add_op(&zklink_op);

        // Check again that limit has not changed.
        assert_eq!(gas_counter.commit_gas_limit(), expected_commit_limit);
        assert_eq!(gas_counter.verify_gas_limit(), expected_verify_limit);
    }
}
