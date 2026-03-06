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

    /// Fallback config if gold-dust-gateway.toml is missing.
    pub fn default_for_demo() -> Self {
        Self {
            backends: BackendConfig {
                oxen_enabled: true,
                tor_enabled: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GoldDustConfig;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_file(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("gold-dust-{name}-{nanos}.toml"))
    }

    #[test]
    fn loads_valid_config() {
        let path = unique_temp_file("config-ok");
        fs::write(
            &path,
            "[backends]\noxen_enabled = true\ntor_enabled = false\n",
        )
        .expect("should write test config");

        let cfg = GoldDustConfig::load(&path).expect("valid config should load");
        assert!(cfg.backends.oxen_enabled);
        assert!(!cfg.backends.tor_enabled);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn fails_on_invalid_config_shape() {
        let path = unique_temp_file("config-bad");
        fs::write(&path, "invalid = true\n").expect("should write test config");
        let result = GoldDustConfig::load(&path);
        assert!(result.is_err());
        let _ = fs::remove_file(path);
    }
}
