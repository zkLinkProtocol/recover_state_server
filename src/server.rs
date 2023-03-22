use actix_web::{web, App, HttpResponse, HttpServer};
use recover_state_config::RecoverStateConfig;
use zklink_prover::{ExitInfo as ExitRequest};
use crate::request::{BalanceRequest, StoredBlockInfoRequest, TokenRequest};
use crate::ServerData;
use crate::utils::{BatchExitInfo as BatchExitRequest};

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
        .await?
    {
        Some(token) => HttpResponse::Ok().json(token),
        None => HttpResponse::NotFound().body("Token not found"),
    };
    Ok(response)
}

/// Get the ZkLink contract addresses of all blockchain.
async fn get_stored_block_info(
    block_info_request: web::Json<StoredBlockInfoRequest>,
    data: web::Data<ServerData>
) -> actix_web::Result<HttpResponse> {
    let chain_id = block_info_request.into_inner().chain_id;
    let response = match data.get_ref()
        .recovered_state
        .stored_block_info(chain_id)
    {
        Some(contracts) => HttpResponse::Ok().json(contracts),
        None => HttpResponse::NotFound().body("The Chain not found")
    };
    Ok(response)
}

/// Get balances fo all token by ZkLinkAddress
async fn get_balances(
    balance_request: web::Json<BalanceRequest>,
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
    let account_address = balance_request.into_inner().address;
    let response = match data.get_ref()
        .recovered_state
        .get_balances_by_cache(account_address)
        .await?
    {
        Some(balances) => HttpResponse::Ok().json(balances),
        None => HttpResponse::NotFound().body("Account not found"),
    };
    Ok(response)
}

/// Get the proof by the specified exit info.
async fn get_proof_by_info(
    exit_request: web::Json<ExitRequest>,
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
    let exit_info = exit_request.into_inner();
    let response = match data.get_ref()
        .get_proof(exit_info)
        .await?
    {
        Some(proof) => HttpResponse::Ok().json(proof),
        None => HttpResponse::NotFound().body("Exit proof task not found"),
    };
    Ok(response)
}

/// Get all proofs of all blockchain by the specified ZkLinkAddress and TokenId.
async fn get_proofs_by_token(
    batch_exit_info: web::Json<BatchExitRequest>,
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
    let exit_info = batch_exit_info.into_inner();
    let response = match data.get_ref()
        .get_proofs(exit_info)
        .await?
    {
        Some(proofs) => HttpResponse::Ok().json(proofs),
        None => HttpResponse::NotFound().body("Account not found"),
    };
    Ok(response)
}

/// Request to generate single proof for the specified exit info.
async fn generate_proof_task_by_info(
    exit_request: web::Json<ExitRequest>,
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
    let exit_info = exit_request.into_inner();
    data.get_ref()
        .generate_proof_task(exit_info)
        .await?;
    Ok(HttpResponse::Ok().finish())
}

/// Request to generate batch proofs of all blockchain for the specified token.
async fn generate_proof_tasks_by_token(
    batch_exit_info: web::Json<BatchExitRequest>,
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
    let exit_info = batch_exit_info.into_inner();
    data.get_ref()
        .generate_proof_tasks(exit_info)
        .await?;
    Ok(HttpResponse::Ok().finish())
}

pub async fn run_server(config: RecoverStateConfig) -> std::io::Result<()> {
    let addrs = config.api.bind_addr();
    let num = config.api.workers_num;
    let server_data = ServerData::new(config).await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server_data.clone()))
            .route("/contracts", web::get().to(get_contracts))
            .route("/tokens", web::get().to(get_tokens))
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
