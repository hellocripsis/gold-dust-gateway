# gold-dust-gateway

`gold-dust-gateway` is a Rust portfolio project that runs locally as an HTTP CONNECT proxy, a small routing/policy layer, and a lightweight dashboard for controlling proxy mode.

It demonstrates:

* Async TCP proxying in Rust (HTTP CONNECT)
* Tor vs direct routing using a simple on-disk flag
* Clean, testable routing decisions and backend selection logic
* A CLI that reports status and queries entropy health metrics via Krypton

Everything runs locally on `127.0.0.1`.

## Binaries

### 1) `dispatcher` (HTTP CONNECT proxy)

A minimal HTTP CONNECT proxy that listens on `127.0.0.1:7777` and routes traffic:

* Via Tor over SOCKS5 (`127.0.0.1:9050`) when `gold-dust-tor.flag` is `on`
* Direct TCP when `gold-dust-tor.flag` is `off`

Run:

```bash
cargo run --bin dispatcher
```

Example usage:

```bash
curl -x http://127.0.0.1:7777 https://check.torproject.org/
```

### 2) `dashboard` (local web UI)

A small Axum-based dashboard on `http://127.0.0.1:3000` that:

* Shows whether the proxy is in Tor or Direct mode
* Toggles Tor ON/OFF by updating the `gold-dust-tor.flag` file used by `dispatcher`

Run:

```bash
cargo run --bin dashboard
```

Visit:

* `http://127.0.0.1:3000/`

### 3) `gold-dust-gateway` (CLI)

A CLI that simulates backend routing decisions and can query Krypton entropy health.

Run:

```bash
cargo run --bin gold-dust-gateway -- --help
```

Examples:

```bash
# Show backend status (simulated Oxen / Tor backends)
cargo run --bin gold-dust-gateway -- status

# Ask which backend would be used for a given target
cargo run --bin gold-dust-gateway -- route example.com:443

# Query Krypton (OS RNG-backed) entropy health
cargo run --bin gold-dust-gateway -- health --samples 4096
```

The `health` command prints:

* sample count
* mean / variance / jitter of bit density
* `Keep` / `Throttle` / `Kill` decision from `krypton-entropy-core`

## Config and flags

Main config file:

* `gold-dust-gateway.toml` (default profile, backend enable flags, etc.)

Proxy mode flag file (project root):

* `gold-dust-tor.flag`

  * `on`  -> Tor mode
  * `off` -> Direct mode

The dashboard reads and writes this flag, and the dispatcher reads it to decide routing.

## Relationship to other crates

This project uses:

* `krypton-entropy-core` for OS RNG-backed entropy metrics and explicit `Keep` / `Throttle` / `Kill` decisions.

All entropy comes from the OS RNG via the `rand` crate. No custom RNG or proprietary entropy engine is implemented here.
