use clap::{Parser, Subcommand};

mod config;
mod router;

use crate::config::GoldDustConfig;
use crate::router::{BackendChoice, Router};

/// Gold Dust VPN: Oxen-first, Tor-fallback routing brain.
///
/// v0.2: adds a simple TCP proxy mode that actually connects to the target.
/// This is still a control plane demo, not a full VPN tunnel.
#[derive(Parser, Debug)]
#[command(name = "gold-dust-vpn", version)]
struct Cli {
    /// Path to the Gold Dust VPN config file
    #[arg(long, short, default_value = "gold-dust-vpn.toml")]
    config: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show current backend health for Oxen and Tor
    Status,
    /// Show which backend Gold Dust would use for a target
    Route {
        /// Target in host:port form, e.g. example.com:443
        target: String,
    },
    /// Connect to a target and proxy stdin/stdout through the chosen backend
    Proxy {
        /// Target in host:port form, e.g. example.com:80
        target: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Load config and build router
    let cfg = GoldDustConfig::load(&cli.config)?;
    let mut router = Router::from_config(&cfg);

    match cli.command {
        Commands::Status => {
            println!("=== Gold Dust VPN backend status ===");
            let health_list = router.backend_health();
            for backend in health_list {
                println!(
                    "- {:<12} [{:?}]  latency={:6.1} ms  failure_rate={:.3}  enabled={}",
                    backend.name,
                    backend.kind,
                    backend.latency_ms,
                    backend.failure_rate,
                    backend.enabled
                );
            }
        }

        Commands::Route { target } => {
            println!("=== Gold Dust VPN route decision ===");
            println!("Target:   {target}");

            let choice = router.choose_backend_for(&target);
            match choice {
                BackendChoice::Backend { backend, health } => {
                    println!("Backend:  {} [{:?}]", backend.name, backend.kind);
                    println!("Latency:  {:.1} ms", health.latency_ms);
                    println!("Failure:  {:.3}", health.failure_rate);
                    println!(
                        "Decision: use {} (Oxen-first, Tor-fallback policy)",
                        backend.name
                    );
                }
                BackendChoice::NoBackend(msg) => {
                    println!("Decision: {msg}");
                }
            }
        }

        Commands::Proxy { target } => {
            println!("=== Gold Dust VPN proxy ===");
            println!("Target:   {target}");

            // Use the router to pick a backend BEFORE we enter async I/O
            let choice = router.choose_backend_for(&target);

            match choice {
                BackendChoice::Backend { backend, health } => {
                    println!("Backend:  {} [{:?}]", backend.name, backend.kind);
                    println!("Latency:  {:.1} ms", health.latency_ms);
                    println!("Failure:  {:.3}", health.failure_rate);
                    println!(
                        "Note: this demo connects directly to {target}.\n\
                         In a full build, Oxen/Tor would be separate network paths."
                    );

                    // Now actually connect and pipe bytes
                    run_proxy_to_target(&target).await?;
                }
                BackendChoice::NoBackend(msg) => {
                    eprintln!("No backend available: {msg}");
                }
            }
        }
    }

    Ok(())
}

/// Connects to the given host:port and shuttles data
/// between stdin/stdout and the remote socket.
async fn run_proxy_to_target(target: &str) -> Result<(), Box<dyn std::error::Error>> {
    use tokio::io;
    use tokio::net::TcpStream;

    println!();
    println!("Connecting to {target} ...");
    let stream = TcpStream::connect(target).await?;
    println!("Connected.");
    println!("Piping stdin -> remote and remote -> stdout.");
    println!("Press Ctrl+C to stop.\n");

    let (mut rd, mut wr) = stream.into_split();

    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // stdin -> remote
    let client_to_remote = io::copy(&mut stdin, &mut wr);
    // remote -> stdout
    let remote_to_client = io::copy(&mut rd, &mut stdout);

    // Run both directions until one side closes
    let _ = tokio::try_join!(client_to_remote, remote_to_client)?;

    Ok(())
}
