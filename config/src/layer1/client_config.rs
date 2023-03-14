use std::time::Duration;
// External uses
use serde::Deserialize;
// Local uses
use crate::envy_load;

/// Configuration for the Ethereum gateways.
#[derive(Default, Debug, Deserialize, Clone, PartialEq)]
pub struct ClientConfig {
    /// Numeric identifier of the L1 network (e.g. `9` for localhost).
    pub chain_id: u32,
    /// Address of the Ethereum node API.
    pub web3_url: Vec<String>,
    /// As `infura` may limit the requests, and then we need to delay sending the request for some time.
    /// Wait this amount of time if we hit rate limit on infura https://infura.io/docs/ethereum/json-rpc/ratelimits
    pub request_rate_limit_delay: u64,
}

impl ClientConfig {
    pub fn from_env(chain_id: u8) -> Self {
        envy_load!("client", format!("CHAIN_{}_CLIENT_", chain_id))
    }

    /// Get first web3 url, useful in direct web3 clients, which don't need any multiplexers
    pub fn web3_url(&self) -> String {
        self.web3_url
            .first()
            .cloned()
            .expect("Should be at least one")
    }

    pub fn limit_delay(&self) -> Duration{
        Duration::from_secs(self.request_rate_limit_delay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configs::test_utils::set_env;

    fn expected_config() -> ClientConfig {
        ClientConfig {
            chain_id: 9,
            web3_url: vec![
                "http://127.0.0.1:8545".into(),
                "http://127.0.0.1:8546".into(),
            ],
            request_rate_limit_delay: 30
        }
    }

    #[test]
    fn from_env() {
        let config = r#"
        CHAIN_1_CLIENT_CHAIN_ID="9"
        CHAIN_1_CLIENT_WEB3_URL="http://127.0.0.1:8545,http://127.0.0.1:8546"
        CHAIN_1_CLIENT_REQUEST_RATE_LIMIT_DELAY=30
        "#;
        set_env(config);

        let actual = ClientConfig::from_env(1);
        assert_eq!(actual, expected_config());
        assert_eq!(actual.web3_url(), "http://127.0.0.1:8545");
    }
}
