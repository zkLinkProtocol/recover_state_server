use actix_web::{http::StatusCode, test, web, App};
use recover_state_config::RecoverStateConfig;
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use zklink_storage::ConnectionPool;
use zklink_types::{ChainId, TokenId, ZkLinkAddress};

use crate::app_data::{
    AcquiredTokens, AppData, ExodusResponse, Progress, ProofsCache, RecoverProgress, TokenInfo,
};
use crate::request::TokenRequest;
use crate::server::exodus_config;

async fn create_app_data() -> AppData {
    dotenvy::dotenv().unwrap();
    let config = RecoverStateConfig::from_env();
    let recover_progress = RecoverProgress::new(&config).await;
    let conn_pool = ConnectionPool::new(config.db.url, config.db.pool_size);
    let proofs_cache = ProofsCache::from_database(conn_pool.clone()).await;
    let contracts = config.layer1.get_contracts();
    AppData::new(false, conn_pool, contracts, proofs_cache, recover_progress).await
}

// Initialize an instance of Recover Progress for testing
fn get_test_recover_progress() -> RecoverProgress {
    RecoverProgress {
        current_sync_height: AtomicU32::new(10),
        total_verified_block: 20.into(),
    }
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
    let expected_tokens = app_data.acquired_tokens().token_by_id.clone();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_data))
            .configure(exodus_config),
    )
    .await;

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
    let expected_token = TokenInfo {
        token_id: 1.into(),
        symbol: "USD".to_string(),
        addresses: Default::default(),
    };
    app_data
        .acquired_tokens
        .get_or_init(|| {
            let mut tokens = AcquiredTokens::default();
            tokens
                .token_by_id
                .insert(expected_token.token_id, expected_token.clone());
            tokens
        })
        .await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_data.clone()))
            .configure(exodus_config),
    )
    .await;

    let token_request = TokenRequest { token_id: 1.into() };
    let req = test::TestRequest::post()
        .uri("/get_token")
        .set_json(token_request)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let resp_body = test::read_body(resp).await;
    let resp_data: ExodusResponse<TokenInfo> = serde_json::from_slice(&resp_body).unwrap();
    assert_eq!(resp_data.code, 0);
    assert!(resp_data.err_msg.is_none());

    let token = resp_data.data.unwrap();
    assert_eq!(token, expected_token);
}

#[actix_rt::test]
async fn test_get_recover_progress() {
    // Create a test service instance
    let mut app_data = create_app_data().await;
    app_data.recover_progress = get_test_recover_progress().into();
    let mut app = test::init_service(App::new().app_data(exodus_config)).await;

    let req = test::TestRequest::get()
        .uri("/recover_progress")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let result: ExodusResponse<Progress> = test::read_body_json(resp).await;
    assert_eq!(result.data.unwrap().current_block, 0.into());
    assert_eq!(result.data.unwrap().total_verified_block, 20.into());
}
