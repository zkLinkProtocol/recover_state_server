// External deps
use crypto::{digest::Digest, sha2::Sha256};
// Workspace deps
use zklink_crypto::{circuit::utils::{append_be_fixed_width, be_bit_vector_into_bytes}, Engine, Fr};
use zklink_crypto::circuit::account::CircuitTidyOrder;
use zklink_crypto::convert::FeConvert;
use zklink_types::{AccountId, ChainId, SubAccountId, TokenId};
// Local deps
use crate::{circuit::*, operation::*, utils::*, witness::*};
use crate::witness::utils::{get_audits, get_leaf_values};

const EXIT_PUB_DATA_BIT_WIDTH: usize = SUBTREE_HASH_WIDTH_PADDED
 + CHAIN_ID_BIT_WIDTH
 + ACCOUNT_ID_BIT_WIDTH
 + SUB_ACCOUNT_ID_BIT_WIDTH
 + ADDRESS_WIDTH
 + 2 * TOKEN_BIT_WIDTH
 + BALANCE_BIT_WIDTH;

#[derive(Clone)]
pub struct ZkLinkExitCircuit<'a, E: RescueEngine> {
    pub params: &'a E::Params,
    pub chain_id: Option<E::Fr>,
    /// The old root of the tree
    pub pub_data_commitment: Option<E::Fr>,
    pub root_hash: Option<E::Fr>,
    pub account_audit_data: OperationBranch<E>,
    pub global_account_audit_datas: Vec<OperationBranch<E>>,
    pub l1_target_token: Option<E::Fr>,
    pub l1_target_token_after_mapping: Option<E::Fr>,
}

// Implementation of our circuit:
impl<'a, E: RescueEngine> Circuit<E> for ZkLinkExitCircuit<'a, E> {
    fn synthesize<CS: ConstraintSystem<E>>(self, cs: &mut CS) -> Result<(), SynthesisError> {
        let zero = AllocatedNum::zero(cs.namespace(||"zero"))?;
        // this is only public input to our circuit
        let public_data_commitment =
            AllocatedNum::alloc(cs.namespace(|| "public_data_commitment"), || {
                self.pub_data_commitment.grab()
            })?;
        public_data_commitment.inputize(cs.namespace(|| "inputize pub_data"))?;

        let root_hash =
            AllocatedNum::alloc(cs.namespace(|| "root_hash"), || self.root_hash.grab())?;
        let chain_id_ce = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "chain_id_ce"),
            ||self.chain_id.grab(),
            CHAIN_ID_BIT_WIDTH
        )?;
        let l1_target_token_id = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "l1_target_token_id"),
            ||self.l1_target_token.grab(),
            TOKEN_BIT_WIDTH
        )?;
        let l1_target_token_after_mapping = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "l1_target_token_after_mapping"),
            ||self.l1_target_token_after_mapping.grab(),
            TOKEN_BIT_WIDTH
        )?;

        let branch = AllocatedOperationBranch::from_witness(
            cs.namespace(|| "lhs"),
            &self.account_audit_data,
        )?;
        let is_usd_l2_token: Boolean = Expression::equals(
            cs.namespace(|| "is usd token"),
            Expression::from(branch.token.get_number()),
            Expression::u64::<CS>(USD_TOKEN_ID as u64),
        )?.into();
        {
            let is_required_source_token_and_target_token = require_source_token_and_target_token(
                cs.namespace(|| "is_required_source_token_and_target_token"),
                &l1_target_token_after_mapping,
                &l1_target_token_id,
                &branch.token,
            )?;
            Boolean::enforce_equal(
                cs.namespace(|| " require correct token"),
                &is_required_source_token_and_target_token,
                &Boolean::constant(true),
            )?;
        }
        // calculate root for given account data
        let (state_root, _, _) = check_account_data(
            cs.namespace(|| "calculate account root"),
            &branch,
            zklink_crypto::params::account_tree_depth(),
            self.params,
        )?;
        // ensure root hash of state is correct
        cs.enforce(
            || "account audit data corresponds to the root hash",
            |lc| lc + state_root.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc + root_hash.get_variable(),
        );

        let mut all_chain_sum = zero.clone();
        let mut target_chain_token_surplus = zero.clone();
        for (index, branch) in self.global_account_audit_datas.iter().enumerate() {
            let mut cs = cs.namespace(||format!("{}th branch", index));
            let is_first_every_chain = if index as u32 % USDX_TOKEN_ID_RANGE == 0{
                Boolean::constant(true)
            } else {
                Boolean::constant(false)
            };
            let global_branch = AllocatedOperationBranch::from_witness(
                cs.namespace(|| "alloc branch"),
                branch,
            )?;
            // calculate root for given account data
            let (state_root, _, _) = check_account_data(
                cs.namespace(|| "calculate account root"),
                &global_branch,
                zklink_crypto::params::account_tree_depth(),
                self.params,
            )?;
            // ensure root hash of state is correct
            cs.enforce(
                || "account audit data corresponds to the root hash",
                |lc| lc + state_root.get_variable(),
                |lc| lc + CS::one(),
                |lc| lc + root_hash.get_variable(),
            );
            let is_correct_token_and_chain = {
                let is_correct_chain = CircuitElement::equals(
                    cs.namespace(|| "is correct sub account id"),
                    &global_branch.sub_account_id,
                    &chain_id_ce,
                )?;
                let is_correct_token = CircuitElement::equals(
                    cs.namespace(|| "is correct is_correct_token id"),
                    &global_branch.token,
                    &l1_target_token_after_mapping,
                )?;
                Boolean::and(
                    cs.namespace(||"is_correct_chain and is_correct_token"),
                    &is_correct_chain,
                    &is_correct_token
                )?
            };
            target_chain_token_surplus = Expression::conditionally_select(
                cs.namespace(||"select surplus of target_chain"),
                global_branch.balance.get_number(),
                &target_chain_token_surplus,
                &is_correct_token_and_chain,
            )?;
            let not_usd_and_not_first = Boolean::and(
                cs.namespace(|| "not_usd_and_not_first"),
                &is_first_every_chain.not(),
                &is_usd_l2_token.not()
            )?;
            let accumalated_balance = AllocatedNum::conditionally_select(
                cs.namespace(||"choose whether to accumlate balance"),
                &zero,
                global_branch.balance.get_number(),
                &not_usd_and_not_first
            )?;
            all_chain_sum = all_chain_sum.add(
                cs.namespace(||"add balance"),
                &accumalated_balance
            )?;
        }

        let withdraw_amount = {
            let all_chain_sum = CircuitElement::from_number_with_known_length(
                cs.namespace(||"all_chain_sum convert as ce"),
                all_chain_sum,
                BALANCE_BIT_WIDTH
            )?;
            let target_chain_token_surplus = CircuitElement::from_number_with_known_length(
                cs.namespace(||"target_chain_token_surplus convert as ce"),
                target_chain_token_surplus,
                BALANCE_BIT_WIDTH
            )?;
            let withdraw_ratio = div_based_on_u126(
                cs.namespace(|| "calculate the proportion of the balance to all chain total"),
                &branch.balance,
                &all_chain_sum,
                TOKEN_MAX_PRECISION
            )?;
            let amount = multiply_based_on_u126(
                cs.namespace(|| "calculate withdraw amount"),
                &withdraw_ratio,
                &target_chain_token_surplus,
                TOKEN_MAX_PRECISION,
            )?;
            CircuitElement::from_number_with_known_length(
                cs.namespace(||"amount to CircuitElement"),
                amount,
                BALANCE_BIT_WIDTH
            )?
        };
        {
            let mut initial_hash_data: Vec<Boolean> = Vec::with_capacity(EXIT_PUB_DATA_BIT_WIDTH);
            let root_hash_ce =
                CircuitElement::from_number(cs.namespace(|| "root_hash_ce"), root_hash)?;
            initial_hash_data.extend(root_hash_ce.into_padded_be_bits(FR_BIT_WIDTH_PADDED));
            initial_hash_data.extend(chain_id_ce.get_bits_be());
            initial_hash_data.extend(branch.account_id.get_bits_be());
            initial_hash_data.extend(branch.sub_account_id.get_bits_be());
            initial_hash_data.extend(branch.account.address.get_bits_be());
            initial_hash_data.extend(l1_target_token_id.get_bits_be());
            initial_hash_data.extend(branch.token.get_bits_be());
            initial_hash_data.extend(withdraw_amount.get_bits_be());

            let mut hash_block =
                sha256::sha256(cs.namespace(|| "sha256 of pub data"), &initial_hash_data)?;

            hash_block.reverse();
            hash_block.truncate(E::Fr::CAPACITY as usize);

            let final_hash =
                AllocatedNum::pack_bits_to_element(cs.namespace(|| "final_hash"), &hash_block)?;

            cs.enforce(
                || "enforce external data hash equality",
                |lc| lc + public_data_commitment.get_variable(),
                |lc| lc + CS::one(),
                |lc| lc + final_hash.get_variable(),
            );
        }
        Ok(())
    }
}

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
    total_chain_num: usize,
    account_tree: &CircuitAccountTree
) -> (Vec<Vec<AccountWitness<Bn256>>>,(Vec<Vec<Fr>>,Vec<Vec<CircuitTidyOrder<Bn256>>>)){
    (1..=total_chain_num).map(|index| {
        if *l2_source_token == USD_TOKEN_ID{
            (USDX_TOKEN_ID_LOWER_BOUND..=USDX_TOKEN_ID_UPPER_BOUND).map(|usdx_id| {
                let (global_account_witness,global_balance,global_order) =
                    get_leaf_values(
                        account_tree,
                        *GLOBAL_ASSET_ACCOUNT_ID,
                        (index as u8, usdx_id, 0),
                    );
                (global_account_witness, (global_balance, global_order))
            }).unzip()
        } else {
            let (global_account_witness,global_balance,global_order) =
                get_leaf_values(
                    account_tree,
                    *GLOBAL_ASSET_ACCOUNT_ID,
                    (index as u8, *l1_target_token_after_mapping, 0),
                );
            (
                vec![global_account_witness; USDX_TOKEN_ID_RANGE as usize],
                (vec![global_balance; USDX_TOKEN_ID_RANGE as usize],vec![global_order; USDX_TOKEN_ID_RANGE as usize])
            )
        }
    }
    ).unzip()
}

/// Get all chain amount of global asset account by l2_source_token
fn get_global_asset_account_audit_paths(
    l2_source_token: TokenId,
    l1_target_token_after_mapping: TokenId,
    total_chain_num: usize,
    account_tree: &CircuitAccountTree
) -> (Vec<Vec<Vec<Option<Fr>>>>,(Vec<Vec<Vec<Option<Fr>>>>,Vec<Vec<Vec<Option<Fr>>>>)){
    (1..=total_chain_num).map(|index| {
        if *l2_source_token == USD_TOKEN_ID{
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
                    0
                );
            (
                vec![global_audit_path; USDX_TOKEN_ID_RANGE as usize],
                (vec![global_audit_balance_path; USDX_TOKEN_ID_RANGE as usize],vec![global_audit_order_path; USDX_TOKEN_ID_RANGE as usize])
            )

        }
    }).unzip()
}

/// Get all chain audit datas of global asset account
fn get_global_account_audit_datas(
    l2_source_token: TokenId,
    l1_target_token_after_mapping_fe: Fr,
    total_chain_num: usize,
    global_account_witnesses: Vec<Vec<AccountWitness<Bn256>>>,
    global_balances: Vec<Vec<Fr>>,
    global_orders: Vec<Vec<CircuitTidyOrder<Bn256>>>,
    global_audit_paths: Vec<Vec<Vec<Option<Fr>>>>,
    global_audit_balance_paths: Vec<Vec<Vec<Option<Fr>>>>,
    global_audit_order_paths: Vec<Vec<Vec<Option<Fr>>>>,
) -> Vec<OperationBranch<Bn256>>{
    (0..total_chain_num).flat_map(|index| {
        (0..USDX_TOKEN_ID_RANGE as usize).map(|usdx_id| {
            OperationBranch {
                account_id: Some(Fr::from_u64(*GLOBAL_ASSET_ACCOUNT_ID as u64)),
                sub_account_id: Some(Fr::from_u64((index + 1) as u64)),
                token: Some(if *l2_source_token == USD_TOKEN_ID { Fr::from_u64((usdx_id + 2) as u64) } else { l1_target_token_after_mapping_fe}),
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
            }
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>()
}

pub fn create_exit_circuit_with_public_input(
    account_tree: &CircuitAccountTree,
    account_id: AccountId,
    sub_account_id: SubAccountId,
    l2_source_token: TokenId,
    l1_target_token: TokenId,
    chain_id: ChainId,
    total_chain_num: usize
) -> (ZkLinkExitCircuit<'static, Engine>, BigUint) {
    let (is_correct_tokens,l1_target_token_after_mapping) =
        check_source_and_target_token(l2_source_token, l1_target_token);
    assert!(is_correct_tokens, "Source token or target token is mismatching in exit circuit witness generation");

    let account_id_fe = Fr::from_u64(*account_id as u64);
    let sub_account_id_fe = Fr::from_u64(*sub_account_id as u64);
    let l2_source_token_fe = Fr::from_u64(*l2_source_token as u64);
    let l1_target_token_fe = Fr::from_u64(*l1_target_token as u64);
    let l1_target_token_after_mapping_fe = Fr::from_u64(*l1_target_token_after_mapping as u64);
    let chain_id_fe = Fr::from_u64(*chain_id as u64);
    let root_hash = account_tree.root_hash();

    let (account_witness, balance, order) =
        get_leaf_values(
            account_tree,
            *account_id,
            (*sub_account_id, *l2_source_token, 0),
        );
    let account_address = account_witness.address.unwrap();
    let (audit_path, audit_balance_path, audit_order_path) =
        get_audits(account_tree, *account_id, *sub_account_id, *l2_source_token, 0);

    let mut pubdata_commitment = Vec::with_capacity(EXIT_PUB_DATA_BIT_WIDTH);
    append_be_fixed_width(&mut pubdata_commitment, &root_hash, SUBTREE_HASH_WIDTH_PADDED);
    append_be_fixed_width(&mut pubdata_commitment, &chain_id_fe, CHAIN_ID_BIT_WIDTH);
    append_be_fixed_width(&mut pubdata_commitment, &account_id_fe, ACCOUNT_ID_BIT_WIDTH);
    append_be_fixed_width(&mut pubdata_commitment, &sub_account_id_fe, SUB_ACCOUNT_ID_BIT_WIDTH);

    let (global_account_witnesses, (global_balances, global_orders)) =
        get_global_asset_account_witnesses(
            l2_source_token,
            l1_target_token_after_mapping,
            total_chain_num,
            account_tree,
        );
    let (global_audit_paths, (global_audit_balance_paths, global_audit_order_paths)) =
        get_global_asset_account_audit_paths(
            l2_source_token,
            l1_target_token_after_mapping,
            total_chain_num,
            account_tree,
        );
    let sum = global_balances.iter().fold(
        Fr::zero(),|mut acc, bal| {
            acc.add_assign(
                &bal.iter().enumerate().fold(
                    Fr::zero(),|mut acc, (index, bal)|
                        {
                            if *l2_source_token == USD_TOKEN_ID || index == 0{
                                acc.add_assign(bal);
                            }
                            acc
                        }
                    )
            );
            acc
        });
    let l1_token_index = [0, (*l1_target_token_after_mapping - 2) as usize][(*l2_source_token == USD_TOKEN_ID) as usize];
    let withdraw_ratio = div_fr_with_arbitrary_precision::<Engine>(balance, sum, TOKEN_MAX_PRECISION).unwrap();
    let withdraw_amount = multiplication_fr_with_arbitrary_precision::<Engine>(
        global_balances[(*chain_id - 1) as usize][l1_token_index],
        withdraw_ratio,
        TOKEN_MAX_PRECISION
    ).unwrap();

    append_be_fixed_width(&mut pubdata_commitment, &account_address, ADDRESS_WIDTH);
    append_be_fixed_width(&mut pubdata_commitment, &l1_target_token_fe, TOKEN_BIT_WIDTH);
    append_be_fixed_width(&mut pubdata_commitment, &l2_source_token_fe, TOKEN_BIT_WIDTH);
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
        total_chain_num,
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
            l1_target_token_after_mapping:  Some(l1_target_token_after_mapping_fe),
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
                    order_subtree_path: audit_order_path
                },
                ..OperationBranch::circuit_init()
            },
            global_account_audit_datas,
        },
        withdraw_amount.into_big_uint()
    )
}

fn require_source_token_and_target_token<E: RescueEngine, CS: ConstraintSystem<E>>(
    mut cs: CS,
    l1_target_token_after_mapping: &CircuitElement<E>,
    l1_target_token: &CircuitElement<E>,
    l2_source_token: &CircuitElement<E>,
)-> Result<Boolean, SynthesisError> {
    let is_correct_usd = {
        let real_l1_token = Expression::from(l1_target_token.get_number()) - Expression::u64::<CS>(USDX_TOKEN_ID_RANGE as u64);
        let is_correct_l1_token: Boolean = Expression::equals(
            cs.namespace(|| "is_correct_l1_token"),
            real_l1_token,
            Expression::from(l1_target_token_after_mapping.get_number()),
        )?.into();
        let is_usd_l2_token: Boolean = Expression::equals(
            cs.namespace(|| "is usd token"),
            Expression::from(l2_source_token.get_number()),
            Expression::u64::<CS>(USD_TOKEN_ID as u64),
        )?.into();
        Boolean::and(
            cs.namespace(||"is_correct_usd_constraint"),
            &is_correct_l1_token, &is_usd_l2_token
        )?
    };
    let is_correct_token_range = {
        let is_correct_token_range = {
            let is_zero_token = Expression::equals(
                cs.namespace(|| "is zero token"),
                Expression::from(l2_source_token.get_number()),
                Expression::u64::<CS>(0)
            )?.into();
            let usdx_tokene_id_upper_bound = CircuitElement::from_fe_with_known_length(
                cs.namespace(|| "usdx_tokene_id_upper_bound"),
                || Ok(E::Fr::from_u64(USDX_TOKEN_ID_UPPER_BOUND as u64)),
                USD_TOKEN_BIT
            )?;
            let is_gt_usdx_upper_bound = CircuitElement::less_than_fixed(
                cs.namespace(|| "is_gt_usdx_upper_bound"),
                &usdx_tokene_id_upper_bound,
                l2_source_token,
            )?;
            boolean_or(
                cs.namespace(||"is_correct_token_range"),
                &is_zero_token, &is_gt_usdx_upper_bound,
            )?
        };
        let is_eq_token = CircuitElement::equals(
            cs.namespace(||"is_eq_token"),
            l2_source_token,
            l1_target_token
        )?;
        Boolean::and(
            cs.namespace(|| "is correct l2 token range"),
            &is_correct_token_range, &is_eq_token
        )?
    };
    boolean_or(
        cs.namespace(||"check source token and target token"),
        &is_correct_usd, &is_correct_token_range
    )
}
