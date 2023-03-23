use serde::Deserialize;
use crate::envy_load;

/// Configuration for the zklink runtime.
#[derive(Default, Debug, Deserialize, Clone, PartialEq)]
pub struct RuntimeConfig {
    /// zklink runtime home path
    pub zklink_home: String,
    /// Path to the directory with the cryptographical keys. Relative to `$ZKLINK_HOME`.
    pub key_dir: String,
}

impl RuntimeConfig {
    pub fn from_env() -> Self {
        let mut config: RuntimeConfig = envy_load!("runtime", "RUNTIME_CONFIG_");
        let mut key_dir = config.zklink_home.clone();
        key_dir.push_str("/");
        key_dir.push_str(&config.key_dir);
        config.key_dir = key_dir;
        config
    }
}
