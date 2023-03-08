pub mod exit_proof;
pub mod fs_utils;
pub mod exodus_prover;

pub use exodus_prover::ExodusProver;

pub const SETUP_MIN_POW2: u32 = 20;
pub const SETUP_MAX_POW2: u32 = 26;
