use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::dev::Service;
use actix_web::{web, App, HttpResponse, HttpServer};

use recover_state_config::RecoverStateConfig;
use zklink_prover::ExitInfo as ExitRequest;
use zklink_storage::ConnectionPool;

use crate::app_data::{ProofsCache, RecoverProgress};
use crate::request::{
    BalanceRequest, BatchExitRequest, ProofsRequest, StoredBlockInfoRequest, TokenRequest,
    UnprocessedDepositRequest,
};
use crate::response::{ExodusResponse, ExodusStatus};
use crate::AppData;

/// Get the ZkLink contract addresses of all blockchain.
async fn get_contracts(data: web::Data<Arc<AppData>>) -> actix_web::Result<HttpResponse> {
    let contracts = data.get_contracts();
    Ok(HttpResponse::Ok().json(contracts))
}

/// Get the info of all tokens.
async fn get_tokens(data: web::Data<Arc<AppData>>) -> actix_web::Result<HttpResponse> {
    if !data.acquired_tokens.initialized() {
        let response: ExodusResponse<()> = ExodusStatus::RecoverStateUnfinished.into();
        return Ok(HttpResponse::Ok().json(response));
    }
    let response = data.acquired_tokens().tokens();
    Ok(HttpResponse::Ok().json(response))
}

/// Request to get recover state progress.
async fn recover_progress(data: web::Data<Arc<AppData>>) -> actix_web::Result<HttpResponse> {
    let response = match data.get_recover_progress().await {
        Ok(progress) => ExodusResponse::Ok().data(progress),
        Err(err) => err.into(),
    };
    Ok(HttpResponse::Ok().json(response))
}

/// Request to get max running task id.
async fn running_max_task_id(data: web::Data<Arc<AppData>>) -> actix_web::Result<HttpResponse> {
    let response = match data.running_max_task_id().await {
        Ok(task_id) => ExodusResponse::Ok().data(task_id),
        Err(err) => err.into(),
    };
    Ok(HttpResponse::Ok().json(response))
}

/// Request to get pending tasks count.
async fn pending_tasks_count(data: web::Data<Arc<AppData>>) -> actix_web::Result<HttpResponse> {
    let response = match data.pending_tasks_count().await {
        Ok(pending_tasks_count) => ExodusResponse::Ok().data(pending_tasks_count),
        Err(err) => err.into(),
    };
    Ok(HttpResponse::Ok().json(response))
}

/// Get token info(supported chains, token's contract addresses) by token_id
async fn get_token(
    token_request: web::Json<TokenRequest>,
    data: web::Data<Arc<AppData>>,
) -> actix_web::Result<HttpResponse> {
    let token_id = token_request.into_inner().token_id;
    let response = match data.acquired_tokens().get_token(token_id).await {
        Ok(token) => ExodusResponse::Ok().data(token),
        Err(err) => err.into(),
    };
    Ok(HttpResponse::Ok().json(response))
}

/// Get the ZkLink contract addresses of all blockchain.
async fn get_stored_block_info(
    block_info_request: web::Json<StoredBlockInfoRequest>,
    data: web::Data<Arc<AppData>>,
) -> actix_web::Result<HttpResponse> {
    let chain_id = block_info_request.into_inner().chain_id;
    let response = match data.get_stored_block_info(chain_id) {
        Ok(stored_block_info) => ExodusResponse::Ok().data(stored_block_info),
        Err(err) => err.into(),
    };
    Ok(HttpResponse::Ok().json(response))
}

/// Get balances of all token by ZkLinkAddress
async fn get_balances(
    balance_request: web::Json<BalanceRequest>,
    data: web::Data<Arc<AppData>>,
) -> actix_web::Result<HttpResponse> {
    let account_address = balance_request.into_inner().address;
    let response = match data
        .recovered_state()
        .get_balances_by_cache(account_address)
        .await
    {
        Ok(balances) => ExodusResponse::Ok().data(balances),
        Err(err) => err.into(),
    };
    Ok(HttpResponse::Ok().json(response))
}

/// Get all unprocessed priority ops of target chain  by chain_id
async fn get_unprocessed_priority_ops(
    unprocessed_deposit_request: web::Json<UnprocessedDepositRequest>,
    data: web::Data<Arc<AppData>>,
) -> actix_web::Result<HttpResponse> {
    let chain_id = unprocessed_deposit_request.into_inner().chain_id;
    let response = match data.get_unprocessed_priority_ops(chain_id).await {
        Ok(ops) => ExodusResponse::Ok().data(ops),
        Err(err) => err.into(),
    };
    Ok(HttpResponse::Ok().json(response))
}

/// Get the specified number of proofs closer to the id by page(page 1,num 20 => proofs ids: 1~20)
async fn get_proofs_by_page(
    proofs_request: web::Json<ProofsRequest>,
    data: web::Data<Arc<AppData>>,
) -> actix_web::Result<HttpResponse> {
    let proof_info = proofs_request.into_inner();
    let response = match data
        .get_proofs_by_page(proof_info.page, proof_info.proofs_num)
        .await
    {
        Ok(proofs) => ExodusResponse::Ok().data(proofs),
        Err(err) => err.into(),
    };
    Ok(HttpResponse::Ok().json(response))
}

/// Get the proof by the specified exit info.
async fn get_proof_by_info(
    exit_request: web::Json<ExitRequest>,
    data: web::Data<Arc<AppData>>,
) -> actix_web::Result<HttpResponse> {
    let exit_info = exit_request.into_inner();
    let response = match data.get_proof(exit_info).await {
        Ok(proof) => ExodusResponse::Ok().data(proof),
        Err(err) => err.into(),
    };
    Ok(HttpResponse::Ok().json(response))
}

/// Get all proofs of all blockchain by the specified ZkLinkAddress and TokenId.
async fn get_proofs_by_token(
    batch_exit_info: web::Json<BatchExitRequest>,
    data: web::Data<Arc<AppData>>,
) -> actix_web::Result<HttpResponse> {
    let exit_info = batch_exit_info.into_inner();
    let response = match data.get_proofs(exit_info).await {
        Ok(proofs) => ExodusResponse::Ok().data(proofs),
        Err(err) => err.into(),
    };
    Ok(HttpResponse::Ok().json(response))
}

/// Request to generate single proof for the specified exit info.
async fn generate_proof_task_by_info(
    exit_request: web::Json<ExitRequest>,
    data: web::Data<Arc<AppData>>,
) -> actix_web::Result<HttpResponse> {
    let exit_info = exit_request.into_inner();
    let response = match data.generate_proof_task(exit_info).await {
        Ok(task_id) => ExodusResponse::Ok().data(task_id),
        Err(err) => err.into(),
    };
    Ok(HttpResponse::Ok().json(response))
}

/// Request to generate batch proofs of all blockchain for the specified token.
async fn generate_proof_tasks_by_token(
    batch_exit_info: web::Json<BatchExitRequest>,
    data: web::Data<Arc<AppData>>,
) -> actix_web::Result<HttpResponse> {
    let batch_exit_info = batch_exit_info.into_inner();
    let response = match data.generate_proof_tasks(batch_exit_info).await {
        Ok(tasks) => ExodusResponse::Ok().data(tasks),
        Err(err) => err.into(),
    };
    Ok(HttpResponse::Ok().json(response))
}

/// Request to get the task id(proof id)
async fn get_proof_task_id(
    task_info: web::Json<ExitRequest>,
    data: web::Data<Arc<AppData>>,
) -> actix_web::Result<HttpResponse> {
    let task_info = task_info.into_inner();
    let response = match data.get_proof_task_id(task_info).await {
        Ok(task_id) => ExodusResponse::Ok().data(task_id),
        Err(err) => err.into(),
    };
    Ok(HttpResponse::Ok().json(response))
}

const RECOVER_PROGRESS_PATH: &str = "/recover_progress";
const CONTRACTS_PATH: &str = "/contracts";
const GENERATE_PROOF_TASKS_BY_TOKEN: &str = "/generate_proof_tasks_by_token";

pub async fn run_server(config: RecoverStateConfig) -> std::io::Result<()> {
    let addrs = config.api.bind_addr();
    let num = config.api.workers_num;
    let enable_http_cors = config.api.enable_http_cors;
    let contracts = config.layer1.get_contracts();
    let clean_interval = config.clean_interval;
    let enable_black_list = clean_interval.is_some();

    let recover_progress = RecoverProgress::from_config(&config).await;
    let conn_pool = ConnectionPool::new(config.db.url, config.db.pool_size);
    let proofs_cache = ProofsCache::from_database(conn_pool.clone()).await;
    let app_data = Arc::new(
        AppData::new(
            enable_black_list,
            conn_pool.clone(),
            contracts,
            proofs_cache,
            recover_progress,
        )
        .await,
    );

    tokio::spawn(
        app_data
            .clone()
            .black_list_escaping(clean_interval.unwrap_or(0)),
    );
    tokio::spawn(app_data.clone().sync_recover_progress());

    HttpServer::new(move || {
        let cors = if enable_http_cors {
            Cors::permissive()
        } else {
            Cors::default()
        };
        App::new()
            .wrap_fn(|req, srv| {
                let data = req.app_data::<web::Data<Arc<AppData>>>().unwrap();

                let fut: Pin<Box<dyn Future<Output = Result<_, _>>>> = match req.path() {
                    RECOVER_PROGRESS_PATH | CONTRACTS_PATH => Box::pin(srv.call(req)),
                    GENERATE_PROOF_TASKS_BY_TOKEN => Box::pin(async move {
                        let response: ExodusResponse<()> =
                            ExodusStatus::ApiClosedTemporarily.into();
                        Ok(req.into_response(HttpResponse::Ok().json(response)))
                    }),
                    _ => {
                        if data.is_not_sync_completed() {
                            Box::pin(async move {
                                let response: ExodusResponse<()> =
                                    ExodusStatus::RecoverStateUnfinished.into();
                                Ok(req.into_response(HttpResponse::Ok().json(response)))
                            })
                        } else {
                            Box::pin(srv.call(req))
                        }
                    }
                };
                async move {
                    let res = fut.await?;
                    Ok(res)
                }
            })
            .wrap(cors)
            .app_data(web::Data::new(app_data.clone()))
            .configure(exodus_config)
    })
    .bind(addrs)?
    .workers(num)
    .run()
    .await
}

pub fn exodus_config(cfg: &mut web::ServiceConfig) {
    cfg.route(CONTRACTS_PATH, web::get().to(get_contracts))
        .route("/tokens", web::get().to(get_tokens))
        .route(RECOVER_PROGRESS_PATH, web::get().to(recover_progress))
        .route("/running_max_task_id", web::get().to(running_max_task_id))
        .route("/pending_tasks_count", web::get().to(pending_tasks_count))
        .route(
            "/get_unprocessed_priority_ops",
            web::post().to(get_unprocessed_priority_ops),
        )
        .route("/get_token", web::post().to(get_token))
        .route(
            "/get_stored_block_info",
            web::post().to(get_stored_block_info),
        )
        .route("/get_balances", web::post().to(get_balances))
        .route("/get_proofs_by_page", web::post().to(get_proofs_by_page))
        .route("/get_proof_by_info", web::post().to(get_proof_by_info))
        .route("/get_proofs_by_token", web::post().to(get_proofs_by_token))
        .route(
            "/generate_proof_task_by_info",
            web::post().to(generate_proof_task_by_info),
        )
        .route(
            GENERATE_PROOF_TASKS_BY_TOKEN,
            web::post().to(generate_proof_tasks_by_token),
        )
        .route("/get_proof_task_id", web::post().to(get_proof_task_id));
}
