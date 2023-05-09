use crate::proving_cache::ProvingCache;
use crate::retries::with_retries;
pub use exit_type::{ExitInfo, ExitProofData};
pub use exodus_prover::ExodusProver;
use futures::FutureExt;
use offchain_recover_state::{contract::ZkLinkContract, get_fully_on_chain_zklink_contract};
use recover_state_config::RecoverStateConfig;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, sleep};
use tracing::{error, info, warn};
use zklink_storage::ConnectionPool;

pub mod exit_proof;
pub mod exit_type;
pub mod exodus_prover;
pub mod proving_cache;
pub mod retries;
pub mod utils;

pub const SETUP_MIN_POW2: u32 = 20;
pub const SETUP_MAX_POW2: u32 = 26;

pub async fn run_exodus_prover(config: RecoverStateConfig, workers_num: Option<usize>) {
    // Priority generate cache.
    let proving_cache =
        ProvingCache::from_config(&config).expect("Failed to generate proving cache");
    // clean old tasks
    let conn_pool = ConnectionPool::new(config.db.url.clone(), config.db.pool_size);
    tokio::spawn(clean_old_task(conn_pool));
    // And then wait completed recovered state.
    if config.enable_sync_mode {
        wait_recovered_state(&config).await;
    }

    let prover = Arc::new(ExodusProver::from_config(config, proving_cache).await);
    let core_num = num_cpus::get();
    let workers_num = workers_num.map_or(core_num / 16, |workers| workers.min(core_num));

    let mut workers = Vec::with_capacity(workers_num);
    for i in 0..workers_num {
        let prover = prover.clone();
        workers.push(tokio::spawn(async move {
            info!("Starting [Worker{}]", i);
            loop {
                match prover.load_new_task(i).await {
                    Ok(task) => {
                        if let Some((proof_id, exit_info)) = task {
                            process_task(prover.clone(), proof_id, exit_info).await;
                        } else {
                            info!("[Worker{}] is waiting for the new exit proof task......", i);
                            sleep(Duration::from_secs(5)).await;
                        }
                    }
                    Err(err) => warn!("[Worker{}] failed to load new task:{}", i, err),
                }
            }
        }));
    }
    let _ = futures::future::select_all(workers).await;
}

async fn wait_recovered_state(config: &RecoverStateConfig) {
    let conn_pool = ConnectionPool::new(config.db.url.clone(), config.db.pool_size);
    let mut storage = conn_pool.access_storage_with_retry().await;

    let (_, _, zklink_contract) = get_fully_on_chain_zklink_contract(config);
    let total_verified_block = zklink_contract
        .get_total_verified_blocks()
        .await
        .expect("Failed to get total verified blocks from zklink contract");
    let mut ticker = interval(Duration::from_secs(10));
    let mut verified_block_num = 0;
    info!("Sync recovering state started!");
    loop {
        ticker.tick().await;

        match storage.chain().block_schema().get_last_block_number().await {
            Ok(verified_block) => verified_block_num = verified_block as u32,
            Err(e) => warn!("Failed to get last block number from db: {}", e),
        }

        if verified_block_num >= total_verified_block {
            info!("Recovering state completed!");
            break;
        } else {
            info!(
                "Waiting to completed recovering state[cur:{}, total:{}]......",
                verified_block_num, total_verified_block
            );
        }
    }
}

async fn process_task(prover: Arc<ExodusProver>, proof_id: i64, exit_info: ExitInfo) {
    let exit_info_clone = exit_info.clone();
    let after_prover = prover.clone();

    let heartbeat_future = prover.clone().update_task_heartbeat(proof_id).fuse();
    let compute_future = async move {
        let exit_info = prover.check_exit_info(exit_info).await;
        let (result_sender, result_receiver) = tokio::sync::oneshot::channel();
        std::thread::spawn(move || {
            let prover_with_proof = prover.create_exit_proof(exit_info);
            result_sender.send(prover_with_proof).unwrap();
        });

        result_receiver.await.unwrap()
    }
    .fuse();
    futures::pin_mut!(compute_future, heartbeat_future);

    futures::select! {
        result = compute_future => {
            // Ensure that the tasks being run have a result(store or cancel)
            let op = || async {
                match result.as_ref() {
                    Ok(exit_proof_data) => {
                        after_prover.store_exit_proof(exit_proof_data).await?;
                        info!("Stored exit proof");
                    }
                    Err(error) => {
                        error!("Failed to compute proof:{}", error);
                        after_prover.cancel_this_task(&exit_info_clone).await?;
                    }
                }
                Ok(())
            };
            with_retries(op).await.expect("Failed to process this task");
        },
        _ = heartbeat_future => unreachable!(),
    }
}

async fn clean_old_task(conn_pool: ConnectionPool) {
    let mut storage = conn_pool.access_storage_with_retry().await;
    let mut clean_ticker = interval(Duration::from_secs(10));
    loop {
        if let Err(err) = storage.prover_schema().clean_old_task().await {
            warn!("Failed to update heartbeat time: {}", err);
        };

        clean_ticker.tick().await;
    }
}
