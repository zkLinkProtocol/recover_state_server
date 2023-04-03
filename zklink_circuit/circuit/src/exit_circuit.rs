// Workspace deps
pub use zklink_crypto::{
    circuit::{*, utils::*}, convert::FeConvert, franklin_crypto::{
        bellman::{
            BitIterator, Circuit, ConstraintSystem, Engine, pairing::ff::{Field, PrimeField, PrimeFieldRepr}, SynthesisError
        },
        circuit::{
            Assignment, boolean::{AllocatedBit, Boolean, le_bits_into_le_bytes}, ecc, expression::Expression,
            float_point::parse_with_exponent_le, multipack, num::{AllocatedNum, Num}, polynomial_lookup::{do_the_lookup, generate_powers}, rescue, sha256
        },
        jubjub::{FixedGenerators, JubjubEngine, JubjubParams}, rescue::{bn256::Bn256RescueParams, rescue_hash, RescueEngine}
    },
};
pub use zklink_types::{H256, operations::*, Order, params::*};

// Local deps
pub use crate::{
    witness::{AccountWitness, OperationBranch}, branch::*, element::CircuitElement,
    utils::{
        boolean_or, multi_and, multi_or, pack_bits_to_element_strict,
        div_based_on_u126, multiply_based_on_u126
    },
};

#[derive(Clone)]
pub struct ZkLinkExitCircuit<'a, E: RescueEngine> {
    pub params: &'a E::Params,
    pub chain_id: Option<E::Fr>,
    pub pub_data_commitment: Option<E::Fr>,
    /// The old root of the tree
    pub root_hash: Option<E::Fr>,
    pub account_audit_data: OperationBranch<E>,
    pub global_account_audit_data: Vec<OperationBranch<E>>,
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
            account_tree_depth(),
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
        for (index, branch) in self.global_account_audit_data.iter().enumerate() {
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
                account_tree_depth(),
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
            let accumulated_balance = AllocatedNum::conditionally_select(
                cs.namespace(||"choose whether to accumulate balance"),
                &zero,
                global_branch.balance.get_number(),
                &not_usd_and_not_first
            )?;
            all_chain_sum = all_chain_sum.add(
                cs.namespace(||"add balance"),
                &accumulated_balance
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
            let mut initial_hash_data: Vec<Boolean> = vec![];
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
            let usdx_token_id_upper_bound = CircuitElement::from_fe_with_known_length(
                cs.namespace(|| "usdx_token_id_upper_bound"),
                || Ok(E::Fr::from_u64(USDX_TOKEN_ID_UPPER_BOUND as u64)),
                USD_TOKEN_BIT
            )?;
            let is_gt_usdx_upper_bound = CircuitElement::less_than_fixed(
                cs.namespace(|| "is_gt_usdx_upper_bound"),
                &usdx_token_id_upper_bound,
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


pub fn check_account_data<E: RescueEngine, CS: ConstraintSystem<E>>(
    mut cs: CS,
    cur: &AllocatedOperationBranch<E>,
    length_to_root: usize,
    params: &E::Params,
) -> Result<(AllocatedNum<E>, Boolean, CircuitElement<E>), SynthesisError> {
    // first we prove calculate root of the subtree to obtain account_leaf_data:
    let (cur_account_leaf_bits, is_account_empty, subtree_root) = allocate_account_leaf_bits(
        cs.namespace(|| "allocate current_account_leaf_hash"),
        cur,
        params,
    )?;

    Ok((
        allocate_merkle_root(
            cs.namespace(|| "account_merkle_root"),
            &cur_account_leaf_bits,
            &cur.account_id.get_bits_le(),
            &cur.account_audit_path,
            length_to_root,
            params,
        )?,
        is_account_empty,
        subtree_root,
    ))
}

pub fn allocate_account_leaf_bits<E: RescueEngine, CS: ConstraintSystem<E>>(
    mut cs: CS,
    branch: &AllocatedOperationBranch<E>,
    params: &E::Params,
) -> Result<(Vec<Boolean>, Boolean, CircuitElement<E>), SynthesisError> {
    let mut account_data = vec![];
    account_data.extend_from_slice(branch.account.nonce.get_bits_le());
    account_data.extend_from_slice(branch.account.pub_key_hash.get_bits_le());
    account_data.extend_from_slice(branch.account.address.get_bits_le());

    let is_account_empty = {
        let zero = AllocatedNum::zero(cs.namespace(||"zero"))?;
        let is_nonce_zero = Boolean::from(AllocatedNum::equals(
            cs.namespace(|| "nonce is zero if empty"),
            branch.account.nonce.get_number(),
            &zero
        )?);
        let is_pubkey_hash_zero = Boolean::from(AllocatedNum::equals(
            cs.namespace(|| "pubkey hash is zero if empty"),
            branch.account.pub_key_hash.get_number(),
            &zero
        )?);
        let is_address_zero = Boolean::from(AllocatedNum::equals(
            cs.namespace(|| "address is zero if empty"),
            branch.account.address.get_number(),
            &zero
        )?);
        multi_and(
            cs.namespace(|| "check if all account words are empty"),
            &[is_nonce_zero, is_pubkey_hash_zero, is_address_zero],
        )?
    };

    // first we prove calculate root of the balance tree to obtain account_leaf_data:
    let state_tree_root = branch.calculate_balance_tree_root(cs.namespace(||"calculate balance tree root"), params)?;
    account_data.extend(state_tree_root.clone().into_padded_le_bits(FR_BIT_WIDTH_PADDED));

    // third we prove calculate root of the order tree
    let order_tree_root = branch.calculate_order_tree_root(cs.namespace(||"calculate order tree root"), params)?;
    account_data.extend(order_tree_root);

    Ok((account_data, is_account_empty, state_tree_root))
}

pub fn allocate_merkle_root<E: RescueEngine, CS: ConstraintSystem<E>>(
    mut cs: CS,
    leaf_bits: &[Boolean],
    index: &[Boolean],
    audit_path: &[AllocatedNum<E>],
    length_to_root: usize,
    params: &E::Params,
) -> Result<AllocatedNum<E>, SynthesisError> {
    // only first bits of index are considered valuable
    assert!(length_to_root <= index.len());
    assert!(index.len() >= audit_path.len());

    let index = &index[0..length_to_root];
    let audit_path = &audit_path[0..length_to_root];

    let leaf_packed = multipack::pack_into_witness(
        cs.namespace(|| "pack leaf bits into field elements"),
        &leaf_bits,
    )?;

    let mut account_leaf_hash = rescue::rescue_hash(
        cs.namespace(|| "account leaf content hash"),
        &leaf_packed,
        params,
    )?;

    assert_eq!(account_leaf_hash.len(), 1);

    let mut cur_hash = account_leaf_hash.pop().expect("must get a single element");

    // Ascend the merkle tree authentication path
    for (i, direction_bit) in index.iter().enumerate() {
        let cs = &mut cs.namespace(|| format!("from merkle tree hash {}", i));

        // "direction_bit" determines if the current subtree
        // is the "right" leaf at this depth of the tree.

        // Witness the authentication path element adjacent
        // at this depth.
        let path_element = &audit_path[i];

        // Swap the two if the current subtree is on the right
        let (xl, xr) = AllocatedNum::conditionally_reverse(
            cs.namespace(|| "conditional reversal of preimage"),
            &cur_hash,
            path_element,
            &direction_bit,
        )?;

        // we do not use any personalization here cause
        // our tree is of a fixed height and hash function
        // is resistant to padding attacks
        let mut sponge_output = rescue::rescue_hash(
            cs.namespace(|| format!("hash tree level {}", i)),
            &[xl, xr],
            params,
        )?;

        assert_eq!(sponge_output.len(), 1);
        cur_hash = sponge_output.pop().expect("must get a single element");
    }

    Ok(cur_hash)
}