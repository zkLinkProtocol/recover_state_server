use actix_web::{web, App, HttpResponse, HttpServer};
use tracing::info;
use recover_state_config::RecoverStateConfig;
use zklink_prover::ExitInfo;
use zklink_types::{ChainId, TokenId, ZkLinkAddress};
use crate::ServerData;
use crate::utils::BatchExitInfo;

/// Get the ZkLink contract addresses of all blockchain.
async fn get_contracts(data: web::Data<ServerData>) -> actix_web::Result<HttpResponse> {
    let contracts = data.get_ref().get_contracts();
    Ok(HttpResponse::Ok().json(contracts))
}

/// Get token info(supported chains, token's contract addresses) by token_id
async fn get_token(
    token_id : web::Json<TokenId>,
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
    let token_id = token_id.into_inner();
    let response = match data.get_ref()
        .get_token(token_id)
        .await?
    {
        Some(token) => HttpResponse::Ok().json(token),
        None => HttpResponse::NotFound().body("Token not found"),
    };
    Ok(response)
}

/// Get the ZkLink contract addresses of all blockchain.
async fn get_stored_block_info(chain_id: web::Json<ChainId>, data: web::Data<ServerData>) -> actix_web::Result<HttpResponse> {
    let chain_id = chain_id.into_inner();
    let contracts = data.get_ref().get_stored_block_info(chain_id);
    Ok(HttpResponse::Ok().json(contracts))
}

/// Get balances fo all token by ZkLinkAddress
async fn get_balances(
    account_address: web::Json<ZkLinkAddress>,
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
    info!("Call get_balance api");
    let account_address = account_address.into_inner();
    let response = match data.get_ref()
        .get_balances(account_address)
        .await?
    {
        Some(balances) => HttpResponse::Ok().json(balances),
        None => HttpResponse::NotFound().body("Account not found"),
    };
    Ok(response)
}

/// Get the proof by the specified exit info.
async fn get_proof_by_info(
    exit_info: web::Json<ExitInfo>,
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
    info!("Call get_proof_by_info api");
    let exit_info = exit_info.into_inner();
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
    exit_info: web::Json<BatchExitInfo>,
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
    info!("Call get_proofs_by_token api");
    let exit_info = exit_info.into_inner();
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
    exit_info: web::Json<ExitInfo>,
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
    info!("Call generate_proof_task api");
    let exit_info = exit_info.into_inner();
    let response = match data.get_ref()
        .generate_proof_task(exit_info)
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().body(format!("{}", e)),
    };
    Ok(response)
}

/// Request to generate batch proofs of all blockchain for the specified token.
async fn generate_proof_tasks_by_token(
    exit_info: web::Json<BatchExitInfo>,
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
    info!("Call generate_proof_task api");
    let exit_info = exit_info.into_inner();
    let Some(token_info) = data.get_ref().token_by_id.get(&exit_info.token_id) else {
        return Ok(HttpResponse::NotFound().body("Token not found"))
    };

    let response = match data.get_ref()
        .generate_proof_tasks(exit_info, token_info)
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().body(format!("{}", e)),
    };
    Ok(response)
}

pub async fn run_server(config: RecoverStateConfig) -> std::io::Result<()> {
    let addrs = config.api.bind_addr();
    let num = config.api.workers_num;
    let server_data = ServerData::new(config).await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server_data.clone()))
            .route("/get_contracts", web::get().to(get_contracts))
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
