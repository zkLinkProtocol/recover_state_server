pub use crate::{
    database::DBConfig, runtime::RuntimeConfig, api::ApiConfig,
    layer1::{ChainType, MultiChainConfigs, ClientConfig, ContractConfig, Layer1Config}
};

mod runtime;
mod database;
mod layer1;
mod api;

pub struct RecoverStateConfig{
    pub runtime: RuntimeConfig,
    pub api: ApiConfig,
    pub db: DBConfig,
    pub layer1: MultiChainConfigs
}

impl RecoverStateConfig {
    pub fn from_env() -> Self{
        Self{
            runtime: RuntimeConfig::from_env(),
            api: ApiConfig::from_env(),
            db: DBConfig::from_env(),
            layer1: MultiChainConfigs::from_env(),
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
