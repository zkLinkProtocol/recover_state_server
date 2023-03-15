pub mod exit_proof;
pub mod fs_utils;
pub mod exodus_prover;
pub mod exit_type;
pub mod retries;

use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};
use recover_state_config::RecoverStateConfig;
use crate::retries::with_retries;
pub use exodus_prover::ExodusProver;
pub use exit_type::{ExitInfo, ExitProofData};

pub const SETUP_MIN_POW2: u32 = 20;
pub const SETUP_MAX_POW2: u32 = 26;

pub async fn run_exodus_prover(config: RecoverStateConfig){
    let prover = Arc::new(ExodusProver::new(config).await);
    let cpus_num = num_cpus::get();
    let mut workers = Vec::with_capacity(cpus_num);
    for i in 0..cpus_num{
        let prover = prover.clone();
        workers.push(tokio::spawn(async move {
            loop {
                match prover.load_new_task().await {
                    Ok(task) => if let Some(exit_info) = task{
                        process_task(prover.clone(), exit_info).await;
                    } else {
                        info!("Worker{} is waiting for the new exit proof task......", i);
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    },
                    Err(err) => warn!("Failed to load new task:{}", err)
                }
            }
        }));
    }
    let _ = futures::future::select_all(workers).await;
}

async fn process_task(
    prover: Arc<ExodusProver>,
    exit_info: ExitInfo,
) {
    let exit_info = prover.check_exit_info(exit_info).await;
    let (result_sender, result_receiver) = tokio::sync::oneshot::channel();
    let after_prover = prover.clone();
    let exit_info_clone = exit_info.clone();
    std::thread::spawn(move || {
        let prover_with_proof = prover.create_exit_proof(exit_info);
        result_sender.send(prover_with_proof).unwrap();
    });

    let result = result_receiver.await.unwrap();
    // Ensure that the tasks being run have a result(store or cancel)
    let op = || async {
         match result.as_ref(){
            Ok(exit_proof_data) => {
                after_prover.store_exit_proof(&exit_proof_data).await?;
                info!("Stored exit proof");
            }
            Err(error) => {
                error!("Failed to compute proof:{}", error);
                after_prover.cancel_this_task(&exit_info_clone).await?;
            }
        }
        Ok(())
    };
    with_retries(op)
        .await
        .expect("Failed to process this task");
}
