use std::collections::HashMap;
use num::{BigUint, Zero};
use serde::{Deserialize, Serialize};
use zklink_crypto::primitives::GetBits;
use zklink_utils::BigUintSerdeWrapper;
use zklink_crypto::franklin_crypto::bellman::pairing::ff::PrimeField;
use zklink_crypto::params::{MAX_SLOT_ID, MAX_TOKEN_ID, total_slots};
use zklink_basic_types::SlotId;
use zklink_crypto::circuit::account::{Balance, CircuitAccount, CircuitTidyOrder};
use zklink_crypto::convert::FeConvert;
use super::{Order, AccountId, Fr, AccountUpdates, Nonce, TokenId};
use crate::utils::{calculate_actual_slot, calculate_actual_token};
pub use self::{account_update::AccountUpdate, pubkey_hash::PubKeyHash};
use crate::ZkLinkAddress;

mod account_update;
mod pubkey_hash;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BalanceNode {
    pub reserve0: BigUintSerdeWrapper,
}

impl BalanceNode {
    pub fn is_zero(&self) -> bool {
        self.reserve0.0.is_zero()
    }
}

impl PartialEq for BalanceNode {
    fn eq(&self, other: &BalanceNode) -> bool {
        self.reserve0.eq(&other.reserve0)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TidyOrder {
    /// Slot nonce
    pub nonce: Nonce,
    /// Residue amount of base token
    pub residue: BigUintSerdeWrapper,
}

impl TidyOrder {
    pub fn update(&mut self, actual_exchanged: &BigUint, order: &Order){
        if self.residue.is_zero() || order.nonce > self.nonce {
            self.residue.0 = order.amount.clone();
            if order.nonce > self.nonce{
                self.nonce = order.nonce;
            }
        }
        self.residue.0 -= actual_exchanged;
        if self.residue.is_zero(){ *self.nonce += 1; }
    }
}

/// zklink network account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Hash of the account public key used to authorize operations for this account.
    /// Once account is created (e.g. by `Transfer` or `Deposit` operation), account owner
    /// has to set its public key hash via `ChangePubKey` transaction, so the server will be
    /// able to verify owner's identity when processing account transactions.
    pub pub_key_hash: PubKeyHash,
    /// Address of the account. Directly corresponds to the L1 address.
    pub address: ZkLinkAddress,
    balances: HashMap<TokenId, BalanceNode>,
    /// Current nonce of the account. All the transactions require nonce field to be set in
    /// order to not allow double spend, and the nonce must increment by one after each operation.
    pub nonce: Nonce,
    /// Current nonce and residue of all order slots.
    pub order_slots: HashMap<SlotId, TidyOrder>
}

impl PartialEq for Account {
    fn eq(&self, other: &Account) -> bool {
        self.get_bits_le().eq(&other.get_bits_le())
    }
}

impl From<Account> for CircuitAccount<super::Engine> {
    fn from(acc: Account) -> Self {
        let mut circuit_account = CircuitAccount::default();

        let balances: Vec<_> = acc
            .balances
            .iter()
            .map(|(id, b)| {
                (
                    *id,
                    Balance { value: Fr::from_big_uint(b.reserve0.0.clone()).unwrap() }
                )
            })
            .collect();

        for (i, b) in balances.into_iter() {
            circuit_account.subtree.insert(*i, b);
        }
        let orders: Vec<_> = acc
            .order_slots
            .iter()
            .map(|(id, b)| {
                (
                    *id,
                    CircuitTidyOrder{
                        nonce: Fr::from_repr((*b.nonce as u64).into()).unwrap(),
                        residue: Fr::from_big_uint(b.residue.0.clone()).unwrap(),
                    }
                )
            })
            .collect();

        for (i, b) in orders.into_iter() {
            circuit_account.order_tree.insert(*i, b);
        }
        circuit_account.nonce = Fr::from_str(&acc.nonce.to_string()).unwrap();
        circuit_account.pub_key_hash = acc.pub_key_hash.to_fr();
        circuit_account.address = acc.address.convert_to_frs();
        circuit_account
    }
}

impl Default for Account {
    fn default() -> Self {
        Account {
            balances: HashMap::new(),
            nonce: Nonce(0),
            pub_key_hash: PubKeyHash::default(),
            address: ZkLinkAddress::from(vec![0;32]),
            order_slots: HashMap::new()
        }
    }
}

impl GetBits for Account {
    fn get_bits_le(&self) -> Vec<bool> {
        CircuitAccount::<super::Engine>::from(self.clone()).get_bits_le()
    }
}

impl Account {
    /// Creates a new empty account object, and sets its address.
    pub fn default_with_address(address: &ZkLinkAddress) -> Account {
        Account {
            address: (*address).clone(),
            ..Default::default()
        }
    }

    /// Creates a new account object and the list of updates that has to be applied on the state
    /// in order to get this account created within the network.
    pub fn create_account(id: AccountId, address: ZkLinkAddress) -> (Account, AccountUpdates) {
        let account = Account::default_with_address(&address);
        let updates = vec![(
            id,
            AccountUpdate::Create {
                address: account.address.clone(),
                nonce: account.nonce,
            },
        )];
        (account, updates)
    }

    /// Returns the token balance for the account.
    pub fn get_balance(&self, token: TokenId) -> BigUint {
        assert!(token < MAX_TOKEN_ID);
        let node = self.balances.get(&token).cloned().unwrap_or_default();
        node.reserve0.0
    }

    /// Returns the order of the special slot for the account.
    pub fn get_order(&self, slot: SlotId) -> TidyOrder {
        assert!(slot < MAX_SLOT_ID);
        let node = self.order_slots.get(&slot).cloned().unwrap_or_default();
        node
    }

    /// Overrides the token balance value.
    pub fn set_balance(&mut self, token: TokenId, amount: BigUint) {
        assert!(token < MAX_TOKEN_ID);
        let node = BalanceNode{ reserve0: BigUintSerdeWrapper(amount) };
        self.balances.insert(token, node);
    }

    pub fn set_order(&mut self, slot: SlotId, nonce: Nonce, residue: BigUint) {
        assert!((*slot as usize) < total_slots());
        let order = TidyOrder {
            nonce,
            residue: BigUintSerdeWrapper(residue),
        };
        self.order_slots.insert(slot, order);
    }

    /// Adds the provided amount to the token balance.
    pub fn add_balance(&mut self, token: TokenId, amount: &BigUint) {
        assert!(token < MAX_TOKEN_ID);
        let mut balance_node = self.balances.remove(&token).unwrap_or_default();
        balance_node.reserve0.0 += amount;
        self.balances.insert(token, balance_node);
    }

    /// Subtracts the provided amount from the token balance.
    ///
    /// # Panics
    ///
    /// Panics if the amount to subtract is greater than the existing token balance.
    pub fn sub_balance(&mut self, token: TokenId, amount: &BigUint) {
        assert!(token < MAX_TOKEN_ID);
        let mut balance_node = self.balances.remove(&token).unwrap_or_default();
        balance_node.reserve0.0 -= amount;
        self.balances.insert(token, balance_node);
    }

    /// Given the list of updates to apply, changes the account state.
    pub fn apply_updates(mut account: Option<Self>, updates: &[AccountUpdate]) -> Option<Self> {
        for update in updates {
            account = Account::apply_update(account, update.clone());
        }
        account
    }

    /// Applies an update to the account state.
    pub fn apply_update(account: Option<Self>, update: AccountUpdate) -> Option<Self> {
        match account {
            Some(mut account) => match update {
                AccountUpdate::Delete { .. } => None,
                AccountUpdate::UpdateBalance {
                    balance_update: (token, sub_account_id,_, amount),
                    new_nonce,
                    ..
                } => {
                    let real_token= calculate_actual_token(sub_account_id, token);
                    account.set_balance(real_token, amount);
                    account.nonce = new_nonce;
                    Some(account)
                }
                AccountUpdate::ChangePubKeyHash {
                    new_pub_key_hash,
                    new_nonce,
                    ..
                } => {
                    account.pub_key_hash = new_pub_key_hash;
                    account.nonce = new_nonce;
                    Some(account)
                }
                AccountUpdate::UpdateTidyOrder {
                    slot_id,
                    sub_account_id,
                    new_order: new_order_nonce, ..
                } => {
                    let slot_id = calculate_actual_slot(sub_account_id, slot_id);
                    account.order_slots.insert(
                        slot_id,
                        TidyOrder{
                            nonce: new_order_nonce.0,
                            residue: new_order_nonce.1.into(),
                        });
                    Some(account)
                }
                _ => {
                    tracing::error!(
                        "Incorrect update received {:?} for account {:?}",
                        update,
                        account
                    );
                    Some(account)
                }
            },
            None => match update {
                AccountUpdate::Create { address, nonce, .. } => Some(Account {
                    address,
                    nonce,
                    ..Default::default()
                }),
                _ => {
                    tracing::error!("Incorrect update received {:?} for empty account", update);
                    None
                }
            },
        }
    }

    /// Returns all the nonzero token balances for the account.
    pub fn get_nonzero_balances(&self) -> HashMap<TokenId, BalanceNode> {
        let mut balances = self.balances.clone();
        balances.retain(|_, v| !(v.is_zero()));
        balances
    }

    /// Returns all the nonzero token balances for the account.
    pub fn get_existing_token_balances(&self) -> &HashMap<TokenId, BalanceNode> {
        &self.balances
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        helpers::{apply_updates, reverse_updates},
        AccountMap, AccountUpdates,SubAccountId
    };
    use crate::helpers::{pack_token_amount, unpack_token_amount};

    #[test]
    fn test_default_account() {
        let a = Account::default();
        a.get_bits_le();
    }

    #[test]
    fn test_amount_pack() {
        let amount = BigUint::from(10u32).pow(18u32);
        dbg!(amount.clone());
        let packed = pack_token_amount(&BigUint::from(amount));
        dbg!(packed.clone());
        let unpacked = unpack_token_amount(&packed).unwrap();
        dbg!(unpacked);
        // let bu = BigUint::from(amount);
        // dbg!(bu.clone());
        // dbg!(bu.to_u128().unwrap());
    }

    #[test]
    fn test_account_update() {
        let create = AccountUpdate::Create {
            address: ZkLinkAddress::default(),
            nonce: Nonce(1),
        };

        let bal_update = AccountUpdate::UpdateBalance {
            old_nonce: Nonce(1),
            new_nonce: Nonce(2),
            balance_update: (TokenId(0), SubAccountId(0), 0u32.into(), 5u32.into()),
        };

        let delete = AccountUpdate::Delete {
            address: ZkLinkAddress::default(),
            nonce: Nonce(2),
        };

        {
            {
                let created_account = Account {
                    nonce: Nonce(1),
                    ..Default::default()
                };
                assert_eq!(
                    Account::apply_update(None, create.clone())
                        .unwrap()
                        .get_bits_le(),
                    created_account.get_bits_le()
                );
            }

            assert!(Account::apply_update(None, bal_update.clone()).is_none());
            assert!(Account::apply_update(None, delete.clone()).is_none());
        }
        {
            assert_eq!(
                Account::apply_update(Some(Account::default()), create)
                    .unwrap()
                    .get_bits_le(),
                Account::default().get_bits_le()
            );
            {
                let mut updated_account = Account {
                    nonce: Nonce(2),
                    ..Default::default()
                };
                updated_account.set_balance(TokenId(0), 5u32.into());
                assert_eq!(
                    Account::apply_update(Some(Account::default()), bal_update)
                        .unwrap()
                        .get_bits_le(),
                    updated_account.get_bits_le()
                );
            }
            assert!(Account::apply_update(Some(Account::default()), delete).is_none());
        }
    }

    #[test]
    fn test_account_updates() {
        // Create two accounts: 0, 1
        // In updates -> delete 0, update balance of 1, create account 2
        // Reverse updates

        let account_map_initial = {
            let mut map = AccountMap::default();
            let account_0 = Account {
                nonce: Nonce(8),
                ..Default::default()
            };
            let account_1 = Account {
                nonce: Nonce(16),
                ..Default::default()
            };
            map.insert(AccountId(0), account_0);
            map.insert(AccountId(1), account_1);
            map
        };

        let account_map_updated_expected = {
            let mut map = AccountMap::default();
            let mut account_1 = Account {
                nonce: Nonce(17),
                ..Default::default()
            };
            account_1.set_balance(TokenId(0), 256u32.into());
            map.insert(AccountId(1), account_1);
            let account_2 = Account {
                nonce: Nonce(36),
                ..Default::default()
            };
            map.insert(AccountId(2), account_2);
            map
        };

        let updates = {
            let mut updates = AccountUpdates::new();
            updates.push((
                AccountId(0),
                AccountUpdate::Delete {
                    address: ZkLinkAddress::default(),
                    nonce: Nonce(8),
                },
            ));
            updates.push((
                AccountId(1),
                AccountUpdate::UpdateBalance {
                    old_nonce: Nonce(16),
                    new_nonce: Nonce(17),
                    balance_update: (TokenId(0), SubAccountId(0), 0u32.into(), 256u32.into()),
                },
            ));
            updates.push((
                AccountId(2),
                AccountUpdate::Create {
                    address: ZkLinkAddress::default(),
                    nonce: Nonce(36),
                },
            ));
            updates
        };

        let account_map_updated = {
            let mut map = account_map_initial.clone();
            apply_updates(&mut map, updates.clone());
            map
        };

        assert_eq!(account_map_updated, account_map_updated_expected);

        let account_map_updated_back = {
            let mut map = account_map_updated;
            let mut reversed = updates;
            reverse_updates(&mut reversed);
            apply_updates(&mut map, reversed);
            map
        };

        assert_eq!(account_map_updated_back, account_map_initial);
    }
}
