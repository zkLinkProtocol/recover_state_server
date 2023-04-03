use lazy_static::lazy_static;

use franklin_crypto::bellman::pairing::ff::{Field, PrimeField};
use franklin_crypto::bellman::pairing::Engine;
use franklin_crypto::bellman::pairing::bn256::{Bn256, Fr};
use franklin_crypto::rescue::RescueEngine;
use crate::merkle_tree::hasher::Hasher;
use crate::merkle_tree::{RescueHasher, SparseMerkleTree};
use crate::primitives::{GetBits, GetBitsFixed};
use crate::convert::FeConvert;
use crate::params;

/// Account tree used in the `zklink_circuit`.
pub type CircuitAccountTree = SparseMerkleTree<CircuitAccount<Bn256>, Fr, RescueHasher<Bn256>>;
/// Balance tree for accounts used in the `zklink_circuit`.
pub type CircuitBalanceTree = SparseMerkleTree<Balance<Bn256>, Fr, RescueHasher<Bn256>>;
/// Order tree for accounts used in the `zklink_circuit`.
pub type CircuitOrderTree = SparseMerkleTree<CircuitTidyOrder<Bn256>, Fr, RescueHasher<Bn256>>;

pub fn empty_account_as_field_elements<E: Engine>() -> Vec<E::Fr> {
    let acc = CircuitAccount::<Bn256>::default();
    let bits = acc.get_bits_le();

    use crate::franklin_crypto::circuit::multipack;

    multipack::compute_multipacking::<E>(&bits)
}

/// Representation of the zklink account used in the `zklink_circuit`.
#[derive(Clone)]
pub struct CircuitAccount<E: RescueEngine> {
    pub subtree: SparseMerkleTree<Balance<E>, E::Fr, RescueHasher<E>>,
    pub nonce: E::Fr,
    pub pub_key_hash: E::Fr,
    pub address: E::Fr,
    pub order_tree: SparseMerkleTree<CircuitTidyOrder<E>, E::Fr, RescueHasher<E>>
}

impl<E: RescueEngine> GetBits for CircuitAccount<E> {
    fn get_bits_le(&self) -> Vec<bool> {
        debug_assert_eq!(
            params::FR_BIT_WIDTH,
            E::Fr::NUM_BITS as usize,
            "FR bit width is not equal to field bit width"
        );
        let mut leaf_content = Vec::new();

        leaf_content.extend(self.nonce.get_bits_le_fixed(params::NONCE_BIT_WIDTH));
        leaf_content.extend(
            self.pub_key_hash.get_bits_le_fixed(params::NEW_PUBKEY_HASH_WIDTH));
        leaf_content.extend(
            self.address.get_bits_le_fixed(params::ETH_ADDRESS_BIT_WIDTH));

        { // calculate hash of the subroot using algebraic hash
            let state_root = self.get_state_root();
            let mut state_tree_hash_bits = state_root.get_bits_le_fixed(params::FR_BIT_WIDTH);
            state_tree_hash_bits.resize(params::FR_BIT_WIDTH_PADDED, false);
            leaf_content.extend(state_tree_hash_bits.into_iter());
        }

        { // calculate hash of the OrderTree using algebraic hash
            let order_root = self.get_order_root();
            let mut order_tree_hash_bits = order_root.get_bits_le_fixed(params::FR_BIT_WIDTH);
            order_tree_hash_bits.resize(params::FR_BIT_WIDTH_PADDED, false);
            leaf_content.extend(order_tree_hash_bits.into_iter());
        }
        assert_eq!(
            leaf_content.len(),
            params::LEAF_DATA_BIT_WIDTH + params::FR_BIT_WIDTH_PADDED + params::FR_BIT_WIDTH_PADDED,
            "Account bit width mismatch"
        );

        leaf_content
    }
}

impl<E: RescueEngine> CircuitAccount<E> {
    fn get_state_root(&self) -> E::Fr {
        let balance_root = self.subtree.root_hash();
        let state_root_padding = E::Fr::zero();
        self.subtree
            .hasher
            .hash_elements(vec![balance_root, state_root_padding])
    }

    fn get_order_root(&self) -> E::Fr {
        let order_root = self.order_tree.root_hash();
        let order_root_padding = E::Fr::zero();
        self.order_tree
            .hasher
            .hash_elements(vec![order_root, order_root_padding])
    }
}

impl std::default::Default for CircuitAccount<Bn256> {
    //default should be changed: since subtree_root_hash is not zero for all zero balances and subaccounts
    fn default() -> Self {
        Self {
            nonce: Fr::zero(),
            pub_key_hash: Fr::zero(),
            address: Fr::zero(),
            subtree: BALANCE_TREE.clone(),
            order_tree: ORDER_TREE.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CircuitTidyOrder<E: Engine> {
    pub nonce: E::Fr,
    pub residue: E::Fr,
}

impl<E: Engine> Default for CircuitTidyOrder<E>  {
    fn default() -> Self {
        CircuitTidyOrder{
            nonce: E::Fr::zero(),
            residue: E::Fr::zero(),
        }
    }
}

impl<E: Engine> GetBits for CircuitTidyOrder<E> {
    fn get_bits_le(&self) -> Vec<bool> {
        let mut content = Vec::new();
        content.extend(self.nonce.get_bits_le_fixed(params::ORDER_NONCE_BIT_WIDTH));
        content.extend(self.residue.get_bits_le_fixed(params::BALANCE_BIT_WIDTH));
        content
    }
}

impl<E: Engine> CircuitTidyOrder<E> {
    pub fn update(&mut self,actual_exchanged: E::Fr, (amount, nonce): (E::Fr, u32)){
        let is_refresh_order = nonce > self.nonce.into_usize() as u32;

        if self.residue == E::Fr::zero() ||  is_refresh_order{
            self.residue = amount;
            if is_refresh_order{
                self.nonce = E::Fr::from_u64(nonce as u64);
            }
        }
        self.residue.sub_assign(&actual_exchanged);
        if self.residue.is_zero(){
            assert_ne!(self.nonce, E::Fr::from_u64(*params::MAX_NONCE as u64));
            self.nonce.add_assign(&E::Fr::one());
        }
    }
}

/// Representation of one token balance used in `zklink_circuit`.
#[derive(Clone, Debug)]
pub struct Balance<E: Engine> {
    pub value: E::Fr, // value 0, for compilation
}

impl<E: Engine> GetBits for Balance<E> {
    fn get_bits_le(&self) -> Vec<bool> {
        let mut leaf_content = Vec::new();
        leaf_content.extend(self.value.get_bits_le_fixed(params::BALANCE_BIT_WIDTH));
        assert!(
            params::BALANCE_BIT_WIDTH < E::Fr::CAPACITY as usize,
            "due to algebraic nature of the hash we should not overflow the capacity"
        );

        leaf_content
    }
}

impl<E: Engine> std::default::Default for Balance<E> {
    //default should be changed: since subtree_root_hash is not zero for all zero balances and sub accounts
    fn default() -> Self {
        Self {
            value: E::Fr::zero(),
        }
    }
}

lazy_static! {
    static ref BALANCE_TREE: CircuitBalanceTree =
        SparseMerkleTree::new(params::balance_tree_depth());
    static ref ORDER_TREE: CircuitOrderTree =
        SparseMerkleTree::new(params::order_tree_depth());
}
