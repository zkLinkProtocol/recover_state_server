use super::{SETUP_MAX_POW2, SETUP_MIN_POW2};
use anyhow::format_err;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use zklink_crypto::bellman::kate_commitment::{Crs, CrsForLagrangeForm, CrsForMonomialForm};
use zklink_crypto::Engine;

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
    Ok(Crs::<Engine, CrsForMonomialForm>::read(&mut buf_reader)
        .map_err(|e| format_err!("Failed to read Crs from setup file: {}", e))?)
}

/// Returns universal setup in lagrange form of the given power of two (range: SETUP_MIN_POW2..=SETUP_MAX_POW2). Checks if file exists
pub fn get_universal_setup_lagrange_form(
    power_of_two: u32,
    zklink_home: &str
) -> Result<Crs<Engine, CrsForLagrangeForm>, anyhow::Error> {
    anyhow::ensure!(
        (SETUP_MIN_POW2..=SETUP_MAX_POW2).contains(&power_of_two),
        "setup power of two is not in the correct range"
    );
    let setup_file_name = format!("setup_2^{}_lagrange.key", power_of_two);
    let mut buf_reader = get_universal_setup_file_buff_reader(zklink_home, &setup_file_name)?;
    Ok(Crs::<Engine, CrsForLagrangeForm>::read(&mut buf_reader)
        .map_err(|e| format_err!("Failed to read Crs from setup file: {}", e))?)
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
        path.push(&setup_file_name);
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