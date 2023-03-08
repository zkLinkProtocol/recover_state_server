//! Generate exit proof for exodus mode given account and token
//! correct verified state should be present in the db (could be restored using `data-restore` module)

use serde::Serialize;
use std::time::Instant;
use structopt::StructOpt;
use zklink_basic_types::{ChainId, SubAccountId};
use zklink_crypto::proof::EncodedSingleProof;
use zklink_prover::exit_proof::create_exit_proof;
use zklink_storage::ConnectionPool;
use zklink_types::{AccountId, ZkLinkAddress, TokenId};
use zklink_utils::BigUintSerdeWrapper;
use zklink_types::params::MAX_CHAIN_ID;
use recover_state_config::{DBConfig, RecoverStateConfig};

#[derive(Serialize, Debug)]
struct ExitProofData {
    l1_target_token: TokenId,
    l2_source_token: TokenId,
    account_id: AccountId,
    chain_id: ChainId,
    account_address: ZkLinkAddress,
    sub_account_id: SubAccountId,
    amount: BigUintSerdeWrapper,
    proof: EncodedSingleProof,
}

#[derive(StructOpt)]
#[structopt(
name = "zklink operator node",
author = "N Labs",
rename_all = "snake_case"
)]
struct Opt {
    /// Account id of the account
    #[structopt(long)]
    account_id: u32,
    /// SubAccount id of the account
    #[structopt(long)]
    sub_account_id: u8,
    /// Target token to withdraw - token id of "USDT"
    #[structopt(long)]
    l1_target_token: String,
    /// Source token to withdraw - token id of "USD"
    #[structopt(long)]
    l2_source_token: String,

    /// Chain to withdraw - "1"
    #[structopt(short = "c", long = "chain")]
    chain: u8,
}

#[tokio::main]
async fn main() {
    vlog::init();
    dotenvy::dotenv().expect(".env file not found");

    let opt = Opt::from_args();
    let account_id = opt.account_id;
    let chain_id = opt.chain;
    let sub_account_id = opt.sub_account_id;
    assert!(chain_id <= *MAX_CHAIN_ID);
    let l1_target_token:i32 = opt.l1_target_token.parse().unwrap();
    let l2_source_token:i32 = opt.l2_source_token.parse().unwrap();

    let timer = Instant::now();
    vlog::info!("Restoring state from db");

    let zklink_config = ZkLinkConfig::from_env();
    let db_config = DBConfig::from_env();


    println!("\n\n");
    println!("==========================");
    println!("Generating proof completed");
    println!("Below you can see the input data for the exit transaction on ZkLink contract");
    println!("Look up the manuals of your desired smart wallet in order to know how to sign and send this transaction to the Ethereum");
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