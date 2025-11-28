use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::config::GoldDustConfig;

/// Which kind of backend we’re using.
#[derive(Debug, Clone)]
pub enum BackendKind {
    Oxen,
    Tor,
}

/// Identity of a backend (name + kind).
#[derive(Debug, Clone)]
pub struct BackendIdentity {
    pub name: String,
    pub kind: BackendKind,
}

/// Synthetic health metrics for a backend.
///
/// In v0.1 these are fake but shaped like real telemetry.
#[derive(Debug, Clone)]
pub struct BackendHealth {
    pub name: String,
    pub kind: BackendKind,
    pub latency_ms: f64,
    pub failure_rate: f64,
    pub enabled: bool,
}

/// Result of a routing decision.
#[derive(Debug, Clone)]
pub enum BackendChoice {
    /// A concrete backend plus the health snapshot we used.
    Backend {
        backend: BackendIdentity,
        health: BackendHealth,
    },
    /// No suitable backend was found.
    NoBackend(String),
}

/// Gold Dust VPN routing engine.
///
/// v0.1:
/// - Builds a small in-memory list of Oxen + Tor backends.
/// - Generates fake latency/failure numbers with a RNG.
/// - Applies “Oxen-first, Tor-fallback” policy.
pub struct Router {
    pub backends: Vec<BackendIdentity>,
    rng: StdRng,
}

impl Router {
    /// Build a router from config.
    ///
    /// This version only looks at feature flags:
    /// - `backends.oxen_enabled`
    /// - `backends.tor_enabled`
    ///
    /// Node names are hard-coded: `oxen-node-1`, `oxen-node-2`, `tor-exit-1`.
    pub fn from_config(config: &GoldDustConfig) -> Self {
        let mut backends = Vec::new();

        if config.backends.oxen_enabled {
            backends.push(BackendIdentity {
                name: "oxen-node-1".to_string(),
                kind: BackendKind::Oxen,
            });
            backends.push(BackendIdentity {
                name: "oxen-node-2".to_string(),
                kind: BackendKind::Oxen,
            });
        }

        if config.backends.tor_enabled {
            backends.push(BackendIdentity {
                name: "tor-exit-1".to_string(),
                kind: BackendKind::Tor,
            });
        }

        // Deterministic RNG so output is stable for demos/tests.
        let seed = [42u8; 32];
        let rng = StdRng::from_seed(seed);

        Router { backends, rng }
    }

    /// Return a synthetic health view for all configured backends.
    pub fn backend_health(&mut self) -> Vec<BackendHealth> {
        let mut list = Vec::new();

        for backend in &self.backends {
            // Base latency by kind.
            let base_latency = match backend.kind {
                BackendKind::Oxen => 55.0,
                BackendKind::Tor => 240.0,
            };

            // ±20 ms jitter.
            let jitter: f64 = self.rng.gen_range(-20.0..20.0);
            let latency_ms = (base_latency + jitter).max(10.0);

            // Failure rate: Oxen slightly worse than Tor here, just for flavor.
            let failure_rate = match backend.kind {
                BackendKind::Oxen => self.rng.gen_range(0.0..0.03),
                BackendKind::Tor => self.rng.gen_range(0.0..0.02),
            };

            list.push(BackendHealth {
                name: backend.name.clone(),
                kind: backend.kind.clone(),
                latency_ms,
                failure_rate,
                enabled: true,
            });
        }

        list
    }

    /// Core routing decision:
    ///
    /// 1. Look at all healthy Oxen nodes.
    /// 2. Pick the lowest-latency Oxen node if any.
    /// 3. Otherwise, pick the lowest-latency Tor exit if any.
    /// 4. Otherwise, return `NoBackend`.
    pub fn choose_backend_for(&mut self, _target: &str) -> BackendChoice {
        let health_list = self.backend_health();

        let mut oxen_best: Option<BackendHealth> = None;
        let mut tor_best: Option<BackendHealth> = None;

        for h in health_list {
            match h.kind {
                BackendKind::Oxen => {
                    if h.failure_rate < 0.05 {
                        let better = oxen_best
                            .as_ref()
                            .map(|curr| h.latency_ms < curr.latency_ms)
                            .unwrap_or(true);
                        if better {
                            oxen_best = Some(h);
                        }
                    }
                }
                BackendKind::Tor => {
                    if h.failure_rate < 0.05 {
                        let better = tor_best
                            .as_ref()
                            .map(|curr| h.latency_ms < curr.latency_ms)
                            .unwrap_or(true);
                        if better {
                            tor_best = Some(h);
                        }
                    }
                }
            }
        }

        // Oxen-first.
        if let Some(h) = oxen_best {
            let identity = BackendIdentity {
                name: h.name.clone(),
                kind: BackendKind::Oxen,
            };
            return BackendChoice::Backend {
                backend: identity,
                health: h,
            };
        }

        // Tor-fallback.
        if let Some(h) = tor_best {
            let identity = BackendIdentity {
                name: h.name.clone(),
                kind: BackendKind::Tor,
            };
            return BackendChoice::Backend {
                backend: identity,
                health: h,
            };
        }

        BackendChoice::NoBackend("No healthy Oxen or Tor backends available".to_string())
    }
}
