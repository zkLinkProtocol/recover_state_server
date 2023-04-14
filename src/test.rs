use std::collections::HashMap;
use actix_web::{http::{StatusCode}, test, App, web};
use zklink_types::{ChainId, TokenId, ZkLinkAddress};
use recover_state_config::RecoverStateConfig;
use zklink_storage::ConnectionPool;

use crate::{
    AppData, server::exodus_config,
    proofs_cache::ProofsCache,
    response::ExodusResponse
};
use crate::acquired_tokens::TokenInfo;
use crate::recover_progress::RecoverProgress;
use crate::request::TokenRequest;

async fn create_app_data() -> AppData {
    dotenvy::dotenv().unwrap();
    let config = RecoverStateConfig::from_env();
    let conn_pool = ConnectionPool::new(config.db.url, config.db.pool_size);
    let recover_progress = RecoverProgress::new(&config).await;
    let proofs_cache = ProofsCache::new(conn_pool.clone()).await;
    let contracts = config.layer1.get_contracts();
    AppData::new(conn_pool, contracts, proofs_cache, recover_progress).await
}

#[actix_rt::test]
async fn test_get_contracts() {
    let app_data = create_app_data().await;
    let expect_contracts = app_data.contracts.clone();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_data))
            .configure(exodus_config),
    )
        .await;

    let req = test::TestRequest::get().uri("/contracts").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let resp_body = test::read_body(resp).await;
    let resp_data: ExodusResponse<HashMap<ChainId, ZkLinkAddress>> =
        serde_json::from_slice(&resp_body).unwrap();

    assert_eq!(resp_data.code, 0);
    assert!(resp_data.err_msg.is_none());

    let contracts = resp_data.data.unwrap();

    assert_eq!(contracts, expect_contracts);
}

#[actix_rt::test]
async fn test_get_tokens() {
    let app_data = create_app_data().await;
    let expected_tokens = app_data.acquired_tokens.token_by_id.clone();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_data))
            .configure(exodus_config),
    ).await;

    let req = test::TestRequest::get().uri("/tokens").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let resp_body = test::read_body(resp).await;
    let resp_data: ExodusResponse<HashMap<TokenId, TokenInfo>> =
        serde_json::from_slice(&resp_body).unwrap();

    assert_eq!(resp_data.code, 0);
    assert!(resp_data.err_msg.is_none());

    let tokens = resp_data.data.unwrap();
    assert_eq!(tokens, expected_tokens);
}

#[actix_rt::test]
async fn test_get_token() {
    let mut app_data = create_app_data().await;
    let expected_token = TokenInfo{
        token_id: 1.into(),
        symbol: "USD".to_string(),
        addresses: Default::default(),
    };
    app_data.acquired_tokens.token_by_id.insert(expected_token.token_id, expected_token.clone());

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_data.clone()))
            .configure(exodus_config),
    ).await;

    let token_request = TokenRequest { token_id: 1.into() };
    let req = test::TestRequest::post()
        .uri("/get_token")
        .set_json(token_request)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let resp_body = test::read_body(resp).await;
    let resp_data: ExodusResponse<TokenInfo> =
        serde_json::from_slice(&resp_body).unwrap();
    assert_eq!(resp_data.code, 0);
    assert!(resp_data.err_msg.is_none());

    let token = resp_data.data.unwrap();
    assert_eq!(token, expected_token);
}