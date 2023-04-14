use recover_state_config::RecoverStateConfig;
use recover_state_server::run_server;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect(".env file not found");
    tracing_subscriber::fmt::init();

    let config = RecoverStateConfig::from_env();
    run_server(config).await.unwrap();
}
