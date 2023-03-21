use dotenvy::dotenv;
use structopt::StructOpt;
use tracing::info;
use zklink_crypto::convert::FeConvert;
use zklink_storage::ConnectionPool;
use offchain_recover_state::{
    database_storage_interactor::DatabaseStorageInteractor,
    END_BLOCK_OFFSET, VIEW_BLOCKS_STEP,
    get_fully_on_chain_zklink_contract,
};
use offchain_recover_state::data_restore_driver::RecoverStateDriver;
use offchain_recover_state::log::init;
use recover_state_config::RecoverStateConfig;

#[derive(StructOpt)]
#[structopt(name = "Recover state driver", author = "N Labs", rename_all = "snake_case")]
struct Opt {
    /// Restores data with provided genesis (zero) block
    #[structopt(long)]
    genesis: bool,

    /// Continues data restoring
    #[structopt(long = "continue", name = "continue")]
    continue_mode: bool,

    /// Restore data until the last verified block and exit, on by default,
    #[structopt(long, parse(try_from_str), default_value = "true")]
    finite: bool,

    /// Expected tree root hash after restoring. This argument is ignored if mode is not `finite`
    #[structopt(long)]
    final_hash: Option<String>,
}

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");
    let _sentry_guard = init();

    let opt: Opt = Opt::from_args();
    let config = RecoverStateConfig::from_env();

    let connection_pool = ConnectionPool::new(config.db.url.clone(), config.db.pool_size);
    let final_hash = opt.final_hash
        .filter(|_|opt.finite)
        .map(|value| FeConvert::from_hex(&value).expect("Can't parse the final hash"));

    info!("Restoring ZkLink state from the contract");
    // Init RecoverStateDriver
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

    // Init storage
    let storage = connection_pool.access_storage().await.unwrap();
    let mut interactor = DatabaseStorageInteractor::new(storage);

    if opt.genesis {
        // There will be fetching contracts creation transactions to get first layer1 block and genesis acc address
        driver.set_genesis_state(&mut interactor, config).await;

        // Get all token events
        driver.download_registered_tokens().await;
    }

    // Continue with recover_state as before
    if opt.continue_mode && driver.load_state_from_storage(&mut interactor).await {
        std::process::exit(0);
    }

    // Process block events
    driver.recover_state(&mut interactor).await;
}
