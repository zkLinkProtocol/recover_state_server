//! Generate exit proof for exodus mode given account and token

use std::fs::File;
use anyhow::format_err;
use num::BigUint;
use std::time::Instant;
use tracing::info;
use recover_state_config::RecoverStateConfig;
use zklink_basic_types::{ChainId, SubAccountId};
use zklink_circuit::exit_circuit::create_exit_circuit_with_public_input;
use zklink_crypto::circuit::account::CircuitAccount;
use zklink_crypto::circuit::CircuitAccountTree;
use zklink_crypto::proof::EncodedSingleProof;
use zklink_types::{AccountId, AccountMap, TokenId};
use zklink_crypto::bellman::plonk::better_cs::{
    keys::VerificationKey, verifier::verify,
};
use zklink_crypto::bellman::plonk::{
    commitments::transcript::keccak_transcript::RollingKeccakTranscript,
    prove_by_steps, setup, transpile,
};
use zklink_crypto::franklin_crypto::bellman::Circuit;
use zklink_crypto::params::RECURSIVE_CIRCUIT_VK_TREE_DEPTH;
use zklink_crypto::proof::SingleProof;
use zklink_crypto::{Engine, Fr};
use crate::SETUP_MIN_POW2;

pub fn create_exit_proof(
    config: &RecoverStateConfig,
    circuit_account_tree: &CircuitAccountTree,
    account_id: AccountId,
    sub_account_id: SubAccountId,
    l2_source_token: TokenId,
    l1_target_token: TokenId,
    chain_id: ChainId,
    total_chain_num: usize
) -> Result<(EncodedSingleProof, BigUint), anyhow::Error> {
    let timer = Instant::now();
    let (exit_circuit,withdraw_amount) =
        create_exit_circuit_with_public_input(
            circuit_account_tree,
            account_id,
            sub_account_id,
            l2_source_token,
            l1_target_token,
            chain_id,
            total_chain_num,
        );
    info!("Exit witness generated: {} s", timer.elapsed().as_secs());
    let commitment = exit_circuit
        .pub_data_commitment
        .expect("Witness should contract commitment");
    info!("Proof commitment: {:?}", commitment);

    let proof = gen_verified_proof_for_exit_circuit(&config, exit_circuit)
        .map_err(|e| format_err!("Failed to generate proof: {}", e))?;

    info!("Exit proof created: {} s", timer.elapsed().as_secs());
    Ok((proof.serialize_single_proof(), withdraw_amount))
}


/// Generates proof for exit given circuit using step-by-step algorithm.
pub fn gen_verified_proof_for_exit_circuit<C: Circuit<Engine> + Clone>(
    config: &RecoverStateConfig,
    circuit: C,
) -> Result<SingleProof, anyhow::Error> {
    let vk = VerificationKey::read(File::open(
        crate::fs_utils::get_exodus_verification_key_path(&config.runtime.key_dir)
    )?)?;

    info!("Proof for circuit started");

    let hints = transpile(circuit.clone())?;
    let setup = setup(circuit.clone(), &hints)?;
    let size_log2 = setup.n.next_power_of_two().trailing_zeros();

    let size_log2 = std::cmp::max(size_log2, SETUP_MIN_POW2); // for exit circuit
    let key_monomial_form = crate::fs_utils::get_universal_setup_monomial_form(
        &config.runtime.zklink_home,
        size_log2
    )?;

    let proof = prove_by_steps::<_, _, RollingKeccakTranscript<Fr>>(
        circuit,
        &hints,
        &setup,
        None,
        &key_monomial_form,
        None,
    )?;

    let valid = verify::<_, _, RollingKeccakTranscript<Fr>>(&proof, &vk, None)?;
    anyhow::ensure!(valid, "proof for exit is invalid");

    info!("Proof for circuit successful");
    Ok(proof.into())
}
