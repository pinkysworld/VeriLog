use serde::{Deserialize, Serialize};
use std::path::Path;

/// VeriLog configuration loaded from `config.toml` or defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub store: StoreConfig,
    pub admin: AdminConfig,
    pub privacy: PrivacyConfig,
    pub retention: RetentionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StoreConfig {
    /// Merkle tree height (default: 32, supports ~4 billion entries).
    pub tree_height: usize,
    /// Enable fsync on every append for crash safety (default: true).
    pub durable_append: bool,
    /// Maximum payload size in bytes (default: 1 MiB).
    pub max_payload_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AdminConfig {
    /// Bind address for the admin console (default: "127.0.0.1:9100").
    pub bind: String,
    /// Enable admin console authentication (default: false for prototype).
    pub require_auth: bool,
    /// Auth token for admin console (if require_auth is true).
    pub auth_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PrivacyConfig {
    /// Default epsilon for differential privacy budget (default: 1.0).
    pub default_epsilon: f64,
    /// Maximum cumulative privacy budget before refusing sensitive appends.
    pub max_budget: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RetentionConfig {
    /// Maximum number of entries before oldest are tombstoned (0 = unlimited).
    pub max_entries: u64,
    /// Maximum age in seconds before entries are tombstoned (0 = unlimited).
    pub max_age_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            store: StoreConfig::default(),
            admin: AdminConfig::default(),
            privacy: PrivacyConfig::default(),
            retention: RetentionConfig::default(),
        }
    }
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self {
            tree_height: 32,
            durable_append: true,
            max_payload_bytes: 1_048_576,
        }
    }
}

impl Default for AdminConfig {
    fn default() -> Self {
        Self {
            bind: "127.0.0.1:9100".to_string(),
            require_auth: false,
            auth_token: None,
        }
    }
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            default_epsilon: 1.0,
            max_budget: 100.0,
        }
    }
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            max_entries: 0,
            max_age_secs: 0,
        }
    }
}

impl Config {
    /// Load config from a TOML file. Returns defaults if file does not exist.
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&text)?;
        Ok(config)
    }

    /// Write the current config to a TOML file.
    pub fn save(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let text = toml::to_string_pretty(self)?;
        std::fs::write(path, text)?;
        Ok(())
    }

    /// Generate a default config file at the given path if it doesn't exist.
    pub fn init_default(path: impl AsRef<Path>) -> anyhow::Result<()> {
        let path = path.as_ref();
        if path.exists() {
            return Ok(());
        }
        Self::default().save(path)
    }
}
