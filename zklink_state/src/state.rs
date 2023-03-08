use std::collections::HashMap;
use anyhow::{ensure, Error, format_err};
use num::BigUint;
use zklink_types::{operations::{
    TransferOp, TransferToNewOp, ZkLinkOp, NoopOp
}, Account, AccountId, AccountMap, AccountTree, AccountUpdate, AccountUpdates, BlockNumber, ZkLinkPriorityOp, ZkLinkTx, SlotId, ChainId, Token, SubAccountId, TokenId, ZkLinkAddress, PubKeyHash};
use zklink_crypto::params::{self, MAIN_SUB_ACCOUNT_ID, FEE_ACCOUNT_ID, MAX_ACCOUNT_ID};
use zklink_types::account::TidyOrder;
use zklink_types::utils::{calculate_actual_slot, calculate_actual_token};
use zklink_crypto::bellman::bn256::Fr;
use crate::handler::TxHandler;

#[derive(Debug)]
pub struct OpSuccess {
    pub updates: AccountUpdates,
    pub executed_op: ZkLinkOp,
}

#[derive(Debug, Clone)]
pub struct ZkLinkState {
    /// Accounts stored in a sparse Merkle tree
    balance_tree: AccountTree,
    account_id_by_address: HashMap<ZkLinkAddress, AccountId>,

    /// Current block number
    pub block_number: BlockNumber,

    /// tokens list
    pub token_by_id: HashMap<TokenId, Token>,
}

/// Helper enum to unify Transfer / TransferToNew operations.
#[derive(Debug)]
pub enum TransferOutcome {
    Transfer(TransferOp),
    TransferToNew(TransferToNewOp),
}

impl TransferOutcome {
    pub fn into_franklin_op(self) -> ZkLinkOp {
        match self {
            Self::Transfer(transfer) => transfer.into(),
            Self::TransferToNew(transfer) => transfer.into(),
        }
    }
}

impl From<TransferOutcome> for ZkLinkOp {
    fn from(op: TransferOutcome) -> Self {
        match op{
            TransferOutcome::Transfer(transfer) => transfer.into(),
            TransferOutcome::TransferToNew(transfer) => transfer.into(),
        }
    }
}

impl ZkLinkState {
    pub fn empty() -> Self {
        let tree_depth = params::account_tree_depth();
        let balance_tree = AccountTree::new(tree_depth);
        Self {
            balance_tree,
            block_number: BlockNumber(0),
            account_id_by_address: HashMap::new(),
            token_by_id: HashMap::new(),
        }
    }

    pub fn from_acc_map(accounts: AccountMap, current_block: BlockNumber) -> Self {
        let mut empty = Self::empty();
        empty.block_number = current_block;
        for (id, account) in accounts {
            empty.insert_account(id, account);
        }
        empty
    }

    pub fn new(
        balance_tree: AccountTree,
        account_id_by_address: HashMap<ZkLinkAddress, AccountId>,
        current_block: BlockNumber,
        token_by_id: HashMap<TokenId, Token>,
    ) -> Self {
        Self {
            balance_tree,
            block_number: current_block,
            account_id_by_address,
            token_by_id,
        }
    }


    pub fn register_token(&mut self, token: Token) {
       self.token_by_id.insert(token.id, token);
    }

    pub fn ensure_token_supported(&self, token_id: &TokenId) -> Result<(), Error>{
        anyhow::ensure!(
                self.is_token_supported(token_id),
                "Token {:?} does not exist.",
                token_id
            );
        Ok(())
    }

    pub fn ensure_token_of_chain_supported(&self, token_id: &TokenId, chain_id: &ChainId) -> Result<(), Error>{
        anyhow::ensure!(
                self.is_token_of_chain_supported(token_id, chain_id),
                "Token {:?} does not exist on target chain {:?}.",
                token_id, chain_id
            );
        Ok(())
    }

    pub fn assert_token_supported(&self, token_id: &TokenId) -> () {
        assert!(
            self.is_token_supported(token_id),
            "Token {:?} does not exist.",
            token_id
        );
    }

    pub fn assert_token_of_chain_supported(&self, token_id: &TokenId, chain_id: &ChainId) -> () {
        assert!(
            self.is_token_of_chain_supported(token_id, chain_id),
            "Token {:?} does not exist on target chain {:?}.",
            token_id, chain_id
            );
    }

    pub fn check_token_supported(&self, token: &TokenId, chain_id: &ChainId) -> Result<(), Error>{
        if let Some(token) = self.token_by_id.get(&token) {
            anyhow::ensure!(
                token.chains.contains(&chain_id),
                "Token {:?} does not exist on target chain {:?}.",
                token, chain_id
            );
            Ok(())
        } else {
            Err(format_err!("Token unsupported."))
        }
    }

    pub fn is_token_supported(&self, token_id: &TokenId) -> bool {
        self.token_by_id.contains_key(token_id)
    }

    pub fn is_token_of_chain_supported(&self, token_id: &TokenId, chain_id: &ChainId) -> bool {
        if let Some(token) = self.token_by_id.get(&token_id) {
            token.chains.contains(&chain_id)
        } else {
            false
        }
    }

    pub fn ensure_account_active_and_tx_pk_correct(&self, account_id: AccountId, tx_pk: PubKeyHash) -> Result<Account, Error> {
        let account = self
            .get_account(account_id)
            .ok_or_else(|| format_err!("Account does not exist"))?;

        // Account pub key hash need to recheck
        ensure!(
            account.pub_key_hash != PubKeyHash::default(),
            "Account not active"
        );
        ensure!(
            account.pub_key_hash == tx_pk,
            "Account pub key hash not match"
        );

        Ok(account)
    }

    pub fn get_accounts(&self) -> Vec<(u32, Account)> {
        self.balance_tree
            .items
            .iter()
            .map(|a| (*a.0 as u32, a.1.clone()))
            .collect()
    }

    pub fn collect_fee(&mut self, token: TokenId, fee: &BigUint, updates: &mut Vec<(AccountId, AccountUpdate)>){
        let mut fee_account = self.get_account(FEE_ACCOUNT_ID).unwrap();
        // collect fee to MAIN_SUB_ACCOUNT_ID
        let sub_account_id = MAIN_SUB_ACCOUNT_ID;
        let actual_token = Self::get_actual_token_by_sub_account(sub_account_id, token);
        let old_amount = fee_account.get_balance(actual_token);
        fee_account.add_balance(actual_token, fee);
        let new_amount = fee_account.get_balance(actual_token);
        updates.push((
            FEE_ACCOUNT_ID,
            AccountUpdate::UpdateBalance {
                balance_update: (token, sub_account_id, old_amount, new_amount),
                old_nonce: fee_account.nonce,
                new_nonce: fee_account.nonce,
            },
        ));
        self.insert_account(FEE_ACCOUNT_ID, fee_account);
    }

    pub fn get_actual_slot(sub_account_id: SubAccountId, slot_id: SlotId) -> SlotId{
        calculate_actual_slot(sub_account_id, slot_id)
    }

    pub fn get_actual_token_by_sub_account(sub_account_id: SubAccountId, token_id: TokenId) -> TokenId{
        calculate_actual_token(sub_account_id, token_id)
    }

    // This is mainly used for the global asset tree
    pub fn get_actual_token_by_chain(chain_id: ChainId, token_id: TokenId) -> TokenId {
        calculate_actual_token(SubAccountId(*chain_id), token_id)
    }

    pub fn root_hash(&self) -> Fr {
        // let start = std::time::Instant::now();
        let hash = self.balance_tree.root_hash();
        // metrics::histogram!("root_hash", start.elapsed());
        hash
    }

    pub fn get_account(&self, account_id: AccountId) -> Option<Account> {
        let start = std::time::Instant::now();

        let account = self.balance_tree.get(*account_id).cloned();

        account
    }

    pub fn chunks_for_tx(&self, franklin_tx: &ZkLinkTx) -> usize {
        match franklin_tx {
            ZkLinkTx::Transfer(tx) => {
                if self.get_account_by_address(&tx.to).is_some() {
                    TransferOp::CHUNKS
                } else {
                    TransferToNewOp::CHUNKS
                }
            }
            _ => franklin_tx.min_chunks(),
        }
    }

    /// Priority op execution should not fail.
    pub fn execute_priority_op(&mut self, op: ZkLinkPriorityOp) -> OpSuccess {
        match op {
            _ => OpSuccess{
                updates: vec![],
                executed_op: ZkLinkOp::Noop(NoopOp{}),
            },
        }
    }

    /// Applies account updates.
    /// Assumes that all updates are correct, panics otherwise.
    pub fn apply_account_updates(&mut self, updates: AccountUpdates) {
        for (account_id, account_update) in updates {
            match account_update {
                AccountUpdate::Create { address, nonce } => {
                    assert!(self.get_account_by_address(&address).is_none());

                    let mut account = Account::default_with_address(&address);
                    account.nonce = nonce;
                    self.insert_account(account_id, account);
                }
                AccountUpdate::Delete { address, nonce } => {
                    let account = self
                        .get_account(account_id)
                        .expect("account to delete must exist");
                    assert_eq!(account.address, address);
                    assert_eq!(account.nonce, nonce);

                    self.remove_account(account_id);
                }
                AccountUpdate::UpdateBalance {
                    old_nonce,
                    new_nonce,
                    balance_update: (token_id, sub_account_id, old_balance, new_balance),
                } => {
                    let mut account = self
                        .get_account(account_id)
                        .expect("account to update balance must exist");
                    let real_token = calculate_actual_token(sub_account_id, token_id);
                    assert_eq!(account.get_balance(real_token), old_balance);
                    assert_eq!(account.nonce, old_nonce);

                    account.set_balance(real_token, new_balance.clone());
                    account.nonce = new_nonce;
                    self.insert_account(account_id, account);
                }
                AccountUpdate::ChangePubKeyHash {
                    old_pub_key_hash,
                    new_pub_key_hash,
                    old_nonce,
                    new_nonce,
                } => {
                    let mut account = self
                        .get_account(account_id)
                        .expect("account to change pubkey must exist");
                    assert_eq!(account.pub_key_hash, old_pub_key_hash);
                    assert_eq!(account.nonce, old_nonce);

                    account.pub_key_hash = new_pub_key_hash;
                    account.nonce = new_nonce;
                    self.insert_account(account_id, account);
                }
                AccountUpdate::UpdateTidyOrder {
                    slot_id,
                    sub_account_id,
                    old_order: _old_order_nonce,
                    new_order: new_order_nonce
                } => {
                    let mut account = self
                        .get_account(account_id)
                        .expect("account to update order nonce must exist");
                    // assert_eq!(old_order_nonce.0, account.order_slots[&slot_id].nonce);
                    let slot_id = calculate_actual_slot(sub_account_id,slot_id);
                    account.order_slots.insert(
                        slot_id,
                        TidyOrder{
                            nonce: new_order_nonce.0,
                            residue: new_order_nonce.1.into(),
                        });
                    self.insert_account(account_id, account);
                }
            }
        }
    }

    pub fn execute_tx(&mut self, tx: ZkLinkTx) -> Result<OpSuccess, Error> {
        match tx {
            ZkLinkTx::Deposit(tx) => self.apply_tx(*tx),
            ZkLinkTx::FullExit(tx) => self.apply_tx(*tx),
            ZkLinkTx::Transfer(tx) => self.apply_tx(*tx),
            ZkLinkTx::Withdraw(tx) => self.apply_tx(*tx),
            ZkLinkTx::ChangePubKey(tx) => self.apply_tx(*tx),
            ZkLinkTx::ForcedExit(tx) => self.apply_tx(*tx),
            ZkLinkTx::OrderMatching(tx) => self.apply_tx(*tx),
        }
    }

    pub fn get_free_account_id(&self) -> AccountId {
        let mut account_id = AccountId(self.balance_tree.items.len() as u32);

        // In the production database it somehow appeared that one account ID in the database got missing,
        // meaning that it was never assigned, but the next one was inserted.
        // This led to the fact that length of the tree is not equal to the most recent ID anymore.
        // In order to prevent similar error-proneness in the future, we scan until we find the next free ID.
        // Amount of steps here is not expected to be high.
        while self.get_account(account_id).is_some() {
            *account_id += 1;
        }

        assert!(account_id <= MAX_ACCOUNT_ID, "No more free account");

        account_id
    }

    pub fn get_account_by_address(&self, address: &ZkLinkAddress) -> Option<(AccountId, Account)> {
        let account_id = *self.account_id_by_address.get(address)?;
        Some((
            account_id,
            self.get_account(account_id)
                .expect("Failed to get account by cached pubkey"),
        ))
    }

    #[doc(hidden)] // Public for benches.
    pub fn insert_account(&mut self, id: AccountId, account: Account) {
        self.account_id_by_address.insert(account.address.clone(), id);
        self.balance_tree.insert(*id, account);
    }

    #[allow(dead_code)]
    pub(crate) fn remove_account(&mut self, id: AccountId) {
        if let Some(account) = self.get_account(id) {
            self.account_id_by_address.remove(&account.address);
            self.balance_tree.remove(*id);
        }
    }

    /// Converts the `ZkLinkTx` object to a `ZkLinkOp`, without applying it.
    pub fn tx_to_op(&self, tx: ZkLinkTx) -> Result<ZkLinkOp, Error> {
        match tx {
            ZkLinkTx::Deposit(tx) => self.create_op(*tx).map(Into::into),
            ZkLinkTx::FullExit(tx) => self.create_op(*tx).map(Into::into),
            ZkLinkTx::Transfer(tx) => self.create_op(*tx).map(From::from),
            ZkLinkTx::Withdraw(tx) => self.create_op(*tx).map(Into::into),
            ZkLinkTx::ChangePubKey(tx) => self.create_op(*tx).map(Into::into),
            ZkLinkTx::ForcedExit(tx) => self.create_op(*tx).map(Into::into),
            ZkLinkTx::OrderMatching(tx) => self.create_op(*tx).map(Into::into),
        }
    }

    /// Converts the `PriorityOp` object to a `ZkLinkOp`, without applying it.
    pub fn priority_op_to_zklink_op(&self, op: ZkLinkPriorityOp) -> ZkLinkOp {
        match op {
            // ZkLinkPriorityOp::Deposit(op) => self.create_op(op).unwrap().into(),
            ZkLinkPriorityOp::FullExit(_) => ZkLinkOp::Noop(NoopOp{}),
            ZkLinkPriorityOp::Deposit(_) => ZkLinkOp::Noop(NoopOp{}),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn apply_updates(&mut self, updates: &[(AccountId, AccountUpdate)]) {
        for (account_id, update) in updates {
            match update {
                AccountUpdate::Create { address, nonce } => {
                    let (mut account, _) = Account::create_account(*account_id, (*address).clone());
                    account.nonce = *nonce;
                    self.insert_account(*account_id, account);
                }
                AccountUpdate::Delete { address, nonce } => {
                    let account = self
                        .get_account(*account_id)
                        .expect("account doesn't exist");
                    assert_eq!(&account.address, address);
                    assert_eq!(&account.nonce, nonce);
                    self.remove_account(*account_id)
                }
                AccountUpdate::UpdateBalance {
                    old_nonce,
                    new_nonce,
                    balance_update,
                } => {
                    let mut account = self
                        .get_account(*account_id)
                        .expect("account doesn't exist");

                    let (token_id, sub_account_id, old_amount, new_amount) = balance_update;
                    let real_token = calculate_actual_token(*sub_account_id, *token_id);

                    assert_eq!(account.nonce, *old_nonce, "nonce mismatch");
                    assert_eq!(
                        &account.get_balance(*token_id),
                        old_amount,
                        "balance mismatch"
                    );
                    account.nonce = *new_nonce;
                    account.set_balance(real_token, new_amount.clone());

                    self.insert_account(*account_id, account);
                }
                AccountUpdate::ChangePubKeyHash {
                    old_pub_key_hash,
                    new_pub_key_hash,
                    old_nonce,
                    new_nonce,
                } => {
                    let mut account = self
                        .get_account(*account_id)
                        .expect("account doesn't exist");

                    assert_eq!(
                        &account.pub_key_hash, old_pub_key_hash,
                        "pub_key_hash mismatch"
                    );
                    assert_eq!(&account.nonce, old_nonce, "nonce mismatch");

                    account.pub_key_hash = *new_pub_key_hash;
                    account.nonce = *new_nonce;

                    self.insert_account(*account_id, account);
                }
                AccountUpdate::UpdateTidyOrder {
                    slot_id,
                    sub_account_id,
                    new_order: new_order_nonce, ..
                } => {
                    let mut account = self
                        .get_account(*account_id)
                        .expect("account doesn't exist");
                    let slot_id = calculate_actual_slot(*sub_account_id, *slot_id);
                    account.order_slots.insert(
                        slot_id,
                        TidyOrder{
                            nonce: new_order_nonce.0.clone(),
                            residue: new_order_nonce.1.clone().into(),
                        }
                    );
                }
            }
        }
    }


    pub fn get_balance_tree(&self) -> AccountTree {
        self.balance_tree.clone()
    }

    pub fn get_account_addresses(&self) -> HashMap<ZkLinkAddress, AccountId> {
        self.account_id_by_address.clone()
    }
}