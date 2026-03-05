use crate::config::GoldDustConfig;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::error::Error;
use std::fmt;

/// Which family a backend belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    Oxen,
    Tor,
}

/// Health snapshot for a single backend.
#[derive(Debug, Clone)]
pub struct BackendHealth {
    pub name: String,
    pub kind: BackendKind,
    pub latency_ms: f64,
    pub failure_rate: f64,
    pub enabled: bool,
}

/// The router’s choice for a given target.
#[derive(Debug, Clone)]
pub struct BackendChoice {
    pub name: String,
    pub kind: BackendKind,
    pub latency_ms: f64,
    pub failure_rate: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouterError {
    NoBackendsConfigured,
}

impl fmt::Display for RouterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoBackendsConfigured => write!(f, "no backends are configured or enabled"),
        }
    }
}

impl Error for RouterError {}

/// Simple in-memory router: Oxen-first, Tor-fallback.
#[derive(Debug)]
pub struct Router {
    backends: Vec<BackendHealth>,
}

impl Router {
    /// Build a router from config flags (oxen_enabled / tor_enabled).
    pub fn from_config(config: &GoldDustConfig) -> Self {
        let mut backends = Vec::new();

        if config.backends.oxen_enabled {
            backends.push(BackendHealth {
                name: "oxen-node-1".to_string(),
                kind: BackendKind::Oxen,
                latency_ms: 60.0,
                failure_rate: 0.02,
                enabled: true,
            });
            backends.push(BackendHealth {
                name: "oxen-node-2".to_string(),
                kind: BackendKind::Oxen,
                latency_ms: 70.0,
                failure_rate: 0.03,
                enabled: true,
            });
        }

        if config.backends.tor_enabled {
            backends.push(BackendHealth {
                name: "tor-exit-1".to_string(),
                kind: BackendKind::Tor,
                latency_ms: 250.0,
                failure_rate: 0.01,
                enabled: true,
            });
        }

        Self { backends }
    }

    /// Return a copy of current backend health for dashboards / CLI.
    pub fn backend_health(&self) -> Vec<BackendHealth> {
        self.backends.clone()
    }

    /// Pick a backend for this target (Oxen-first, Tor-fallback).
    pub fn choose_backend_for(&mut self, _target: &str) -> Result<BackendChoice, RouterError> {
        let mut rng = thread_rng();

        // 1) Prefer enabled Oxen
        if let Some(chosen) = self
            .backends
            .iter()
            .filter(|b| b.enabled && matches!(b.kind, BackendKind::Oxen))
            .collect::<Vec<_>>()
            .choose(&mut rng)
        {
            return Ok(BackendChoice {
                name: chosen.name.clone(),
                kind: chosen.kind,
                latency_ms: chosen.latency_ms,
                failure_rate: chosen.failure_rate,
            });
        }

        // 2) Fall back to enabled Tor
        if let Some(chosen) = self
            .backends
            .iter()
            .filter(|b| b.enabled && matches!(b.kind, BackendKind::Tor))
            .collect::<Vec<_>>()
            .choose(&mut rng)
        {
            return Ok(BackendChoice {
                name: chosen.name.clone(),
                kind: chosen.kind,
                latency_ms: chosen.latency_ms,
                failure_rate: chosen.failure_rate,
            });
        }

        Err(RouterError::NoBackendsConfigured)
    }
}

#[cfg(test)]
mod tests {
    use super::{BackendKind, Router, RouterError};
    use crate::config::{BackendConfig, GoldDustConfig};

    fn build_router(oxen_enabled: bool, tor_enabled: bool) -> Router {
        let cfg = GoldDustConfig {
            backends: BackendConfig {
                oxen_enabled,
                tor_enabled,
            },
        };
        Router::from_config(&cfg)
    }

    #[test]
    fn returns_error_when_no_backends_enabled() {
        let mut router = build_router(false, false);
        let result = router.choose_backend_for("example.com:443");
        assert!(matches!(result, Err(RouterError::NoBackendsConfigured)));
    }

    #[test]
    fn prefers_oxen_when_available() {
        let mut router = build_router(true, true);
        let choice = router
            .choose_backend_for("example.com:443")
            .expect("oxen backends should be selected");
        assert_eq!(choice.kind, BackendKind::Oxen);
    }

    #[test]
    fn falls_back_to_tor_when_oxen_disabled() {
        let mut router = build_router(false, true);
        let choice = router
            .choose_backend_for("example.com:443")
            .expect("tor backend should be selected");
        assert_eq!(choice.kind, BackendKind::Tor);
    }
}
