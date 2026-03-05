gold-dust-gateway

gold-dust-gateway is a Rust portfolio project that runs locally as an HTTP CONNECT proxy, a small routing/policy layer, and a lightweight dashboard for controlling proxy mode.

It demonstrates:

Async TCP proxying in Rust (HTTP CONNECT)

Tor vs direct routing using a simple on-disk flag

Backend selection and routing decisions

A CLI that reports status and queries entropy health metrics via Krypton

Everything runs locally on 127.0.0.1.

Binaries
1. gold-dust-gateway (CLI)

Routing brain + status.

From this crate:

cargo run --bin gold-dust-gateway -- --help

Examples:

# Show backend status (simulated Oxen / Tor backends)
cargo run --bin gold-dust-gateway -- status

# Ask which backend would be used for a given target
cargo run --bin gold-dust-gateway -- route example.com:443

# Ask Krypton (OSRNG-based) for entropy health
cargo run --bin gold-dust-gateway -- health --samples 4096

The health command prints:

number of samples

mean / variance / jitter of bit density

Keep / Throttle / Kill decision from krypton-entropy-core

2. dispatcher (HTTP CONNECT proxy)

A minimal HTTP CONNECT proxy that listens on 127.0.0.1:7777 and routes:

via Tor (SOCKS5 on 127.0.0.1:9050) when gold-dust-tor.flag is on

direct TCP when gold-dust-tor.flag is off

Run it:

cargo run --bin dispatcher

Then point a tool at it, for example:

# Example: curl over the proxy
curl -x http://127.0.0.1:7777 https://check.torproject.org/
3. dashboard (web UI)

A small Axum-based dashboard on http://127.0.0.1:3000 that:

Shows whether the proxy is in Tor or Direct mode

Lets you click a button to toggle Tor ON/OFF (writes the flag file)

Run it:

cargo run --bin dashboard

Then visit:

http://127.0.0.1:3000/ in a browser

Config and flags

The main config file:

gold-dust-gateway.toml – default profile (backend enable flags, etc.)

The proxy mode is controlled by a simple flag file in the project root:

gold-dust-tor.flag

contents on → Tor mode

contents off → Direct mode

The dashboard reads and writes this flag, so toggling in the UI immediately affects the dispatcher.

Relationship to other crates

This binary uses:

krypton-entropy-core for OSRNG-based entropy metrics and Keep/Throttle/Kill decisions.

The health CLI command calls into Krypton. All entropy comes from the OS RNG via the rand crate. No custom RNG or proprietary entropy core is implemented here.
