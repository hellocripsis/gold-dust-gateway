use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Per-backend toggle config.
#[derive(Debug, Clone, Deserialize)]
pub struct BackendConfig {
    /// Enable Oxen backends.
    pub oxen_enabled: bool,
    /// Enable Tor backends.
    pub tor_enabled: bool,
}

/// Top-level Gold Dust config.
///
/// For v0.2 this is very simple: just switches for Oxen/Tor.
#[derive(Debug, Clone, Deserialize)]
pub struct GoldDustConfig {
    pub backends: BackendConfig,
}

impl GoldDustConfig {
    /// Load Gold Dust config from a TOML file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let text = fs::read_to_string(path)?;
        let cfg: GoldDustConfig = toml::from_str(&text)?;
        Ok(cfg)
    }
}
