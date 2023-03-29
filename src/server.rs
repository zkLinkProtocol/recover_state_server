use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer};
use recover_state_config::RecoverStateConfig;
use zklink_prover::{ExitInfo as ExitRequest};
use zklink_storage::ConnectionPool;
use crate::proofs_cache::ProofsCache;
use crate::request::{BalanceRequest, StoredBlockInfoRequest, TokenRequest, BatchExitRequest, UnprocessedDepositRequest};
use crate::response::ExodusResponse;
use crate::ServerData;

/// Get the ZkLink contract addresses of all blockchain.
async fn get_contracts(data: web::Data<ServerData>) -> actix_web::Result<HttpResponse> {
    let contracts = data.get_ref().get_contracts();
    Ok(HttpResponse::Ok().json(contracts))
}

/// Get the info of all tokens.
async fn get_tokens(
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
    let response = data.get_ref()
        .acquired_tokens
        .tokens();
    Ok(HttpResponse::Ok().json(response))
}

/// Get token info(supported chains, token's contract addresses) by token_id
async fn get_token(
    token_request : web::Json<TokenRequest>,
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
    let token_id = token_request.into_inner().token_id;
    let response = match data.get_ref()
        .acquired_tokens
        .get_token(token_id)
        .await
    {
        Ok(token) => ExodusResponse::Ok().data(token),
        Err(err) => err.into()
    };
    Ok(HttpResponse::Ok().json(response))
}

/// Get the ZkLink contract addresses of all blockchain.
async fn get_stored_block_info(
    block_info_request: web::Json<StoredBlockInfoRequest>,
    data: web::Data<ServerData>
) -> actix_web::Result<HttpResponse> {
    let chain_id = block_info_request.into_inner().chain_id;
    let response = match data
        .get_ref()
        .get_stored_block_info(chain_id)
    {
        Ok(stored_block_info) => ExodusResponse::Ok().data(stored_block_info),
        Err(err) => err.into()
    };
    Ok(HttpResponse::Ok().json(response))
}

/// Get balances of all token by ZkLinkAddress
async fn get_balances(
    balance_request: web::Json<BalanceRequest>,
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
    let account_address = balance_request.into_inner().address;
    let response = match data.get_ref()
        .recovered_state
        .get_balances_by_cache(account_address)
        .await
    {
        Ok(balances) => ExodusResponse::Ok().data(balances),
        Err(err) => err.into()
    };
    Ok(HttpResponse::Ok().json(response))
}

/// Get all unprocessed priority ops of target chain  by chain_id
async fn get_unprocessed_priority_ops(
    unprocessed_deposit_request: web::Json<UnprocessedDepositRequest>,
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
    let chain_id = unprocessed_deposit_request.into_inner().chain_id;
    let response = match data.get_ref()
        .get_unprocessed_priority_ops(chain_id)
        .await
    {
        Ok(ops) => ExodusResponse::Ok().data(ops),
        Err(err) => err.into()
    };
    Ok(HttpResponse::Ok().json(response))
}

/// Get the proof by the specified exit info.
async fn get_proof_by_info(
    exit_request: web::Json<ExitRequest>,
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
    let exit_info = exit_request.into_inner();
    let response = match data.get_ref()
        .get_proof(exit_info)
        .await
    {
        Ok(proof) => ExodusResponse::Ok().data(proof),
        Err(err) => err.into()
    };
    Ok(HttpResponse::Ok().json(response))
}

/// Get all proofs of all blockchain by the specified ZkLinkAddress and TokenId.
async fn get_proofs_by_token(
    batch_exit_info: web::Json<BatchExitRequest>,
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
    let exit_info = batch_exit_info.into_inner();
    let response = match data.get_ref()
        .get_proofs(exit_info)
        .await
    {
        Ok(proofs) => ExodusResponse::Ok().data(proofs),
        Err(err) => err.into()
    };
    Ok(HttpResponse::Ok().json(response))
}

/// Request to generate single proof for the specified exit info.
async fn generate_proof_task_by_info(
    exit_request: web::Json<ExitRequest>,
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
    let exit_info = exit_request.into_inner();
    let response = match data.get_ref()
        .generate_proof_task(exit_info)
        .await
    {
        Ok(_) => ExodusResponse::<()>::Ok(),
        Err(err) => err.into()
    };
    Ok(HttpResponse::Ok().json(response))
}

/// Request to generate batch proofs of all blockchain for the specified token.
async fn generate_proof_tasks_by_token(
    batch_exit_info: web::Json<BatchExitRequest>,
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
    let batch_exit_info = batch_exit_info.into_inner();
    let response = match data.get_ref()
        .generate_proof_tasks(batch_exit_info)
        .await
    {
        Ok(_) => ExodusResponse::<()>::Ok(),
        Err(err) => err.into()
    };
    Ok(HttpResponse::Ok().json(response))
}

pub async fn run_server(config: RecoverStateConfig) -> std::io::Result<()> {
    let addrs = config.api.bind_addr();
    let num = config.api.workers_num;
    let enable_http_cors = config.api.enable_http_cors;
    let contracts = config.layer1.get_contracts();
    let conn_pool = ConnectionPool::new(config.db.url, config.db.pool_size);

    let proofs_cache = ProofsCache::new(conn_pool.clone()).await;
    let server_data = ServerData::new(conn_pool, contracts, proofs_cache).await;

    HttpServer::new(move || {
        let cors = if enable_http_cors {
            Cors::permissive()
        } else {
            Cors::default()
        };
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(server_data.clone()))
            .route("/contracts", web::get().to(get_contracts))
            .route("/tokens", web::get().to(get_tokens))
            .route("/get_unprocessed_priority_ops", web::post().to(get_unprocessed_priority_ops))
            .route("/get_token", web::post().to(get_token))
            .route("/get_stored_block_info", web::post().to(get_stored_block_info))
            .route("/get_balances", web::post().to(get_balances))

            .route("/get_proof_by_info", web::post().to(get_proof_by_info))
            .route("/get_proofs_by_token", web::post().to(get_proofs_by_token))
            .route("/generate_proof_task_by_info", web::post().to(generate_proof_task_by_info))
            .route("/generate_proof_tasks_by_token", web::post().to(generate_proof_tasks_by_token))
    })
        .bind(addrs)?
        .workers(num)
        .run()
        .await
}
