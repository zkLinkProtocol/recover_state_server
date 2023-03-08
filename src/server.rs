use actix_web::{web, App, HttpResponse, HttpServer};
use recover_state_config::RecoverStateConfig;
use zklink_types::{SubAccountId, TokenId};
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

async fn get_proof(
    account: web::Path<AccountQuery>,
    sub_account_id: web::Path<SubAccountId>,
    token_id: web::Path<TokenId>,
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
    let account = account.into_inner();
    let sub_account_id = sub_account_id.into_inner();
    let token_id = token_id.into_inner();
    let response = match data.get_ref()
        .get_proof(account, sub_account_id, token_id)
        .await?
    {
        Some(_account) => HttpResponse::Ok().finish(),
        None => HttpResponse::NotFound().body("Account not found"),
    };
    Ok(response)
}

async fn generate_proof_task(
    account: web::Path<AccountQuery>,
    sub_account_id: web::Path<SubAccountId>,
    token_id: web::Path<TokenId>,
    data: web::Data<ServerData>,
) -> actix_web::Result<HttpResponse> {
    let account = account.into_inner();
    let sub_account_id = sub_account_id.into_inner();
    let token_id = token_id.into_inner();
    let response = match data.get_ref()
        .generate_proof_task(account, sub_account_id, token_id)
        .await?
    {
        Some(_account) => HttpResponse::Ok().finish(),
        None => HttpResponse::NotFound().body("Account not found"),
    };
    Ok(response)
}

pub async fn run_server(config: RecoverStateConfig) -> std::io::Result<()> {
    let addrs = config.api.bind_addr();
    let server_data = ServerData::new(config);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server_data.clone()))
            .route("/get_balance", web::get().to(get_balance))
            .route("/get_proof", web::get().to(get_proof))
            .route("/request_generate_proof", web::get().to(generate_proof_task))
    })
        .bind(addrs)?
        .run()
        .await
}
