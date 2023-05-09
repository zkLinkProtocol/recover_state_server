use crate::evn_tools::{parse_env_if_exists, parse_env_to_vec_if_exists};
pub use crate::{
    api::ApiConfig,
    database::DBConfig,
    layer1::{ChainType, ClientConfig, ContractConfig, Layer1Config, MultiChainConfigs},
    runtime::RuntimeConfig,
};

mod api;
mod database;
pub mod evn_tools;
mod layer1;
mod runtime;

#[derive(Debug, Clone)]
pub struct RecoverStateConfig {
    pub runtime: RuntimeConfig,
    pub api: ApiConfig,
    pub db: DBConfig,
    pub layer1: MultiChainConfigs,
    pub upgrade_layer2_blocks: Vec<u32>,
    pub black_list_time: Option<u32>,
    pub enable_sync_mode: bool,
}

impl RecoverStateConfig {
    pub fn from_env() -> Self {
        Self {
            runtime: RuntimeConfig::from_env(),
            api: ApiConfig::from_env(),
            db: DBConfig::from_env(),
            layer1: MultiChainConfigs::from_env(),
            upgrade_layer2_blocks: parse_env_to_vec_if_exists("UPGRADED_LAYER2_BLOCKS")
                .unwrap_or_default(),
            black_list_time: parse_env_if_exists("CLEAN_INTERVAL"),
            enable_sync_mode: parse_env_if_exists("ENABLE_SYNC_MODE").unwrap_or_default(),
        }
    }
}

/// Convenience macro that loads the structure from the environment variable given the prefix.
///
/// # Panics
///
/// Panics if the config cannot be loaded from the environment variables.
#[macro_export]
macro_rules! envy_load {
    ($name:expr, $prefix:expr) => {
        envy::prefixed($prefix)
            .from_env()
            .unwrap_or_else(|err| panic!("Cannot load config <{}>: {}", $name, err))
    };
}
