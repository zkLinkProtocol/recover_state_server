use crate::exit_circuit::*;
use crate::witness::account::AccountWitness;

#[derive(Clone, Debug)]
pub struct OperationBranchWitness<E: RescueEngine> {
    pub account_witness: AccountWitness<E>,
    pub account_path: Vec<Option<E::Fr>>,

    pub balance_value: Option<E::Fr>,
    pub balance_subtree_path: Vec<Option<E::Fr>>,

    pub order_nonce: Option<E::Fr>,
    pub order_residue: Option<E::Fr>,
    pub order_subtree_path: Vec<Option<E::Fr>>,
}

impl<E: RescueEngine> Default for OperationBranchWitness<E> {
    fn default() -> Self {
        Self {
            account_witness: Default::default(),
            account_path: vec![None; account_tree_depth()],
            balance_value: None,
            balance_subtree_path: vec![None; balance_tree_depth()],
            order_nonce: None,
            order_residue: None,
            order_subtree_path: vec![None; order_tree_depth()],
        }
    }
}

impl<E: RescueEngine> OperationBranchWitness<E> {
    fn circuit_init() -> Self {
        Self {
            account_witness: AccountWitness::circuit_init(),
            account_path: vec![Some(E::Fr::zero()); account_tree_depth()],
            balance_value: None,
            balance_subtree_path: vec![Some(E::Fr::zero()); balance_tree_depth()],
            order_nonce: None,
            order_residue: None,
            order_subtree_path: vec![Some(E::Fr::zero()); order_tree_depth()],
        }
    }
}

#[derive(Clone, Debug)]
pub struct OperationBranch<E: RescueEngine> {
    pub account_id: Option<E::Fr>,
    pub sub_account_id: Option<E::Fr>,
    pub token: Option<E::Fr>,
    pub slot_number: Option<E::Fr>,

    pub witness: OperationBranchWitness<E>,
}

impl<E: RescueEngine> Default for OperationBranch<E> {
    fn default() -> Self {
        Self {
            account_id: None,
            sub_account_id: None,
            token: None,
            slot_number: None,
            witness: Default::default(),
        }
    }
}

impl<E: RescueEngine> OperationBranch<E> {
    pub fn circuit_init() -> Self {
        Self {
            account_id: Some(E::Fr::zero()),
            sub_account_id: Some(E::Fr::zero()),
            token: Some(E::Fr::zero()),
            slot_number: Some(E::Fr::zero()),
            witness: OperationBranchWitness::circuit_init(),
        }
    }
}
