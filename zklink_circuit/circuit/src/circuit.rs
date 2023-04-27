// Workspace deps
pub use zklink_crypto::{
    franklin_crypto::{
        bellman::{
            bn256::{Bn256, Fr}, pairing::ff::{Field, PrimeField, PrimeFieldRepr},
            Circuit, ConstraintSystem, BitIterator, SynthesisError, Engine
        },
        circuit::{
            Assignment, num::{AllocatedNum, Num}, boolean::{le_bits_into_le_bytes, Boolean, AllocatedBit}, expression::Expression,
            polynomial_lookup::{do_the_lookup, generate_powers}, rescue, sha256, multipack, ecc, float_point::parse_with_exponent_le
        },
        jubjub::{FixedGenerators, JubjubEngine, JubjubParams}, rescue::{bn256::Bn256RescueParams, RescueEngine, rescue_hash}
    }, convert::FeConvert, circuit::{*, utils::*},
};
pub use zklink_types::{operations::*, params::*, Order, H256};
// Local deps
pub use crate::{
    account::AccountWitness, allocated_structures::*, element::CircuitElement, operation::OperationUnit,
    utils::{
        boolean_or, multi_or, multi_and, pack_bits_to_element_strict,
    },
};

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
            cur.account_id.get_bits_le(),
            &cur.account_audit_path,
            length_to_root,
            params,
        )?,
        is_account_empty,
        subtree_root,
    ))
}

/// Account tree state will be extended in the future, so for current balance tree we
/// append emtpy hash to reserve place for the future tree before hashing.
pub fn calc_account_state_tree_root<E: RescueEngine, CS: ConstraintSystem<E>>(
    mut cs: CS,
    balance_root: &CircuitElement<E>,
    params: &E::Params,
) -> Result<CircuitElement<E>, SynthesisError> {
    let state_tree_root_input = balance_root.get_number().clone();
    let empty_root_padding =
        AllocatedNum::zero(cs.namespace(|| "allocate zero element for padding"))?;

    let mut sponge_output = rescue::rescue_hash(
        cs.namespace(|| "hash state root and balance root"),
        &[state_tree_root_input, empty_root_padding],
        params,
    )?;

    assert_eq!(sponge_output.len(), 1);
    let state_tree_root = sponge_output.pop().expect("must get a single element");

    CircuitElement::from_number(cs.namespace(|| "total_subtree_root_ce"), state_tree_root)
}

pub fn allocate_account_leaf_bits<E: RescueEngine, CS: ConstraintSystem<E>>(
    mut cs: CS,
    branch: &AllocatedOperationBranch<E>,
    params: &E::Params,
) -> Result<(Vec<Boolean>, Boolean, CircuitElement<E>), SynthesisError> {
    let mut account_data = Vec::with_capacity(NONCE_BIT_WIDTH + NEW_PUBKEY_HASH_WIDTH + ETH_ADDRESS_BIT_WIDTH + 2 * FR_BIT_WIDTH_PADDED);
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
        leaf_bits,
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
            direction_bit,
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
