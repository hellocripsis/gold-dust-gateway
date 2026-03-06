pub mod config;
pub mod router;

/// Shared path for the Tor/direct mode flag file.
/// Both the dispatcher and dashboard must read/write this same path.
pub const FLAG_PATH: &str = "gold-dust-tor.flag";
