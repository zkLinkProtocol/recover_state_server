use crate::envy_load;
use serde::Deserialize;
use std::net::SocketAddr;

/// the api configuration of Recover State Server.
#[derive(Default, Debug, Deserialize, Clone, PartialEq)]
pub struct ApiConfig {
    /// Port to which the API server is listening.
    pub server_http_port: u16,
    /// Work threads num which the API server is listening.
    pub workers_num: usize,
    /// Enable cors cross-domain
    pub enable_http_cors: bool,
}

impl ApiConfig {
    pub fn from_env() -> Self {
        envy_load!("api", "API_CONFIG_")
    }

    pub fn bind_addr(&self) -> SocketAddr {
        SocketAddr::new("0.0.0.0".parse().unwrap(), self.server_http_port)
    }
}
