//! Generate exit proof for exodus mode given account and token
//! correct verified state should be present in the db (could be restored using `data-restore` module)

use serde::Serialize;
use std::time::Instant;
use structopt::StructOpt;
use tracing::info;
use zklink_basic_types::{ChainId, SubAccountId};
use zklink_crypto::proof::EncodedSingleProof;
use zklink_storage::ConnectionPool;
use zklink_utils::BigUintSerdeWrapper;
use zklink_types::params::MAX_CHAIN_ID;
use recover_state_config::{DBConfig, RecoverStateConfig};
use zklink_prover::exit_type::ExitInfo;
use zklink_prover::{ExodusProver, run_exodus_prover};

#[derive(StructOpt)]
#[structopt(
    name = "ZkLink exodus prover",
    author = "N Labs",
    rename_all = "snake_case"
)]
enum Opt {
    /// Runs prover tasks module(Running programmer)
    #[structopt(name = "tasks")]
    Tasks,
    /// Generates a single proof based on the specified exit information(Command tool)
    #[structopt(name = "single")]
    Single {
        /// Chain to withdraw - "1"
        #[structopt(short = "c", long = "chain-id")]
        chain_id: u8,
        /// Account id of the account - "0"(can't be negative or 1)
        #[structopt(short = "a", long = "account-id")]
        account_id: u32,
        /// SubAccount id of the account - "0"
        #[structopt(short = "s", long = "sub-account-id")]
        sub_account_id: u8,
        /// Target token to withdraw - token id of "USDT"
        #[structopt(short = "l1", long = "l1-target-token")]
        l1_target_token: u16,
        /// Source token to withdraw - token id of "USD"
        #[structopt(short = "l2", long = "l2-source-token")]
        l2_source_token: u16,
    },
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().expect(".env file not found");

    let opt = Opt::from_args();
    let recover_state_config = RecoverStateConfig::from_env();

    match opt{
        Opt::Tasks => {
            info!("Run the task mode of exodus prover for exit proof tasks!");
            run_exodus_prover(recover_state_config).await;
        }
        Opt::Single {
            chain_id,
            account_id,
            sub_account_id,
            l1_target_token,
            l2_source_token
        } => {
            info!("Run the command mode of exodus command for generating single exit proof!");
            info!("Construct exit info");
            let exit_info = ExitInfo{
                chain_id: account_id.into(),
                account_address: Default::default(),
                account_id: chain_id.into(),
                sub_account_id: sub_account_id.into(),
                l1_target_token: l1_target_token.into(),
                l2_source_token: l2_source_token.into(),
            };
            let prover = ExodusProver::new(recover_state_config).await;

            info!("Start proving");
            let timer = Instant::now();
            let proof_data = prover
                .create_exit_proof(exit_info)
                .expect("Failed to create exit proof");
            info!("End proving, elapsed time: {} s", timer.elapsed().as_secs());

            let stored_block_info = prover.last_executed_block.stored_block_info(exit_info.chain_id);

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
    }
}