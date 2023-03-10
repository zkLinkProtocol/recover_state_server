use actix_web::{web, App, HttpResponse, HttpServer};
use recover_state_config::RecoverStateConfig;
use zklink_prover::ExitInfo;
use zklink_types::{TokenId, ZkLinkAddress};
use crate::{AccountQuery, ServerData};

async fn get_balance(
    account: web::Path<AccountQuery>,
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
    let account = account.into_inner();
    let response = match data.get_ref()
        .get_balances(account)
        .await?
    {
        Some(balances) => HttpResponse::Ok().json(balances),
        None => HttpResponse::NotFound().body("Account not found"),
    };
    Ok(response)
}

async fn get_proof_by_info(
    exit_info: web::Json<ExitInfo>,
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
    let exit_info = exit_info.into_inner();
    let response = match data.get_ref()
        .get_proof(exit_info)
        .await?
    {
        Some(_account) => HttpResponse::Ok().finish(),
        None => HttpResponse::NotFound().body("Account not found"),
    };
    Ok(response)
}

async fn get_proofs_by_token(
    account: web::Path<ZkLinkAddress>,
    token: web::Path<TokenId>,
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
    let account_address = account.into_inner();
    let token_id = token.into_inner();
    let response = match data.get_ref()
        .get_proofs(account_address, token_id)
        .await?
    {
        Some(_account) => HttpResponse::Ok().finish(),
        None => HttpResponse::NotFound().body("Account not found"),
    };
    Ok(response)
}

async fn request_generate_proof_task(
    exit_info: web::Json<ExitInfo>,
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
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

async fn get_contracts(data: web::Data<ServerData>) -> actix_web::Result<HttpResponse> {
    let contracts = data.get_ref().get_contracts();
    Ok(HttpResponse::Ok().json(contracts))
}

pub async fn run_server(config: RecoverStateConfig) -> std::io::Result<()> {
    let addrs = config.api.bind_addr();
    let server_data = ServerData::new(config).await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server_data.clone()))
            .route("/get_balance", web::get().to(get_balance))
            .route("/get_proof_by_info", web::get().to(get_proof_by_info))
            .route("/get_proofs_by_token", web::get().to(get_proofs_by_token))
            .route("/get_contracts", web::get().to(get_contracts))
            .route("/request_generate_proof", web::post().to(request_generate_proof_task))
    })
        .bind(addrs)?
        .run()
        .await
}
