use dotenvy::dotenv;
use structopt::StructOpt;
use tokio::sync::mpsc;
use tracing::info;
use zklink_crypto::convert::FeConvert;
use zklink_storage::ConnectionPool;
use offchain_recover_state::contract::ZkLinkEvmContract;
use offchain_recover_state::{database_storage_interactor::DatabaseStorageInteractor, END_BLOCK_OFFSET, get_fully_on_chain_zklink_contract, VIEW_BLOCKS_STEP};
use offchain_recover_state::data_restore_driver::RecoverStateDriver;
use offchain_recover_state::log::init;
use recover_state_config::{DBConfig, RecoverStateConfig};

#[derive(StructOpt)]
#[structopt(name = "Recover state driver", author = "N Labs", rename_all = "snake_case")]
struct Opt {
    /// Restores data with provided genesis (zero) block
    #[structopt(long)]
    genesis: bool,

    /// Continues data restoring
    #[structopt(long = "continue", name = "continue")]
    continue_mode: bool,

    /// Restore data until the last verified block and exit
    #[structopt(long)]
    finite: bool,

    /// Expected tree root hash after restoring. This argument is ignored if mode is not `finite`
    #[structopt(long)]
    final_hash: Option<String>,
}

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    let _sentry_guard = init();
    info!("Restoring zkLink state from the contract");

    let db_config = DBConfig::from_env();
    let connection_pool = ConnectionPool::new(db_config.url, db_config.pool_size);

    let opt = Opt::from_args();
    let config = RecoverStateConfig::from_env();

    let final_hash = opt.final_hash
        .filter(|_|opt.finite)
        .map(|value| FeConvert::from_hex(&value).expect("Can't parse the final hash"));

    let (deploy_block_number, zklink_contract) = get_fully_on_chain_zklink_contract(&config);
    let mut driver = RecoverStateDriver::new(
        zklink_contract,
        &config,
        VIEW_BLOCKS_STEP,
        END_BLOCK_OFFSET,
        opt.finite,
        final_hash,
        deploy_block_number,
        connection_pool.clone(),
    ).await;

    let storage = connection_pool.access_storage().await.unwrap();
    let mut interactor = DatabaseStorageInteractor::new(storage);
    // If genesis is argument is present - there will be fetching contracts creation transactions to get first layer1 block and genesis acc address
    if opt.genesis {
        driver.set_genesis_state(&mut interactor, config).await;
    }

    if opt.continue_mode && driver.load_state_from_storage(&mut interactor).await {
        std::process::exit(0);
    }

    let (token_sender, token_receiver) = mpsc::channel(100_000);
    driver.download_registered_tokens(token_sender).await;
    driver.recover_state(&mut interactor, token_receiver).await;
}
