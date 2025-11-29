use std::error::Error;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use gold_dust_gateway::config::GoldDustConfig;
use gold_dust_gateway::router::{BackendChoice, BackendHealth, BackendKind, Router};

/// Gold Dust Gateway: Oxen-first, Tor-fallback routing brain.
///
/// v0.2: shared core + dispatcher + HTTP CONNECT proxy.
#[derive(Parser, Debug)]
#[command(name = "gold-dust-gateway", version)]
struct Cli {
    /// Optional path to a config TOML. Defaults to gold-dust-gateway.toml
    #[arg(long, short)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show backend health snapshot.
    Status,
    /// Ask the gateway which backend it would use for this target.
    Route {
        /// Host:port you want to reach (e.g. example.com:80)
        target: String,
    },
}

fn load_config(path: Option<PathBuf>) -> Result<GoldDustConfig, Box<dyn Error>> {
    let cfg_path = path.unwrap_or_else(|| PathBuf::from("gold-dust-gateway.toml"));
    GoldDustConfig::load(cfg_path)
}

fn backend_label(kind: BackendKind) -> &'static str {
    match kind {
        BackendKind::Oxen => "Oxen-first, Tor-fallback policy",
        BackendKind::Tor => "Tor fallback",
    }
}

fn print_status(router: &mut Router) {
    let health_list = router.backend_health();

    println!("=== Gold Dust Gateway backend status ===");
    for h in health_list {
        println!(
            "- {:<12} [{:?}]  latency={:6.1} ms  failure_rate={:.3}  enabled={}",
            h.name, h.kind, h.latency_ms, h.failure_rate, h.enabled
        );
    }
}

fn print_route_decision(target: &str, choice: &BackendChoice) {
    println!("=== Gold Dust Gateway route decision ===");
    println!("Target:   {}", target);
    println!("Backend:  {} [{:?}]", choice.name, choice.kind);
    println!("Latency:  {:.1} ms", choice.latency_ms);
    println!("Failure:  {:.3}", choice.failure_rate);
    println!(
        "Decision: use {} ({})",
        choice.name,
        backend_label(choice.kind.clone())
    );
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // Load config and build router
    let cfg = load_config(cli.config)?;
    let mut router = Router::from_config(&cfg);

    match cli.command {
        Commands::Status => {
            print_status(&mut router);
        }
        Commands::Route { target } => {
            let choice = router.choose_backend_for(&target);
            print_route_decision(&target, &choice);
        }
    }

    Ok(())
}
