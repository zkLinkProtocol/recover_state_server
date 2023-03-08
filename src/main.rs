use structopt::StructOpt;
use recover_state_config::RecoverStateConfig;
use recover_state_server::{run_server};

#[derive(StructOpt)]
#[structopt(name = "Recover state server", author = "N Labs", rename_all = "snake_case")]
struct ServerOpt {
    /// Recovers ZkLink `Executed` state from ZkLink contract in all chains.
    #[structopt(long)]
    init: bool,

    /// Runs the Recover State Server for user to access proof.
    #[structopt(long = "server", name = "server")]
    server: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let opt = ServerOpt::from_args();

    let config = RecoverStateConfig::from_env();
    if opt.init{
        todo!()
    }
    if opt.server{
        run_server(config).await.unwrap();
    }
}
