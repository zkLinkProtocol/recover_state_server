#![allow(dead_code)]
use super::{SETUP_MAX_POW2, SETUP_MIN_POW2};
use anyhow::format_err;
use std::fs::{create_dir_all, remove_file, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use tracing::info;
use zklink_crypto::bellman::kate_commitment::{Crs, CrsForLagrangeForm, CrsForMonomialForm};
use zklink_crypto::bellman::plonk::{make_verification_key, setup, transpile_with_gates_count};
use zklink_crypto::franklin_crypto::bellman::plonk::better_cs::adaptor::*;
use zklink_crypto::franklin_crypto::bellman::Circuit;
use zklink_crypto::Engine;

/// Generates PLONK verification key for given circuit and saves key at the given path.
/// Returns used setup power of two. (e.g. 22)
fn generate_verification_key<C: Circuit<Engine> + Clone, P: AsRef<Path>>(
    zklink_home: &str,
    circuit: C,
    path: P,
) -> u32 {
    let path = path.as_ref();
    assert!(
        !path.exists(),
        "path for saving verification key exists: {}",
        path.display()
    );
    {
        println!("path is {:?}", path);
        let parent_dir = path.parent().unwrap();
        if !parent_dir.exists() {
            create_dir_all(parent_dir).expect("can't create verification key parent dir");
        }
        File::create(path).expect("can't create file at verification key path ");
        remove_file(path).unwrap_or_default()
    }

    info!("Transpiling circuit");
    let (gates_count, mut transpilation_hints) =
        transpile_with_gates_count(circuit.clone()).expect("failed to transpile");
    let _size_log2 = gates_count.next_power_of_two().trailing_zeros();
    println!("Transpiled into {} gates", gates_count);

    let mut tmp_buff = Vec::new();
    write_transpilation_hints(&transpilation_hints, &mut tmp_buff).expect("hint write");
    transpilation_hints = read_transpilation_hints(tmp_buff.as_slice()).expect("hint read");

    let mut hints_hist = std::collections::HashMap::new();
    hints_hist.insert("into addition gate".to_owned(), 0);
    hints_hist.insert("merge LC".to_owned(), 0);
    hints_hist.insert("into quadratic gate".to_owned(), 0);
    hints_hist.insert("into multiplication gate".to_owned(), 0);

    for (_, h) in transpilation_hints.iter() {
        match h {
            TranspilationVariant::IntoQuadraticGate => {
                *hints_hist
                    .get_mut(&"into quadratic gate".to_owned())
                    .unwrap() += 1;
            }
            TranspilationVariant::MergeLinearCombinations(..) => {
                *hints_hist.get_mut(&"merge LC".to_owned()).unwrap() += 1;
            }
            TranspilationVariant::IntoAdditionGate(..) => {
                *hints_hist
                    .get_mut(&"into addition gate".to_owned())
                    .unwrap() += 1;
            }
            TranspilationVariant::IntoMultiplicationGate(..) => {
                *hints_hist
                    .get_mut(&"into multiplication gate".to_owned())
                    .unwrap() += 1;
            }
        }
    }
    println!("Transpilation hist = {:?}", hints_hist);

    let size_log2 = gates_count.next_power_of_two().trailing_zeros();
    assert!(
        size_log2 <= 26,
        "power of two too big {}, max: 26",
        size_log2
    );

    // exodus circuit is to small for the smallest setup
    let size_log2 = std::cmp::max(20, size_log2);
    info!(
        "Reading setup file, gates_count: {}, pow2: {}",
        gates_count, size_log2
    );

    let key_monomial_form = get_universal_setup_monomial_form(zklink_home, size_log2)
        .expect("Failed to read setup file.");

    info!("Generating setup");
    let setup = setup(circuit, &transpilation_hints).expect("failed to make setup");
    info!("Generating verification key");
    let verification_key = make_verification_key(&setup, &key_monomial_form)
        .expect("failed to create verification key");
    verification_key
        .write(File::create(path).unwrap())
        .expect("Failed to write verification file."); // unwrap - checked at the function entry
    info!("Verification key successfully generated");
    size_log2
}

/// Returns universal setup in the monomial form of the given power of two (range: SETUP_MIN_POW2..=SETUP_MAX_POW2). Checks if file exists
pub fn get_universal_setup_monomial_form(
    zklink_home: &str,
    power_of_two: u32,
) -> Result<Crs<Engine, CrsForMonomialForm>, anyhow::Error> {
    anyhow::ensure!(
        (SETUP_MIN_POW2..=SETUP_MAX_POW2).contains(&power_of_two),
        "setup power of two is not in the correct range"
    );
    // zklink for test
    let setup_file_name = format!("setup_2^{}.key", power_of_two);
    let mut buf_reader = get_universal_setup_file_buff_reader(zklink_home, &setup_file_name)?;
    Crs::<Engine, CrsForMonomialForm>::read(&mut buf_reader)
        .map_err(|e| format_err!("Failed to read Crs from setup file: {}", e))
}

/// Returns universal setup in lagrange form of the given power of two (range: SETUP_MIN_POW2..=SETUP_MAX_POW2). Checks if file exists
pub fn get_universal_setup_lagrange_form(
    power_of_two: u32,
    zklink_home: &str,
) -> Result<Crs<Engine, CrsForLagrangeForm>, anyhow::Error> {
    anyhow::ensure!(
        (SETUP_MIN_POW2..=SETUP_MAX_POW2).contains(&power_of_two),
        "setup power of two is not in the correct range"
    );
    let setup_file_name = format!("setup_2^{}_lagrange.key", power_of_two);
    let mut buf_reader = get_universal_setup_file_buff_reader(zklink_home, &setup_file_name)?;
    Crs::<Engine, CrsForLagrangeForm>::read(&mut buf_reader)
        .map_err(|e| format_err!("Failed to read Crs from setup file: {}", e))
}

pub fn get_exodus_verification_key_path(key_dir: &str) -> PathBuf {
    let mut key = PathBuf::new();
    key.push(key_dir);
    key.push("verification_exit.key");
    key
}

fn get_universal_setup_file_buff_reader(
    zklink_home: &str,
    setup_file_name: &str,
) -> Result<BufReader<File>, anyhow::Error> {
    let setup_file = {
        let mut path = base_universal_setup_dir(zklink_home)?;
        path.push(setup_file_name);
        File::open(path).map_err(|e| {
            format_err!(
                "Failed to open universal setup file {}, err: {}",
                setup_file_name,
                e
            )
        })?
    };
    Ok(BufReader::with_capacity(1 << 29, setup_file))
}

fn base_universal_setup_dir(zklink_home: &str) -> Result<PathBuf, anyhow::Error> {
    let mut dir = PathBuf::new();
    // root is used by default for provers
    dir.push(zklink_home);
    dir.push("zklink_keys");
    anyhow::ensure!(dir.exists(), "Universal setup dir does not exits");
    Ok(dir)
}
