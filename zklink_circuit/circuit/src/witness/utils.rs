// External deps
// Workspace deps
use zklink_crypto::circuit::account::{CircuitAccount, CircuitTidyOrder};
use zklink_crypto::bellman::bn256::Bn256;
use zklink_crypto::Fr;
use zklink_types::utils::{calculate_actual_slot, calculate_actual_token};
// Local deps
use crate::exit_circuit::*;
use crate::witness::account::AccountWitness;

pub fn get_audits(
    tree: &CircuitAccountTree,
    account_id: u32,
    sub_account_id: u8,
    token: u32,
    slot_id: u32,
) -> (Vec<Option<Fr>>, Vec<Option<Fr>>, Vec<Option<Fr>>) {
    let token_id = calculate_actual_token(sub_account_id.into(), token.into());
    let slot_id = calculate_actual_slot( sub_account_id.into(), slot_id.into()).0 as u32;
    let default_account = CircuitAccount::default();
    let audit_account: Vec<Option<Fr>> = tree
        .merkle_path(account_id)
        .into_iter()
        .map(|e| Some(e.0))
        .collect();

    let audit_balance: Vec<Option<Fr>> = tree
        .get(account_id)
        .unwrap_or(&default_account)
        .subtree
        .merkle_path(*token_id)
        .into_iter()
        .map(|(e, _)| Some(e))
        .collect();
    let audit_order: Vec<Option<Fr>> = tree
        .get(account_id)
        .unwrap_or(&default_account)
        .order_tree
        .merkle_path(slot_id)
        .into_iter()
        .map(|(e, _)| Some(e))
        .collect();
    (audit_account, audit_balance, audit_order)
}

pub fn get_leaf_values(
    tree: &CircuitAccountTree,
    account_id: u32,
    (sub_account_id, token_id, slot_id): (u8, u32, u32),
) -> (AccountWitness<Bn256>, Fr, CircuitTidyOrder<Bn256>) {
    let account = tree.get(account_id).unwrap();
    let account_witness = AccountWitness::from_circuit_account(&account);

    let token_id = calculate_actual_token(sub_account_id.into(), token_id.into());
    let slot_id = calculate_actual_slot(sub_account_id.into(),slot_id.into()).0 as u32;

    let balance = account
        .subtree
        .get(*token_id)
        .cloned()
        .unwrap_or_default()
        .value;
    let order = account
        .order_tree
        .get(slot_id)
        .cloned()
        .unwrap_or_default();

    (
        account_witness,
        balance,
        order,
    )
}


