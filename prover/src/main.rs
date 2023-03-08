//! Generate exit proof for exodus mode given account and token
//! correct verified state should be present in the db (could be restored using `data-restore` module)

use serde::Serialize;
use std::time::Instant;
use structopt::StructOpt;
use tracing::info;
use zklink_basic_types::{ChainId, SubAccountId};
use zklink_crypto::proof::EncodedSingleProof;
use zklink_prover::exit_proof::create_exit_proof;
use zklink_storage::ConnectionPool;
use zklink_utils::BigUintSerdeWrapper;
use zklink_types::params::MAX_CHAIN_ID;
use recover_state_config::{DBConfig, RecoverStateConfig};
use zklink_prover::exodus_prover::ExitInfo;
use zklink_prover::ExodusProver;

#[derive(StructOpt)]
#[structopt(
    name = "ZkLink exodus prover",
    author = "N Labs",
    rename_all = "snake_case"
)]
struct Opt {
    /// Chain to withdraw - "1"
    #[structopt(short = "c", long = "chain")]
    chain_id: u8,
    /// Account id of the account
    #[structopt(long)]
    account_id: u32,
    /// SubAccount id of the account
    #[structopt(long)]
    sub_account_id: u8,
    /// Target token to withdraw - token id of "USDT"
    #[structopt(long)]
    l1_target_token: u16,
    /// Source token to withdraw - token id of "USD"
    #[structopt(long)]
    l2_source_token: u16,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().expect(".env file not found");

    let opt = Opt::from_args();
    assert!(opt.chain_id <= *MAX_CHAIN_ID);

    info!("Construct exit info.");
    let exit_info = ExitInfo{
        chain_id: opt.account_id.into(),
        account_address: Default::default(),
        account_id: opt.chain_id.into(),
        sub_account_id: opt.sub_account_id.into(),
        l1_target_token: opt.l1_target_token.into(),
        l2_source_token: opt.l2_source_token.into(),
    };
    let recover_state_config = RecoverStateConfig::from_env();
    let prover = ExodusProver::new(recover_state_config);
    let (stored_block_info, proof_data) = prover
        .create_proof(exit_info)
        .await
        .expect("Failed to create proof");

    println!("\n\n");
    println!("==========================");
    println!("Generating proof completed!");
    println!("Below you can see the input data for the exit transaction on ZkLink contract");
    println!(
        "Look up the manuals of your desired smart wallet in order to know how to sign \
        and send this transaction to the blockchain of {:?}", proof_data.exit_info.chain_id
    );
    println!("==========================");

    println!("Exit transaction inputs:");

    println!(
        "store_block_info: {}",
        serde_json::to_string_pretty(&stored_block_info).expect("proof data serialize")
    );
    println!(
        "exit_proof_data: {}",
        serde_json::to_string_pretty(&proof_data).expect("proof data serialize")
    );
}