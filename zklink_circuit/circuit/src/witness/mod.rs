// External deps
use crypto::{digest::Digest, sha2::Sha256};
use num::BigUint;

// Workspace deps
use zklink_crypto::circuit::account::CircuitTidyOrder;
use zklink_crypto::convert::FeConvert;
use zklink_crypto::{
    circuit::utils::{append_be_fixed_width, be_bit_vector_into_bytes},
    Engine, Fr,
};
use zklink_types::{AccountId, ChainId, SubAccountId, TokenId};

// Local deps
pub use crate::witness::account::AccountWitness;
pub use crate::witness::branch::{OperationBranch, OperationBranchWitness};
pub use crate::witness::utils::{get_audits, get_leaf_values};
use crate::{exit_circuit::*, utils::*};

mod account;
mod branch;
mod utils;

type OpenValues = (
    Vec<Vec<AccountWitness<Engine>>>,
    (Vec<Vec<Fr>>, Vec<Vec<CircuitTidyOrder<Engine>>>),
);
type GlobalAssetAccountAuditPath = (
    Vec<Vec<Vec<Option<Fr>>>>,
    (Vec<Vec<Vec<Option<Fr>>>>, Vec<Vec<Vec<Option<Fr>>>>),
);

fn check_source_and_target_token(l2_token: TokenId, l1_token: TokenId) -> (bool, TokenId) {
    let mut real_l1_token = l1_token;
    let is_required_tokens = if *l2_token == USD_TOKEN_ID {
        *real_l1_token = *l1_token - USDX_TOKEN_ID_RANGE;
        USDX_TOKEN_ID_LOWER_BOUND <= *real_l1_token && *real_l1_token <= USDX_TOKEN_ID_UPPER_BOUND
    } else if USDX_TOKEN_ID_LOWER_BOUND <= *l2_token && *l2_token <= USDX_TOKEN_ID_UPPER_BOUND {
        false
    } else {
        l2_token == l1_token
    };
    (is_required_tokens, real_l1_token)
}

/// Get all chain value of global asset account by l2_source_token
fn get_global_asset_account_witnesses(
    l2_source_token: TokenId,
    l1_target_token_after_mapping: TokenId,
    max_chain_num: usize,
    account_tree: &CircuitAccountTree,
) -> OpenValues {
    (1..=max_chain_num)
        .map(|index| {
            if *l2_source_token == USD_TOKEN_ID {
                (USDX_TOKEN_ID_LOWER_BOUND..=USDX_TOKEN_ID_UPPER_BOUND)
                    .map(|usdx_id| {
                        let (global_account_witness, global_balance, global_order) =
                            get_leaf_values(
                                account_tree,
                                *GLOBAL_ASSET_ACCOUNT_ID,
                                (index as u8, usdx_id, 0),
                            );
                        (global_account_witness, (global_balance, global_order))
                    })
                    .unzip()
            } else {
                let (global_account_witness, global_balance, global_order) = get_leaf_values(
                    account_tree,
                    *GLOBAL_ASSET_ACCOUNT_ID,
                    (index as u8, *l1_target_token_after_mapping, 0),
                );
                (
                    vec![global_account_witness; USDX_TOKEN_ID_RANGE as usize],
                    (
                        vec![global_balance; USDX_TOKEN_ID_RANGE as usize],
                        vec![global_order; USDX_TOKEN_ID_RANGE as usize],
                    ),
                )
            }
        })
        .unzip()
}

/// Get all chain amount of global asset account by l2_source_token
fn get_global_asset_account_audit_paths(
    l2_source_token: TokenId,
    l1_target_token_after_mapping: TokenId,
    max_chain_num: usize,
    account_tree: &CircuitAccountTree,
) -> GlobalAssetAccountAuditPath {
    (1..=max_chain_num)
        .map(|index| {
            if *l2_source_token == USD_TOKEN_ID {
                (USDX_TOKEN_ID_LOWER_BOUND..=USDX_TOKEN_ID_UPPER_BOUND).map(|usdx_id| {
                let (global_audit_path, global_audit_balance_path, global_audit_order_path) =
                    get_audits(
                        account_tree,
                        *GLOBAL_ASSET_ACCOUNT_ID,
                        index as u8,
                        usdx_id,
                        0
                    );
                (global_audit_path, (global_audit_balance_path, global_audit_order_path))
            }).unzip::<_,_,Vec<Vec<Option<Fr>>>,(Vec<Vec<Option<Fr>>>,Vec<Vec<Option<Fr>>>)>()
            } else {
                let (global_audit_path, global_audit_balance_path, global_audit_order_path) =
                    get_audits(
                        account_tree,
                        *GLOBAL_ASSET_ACCOUNT_ID,
                        index as u8,
                        *l1_target_token_after_mapping,
                        0,
                    );
                (
                    vec![global_audit_path; USDX_TOKEN_ID_RANGE as usize],
                    (
                        vec![global_audit_balance_path; USDX_TOKEN_ID_RANGE as usize],
                        vec![global_audit_order_path; USDX_TOKEN_ID_RANGE as usize],
                    ),
                )
            }
        })
        .unzip()
}

/// Get all chain audit datas of global asset account
fn get_global_account_audit_datas(
    l2_source_token: TokenId,
    l1_target_token_after_mapping_fe: Fr,
    total_chain_num: usize,
    global_account_witnesses: Vec<Vec<AccountWitness<Engine>>>,
    global_balances: Vec<Vec<Fr>>,
    global_orders: Vec<Vec<CircuitTidyOrder<Engine>>>,
    global_audit_paths: Vec<Vec<Vec<Option<Fr>>>>,
    global_audit_balance_paths: Vec<Vec<Vec<Option<Fr>>>>,
    global_audit_order_paths: Vec<Vec<Vec<Option<Fr>>>>,
) -> Vec<OperationBranch<Engine>> {
    (0..total_chain_num)
        .flat_map(|index| {
            (0..USDX_TOKEN_ID_RANGE as usize)
                .map(|usdx_id| OperationBranch {
                    account_id: Some(Fr::from_u64(*GLOBAL_ASSET_ACCOUNT_ID as u64)),
                    sub_account_id: Some(Fr::from_u64((index + 1) as u64)),
                    token: Some(if *l2_source_token == USD_TOKEN_ID {
                        Fr::from_u64((usdx_id + 2) as u64)
                    } else {
                        l1_target_token_after_mapping_fe
                    }),
                    witness: OperationBranchWitness {
                        account_witness: global_account_witnesses[index][usdx_id].clone(),
                        account_path: global_audit_paths[index][usdx_id].clone(),
                        balance_value: Some(global_balances[index][usdx_id]),
                        balance_subtree_path: global_audit_balance_paths[index][usdx_id].clone(),
                        order_nonce: Some(global_orders[index][usdx_id].nonce),
                        order_residue: Some(global_orders[index][usdx_id].residue),
                        order_subtree_path: global_audit_order_paths[index][usdx_id].clone(),
                    },
                    ..OperationBranch::circuit_init()
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
}

pub fn create_exit_circuit_with_public_input(
    account_tree: &CircuitAccountTree,
    account_id: AccountId,
    sub_account_id: SubAccountId,
    l2_source_token: TokenId,
    l1_target_token: TokenId,
    chain_id: ChainId,
    max_chain_num: usize,
) -> (ZkLinkExitCircuit<'static, Engine>, BigUint) {
    let (is_correct_tokens, l1_target_token_after_mapping) =
        check_source_and_target_token(l2_source_token, l1_target_token);
    assert!(
        is_correct_tokens,
        "Source token or target token is mismatching in exit circuit witness generation"
    );

    let account_id_fe = Fr::from_u64(*account_id as u64);
    let sub_account_id_fe = Fr::from_u64(*sub_account_id as u64);
    let l2_source_token_fe = Fr::from_u64(*l2_source_token as u64);
    let l1_target_token_fe = Fr::from_u64(*l1_target_token as u64);
    let l1_target_token_after_mapping_fe = Fr::from_u64(*l1_target_token_after_mapping as u64);
    let chain_id_fe = Fr::from_u64(*chain_id as u64);
    let root_hash = account_tree.root_hash();

    let (account_witness, balance, order) = get_leaf_values(
        account_tree,
        *account_id,
        (*sub_account_id, *l2_source_token, 0),
    );
    let account_address = account_witness.address.unwrap();
    let (audit_path, audit_balance_path, audit_order_path) = get_audits(
        account_tree,
        *account_id,
        *sub_account_id,
        *l2_source_token,
        0,
    );

    let mut pubdata_commitment = Vec::new();
    append_be_fixed_width(
        &mut pubdata_commitment,
        &root_hash,
        SUBTREE_HASH_WIDTH_PADDED,
    );
    append_be_fixed_width(&mut pubdata_commitment, &chain_id_fe, CHAIN_ID_BIT_WIDTH);
    append_be_fixed_width(
        &mut pubdata_commitment,
        &account_id_fe,
        ACCOUNT_ID_BIT_WIDTH,
    );
    append_be_fixed_width(
        &mut pubdata_commitment,
        &sub_account_id_fe,
        SUB_ACCOUNT_ID_BIT_WIDTH,
    );

    let (global_account_witnesses, (global_balances, global_orders)) =
        get_global_asset_account_witnesses(
            l2_source_token,
            l1_target_token_after_mapping,
            max_chain_num,
            account_tree,
        );
    let (global_audit_paths, (global_audit_balance_paths, global_audit_order_paths)) =
        get_global_asset_account_audit_paths(
            l2_source_token,
            l1_target_token_after_mapping,
            max_chain_num,
            account_tree,
        );
    let sum = global_balances.iter().fold(Fr::zero(), |mut acc, bal| {
        acc.add_assign(
            &bal.iter()
                .enumerate()
                .fold(Fr::zero(), |mut acc, (index, bal)| {
                    if *l2_source_token == USD_TOKEN_ID || index == 0 {
                        acc.add_assign(bal);
                    }
                    acc
                }),
        );
        acc
    });
    let l1_token_index = [0, (*l1_target_token_after_mapping - 2) as usize]
        [(*l2_source_token == USD_TOKEN_ID) as usize];
    let withdraw_ratio =
        div_fr_with_arbitrary_precision::<Engine>(balance, sum, TOKEN_MAX_PRECISION).unwrap();
    let withdraw_amount = multiplication_fr_with_arbitrary_precision::<Engine>(
        global_balances[(*chain_id - 1) as usize][l1_token_index],
        withdraw_ratio,
        TOKEN_MAX_PRECISION,
    )
    .unwrap();

    append_be_fixed_width(&mut pubdata_commitment, &account_address, ADDRESS_WIDTH);
    append_be_fixed_width(
        &mut pubdata_commitment,
        &l1_target_token_fe,
        TOKEN_BIT_WIDTH,
    );
    append_be_fixed_width(
        &mut pubdata_commitment,
        &l2_source_token_fe,
        TOKEN_BIT_WIDTH,
    );
    append_be_fixed_width(&mut pubdata_commitment, &withdraw_amount, BALANCE_BIT_WIDTH);

    let mut h = Sha256::new();
    let bytes_to_hash = be_bit_vector_into_bytes(&pubdata_commitment);
    h.input(&bytes_to_hash);
    let mut hash_result = [0u8; 32];
    h.result(&mut hash_result[..]);
    hash_result[0] &= BN256_MASK; // temporary solution, this nullifies top bits to be encoded into field element correctly

    let pub_data_commitment = Fr::from_bytes(&hash_result).unwrap();
    let global_account_audit_datas = get_global_account_audit_datas(
        l2_source_token,
        l1_target_token_after_mapping_fe,
        max_chain_num,
        global_account_witnesses,
        global_balances,
        global_orders,
        global_audit_paths,
        global_audit_balance_paths,
        global_audit_order_paths,
    );
    (
        ZkLinkExitCircuit {
            params: &zklink_crypto::params::RESCUE_PARAMS,
            chain_id: Some(chain_id_fe),
            l1_target_token: Some(l1_target_token_fe),
            l1_target_token_after_mapping: Some(l1_target_token_after_mapping_fe),
            pub_data_commitment: Some(pub_data_commitment),
            root_hash: Some(root_hash),
            account_audit_data: OperationBranch {
                account_id: Some(account_id_fe),
                sub_account_id: Some(sub_account_id_fe),
                token: Some(l2_source_token_fe),
                witness: OperationBranchWitness {
                    account_witness,
                    account_path: audit_path,
                    balance_value: Some(balance),
                    balance_subtree_path: audit_balance_path,
                    order_nonce: Some(order.nonce),
                    order_residue: Some(order.residue),
                    order_subtree_path: audit_order_path,
                },
                ..OperationBranch::circuit_init()
            },
            global_account_audit_data: global_account_audit_datas,
        },
        withdraw_amount.into_big_uint(),
    )
}
