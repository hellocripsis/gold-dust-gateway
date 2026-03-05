# gold-dust-gateway

`gold-dust-gateway` is a small Rust portfolio project that runs locally as:

- a minimal HTTP CONNECT proxy (`dispatcher`)
- a simple routing-policy CLI (`gold-dust-gateway`)
- a lightweight dashboard (`dashboard`) to toggle proxy mode

It’s built to demonstrate secure-networking-adjacent systems work in Rust: bounded parsing, explicit routing behavior, and clean local control surfaces.

Everything runs locally on 127.0.0.1.

---

## What it does

1) HTTP CONNECT proxy on 127.0.0.1:7777
   - Routes via Tor (SOCKS5 on 127.0.0.1:9050) when the flag is ON
   - Routes direct TCP when the flag is OFF
   - Enforces basic safety bounds (CONNECT-only, header size limit)

2) Dashboard on 127.0.0.1:3000
   - Shows current proxy mode (Tor vs Direct)
   - Toggles the mode by writing the flag file used by the proxy

3) CLI “routing brain” (simulation)
   - Loads a tiny TOML config (enable Oxen and/or Tor backends)
   - Prints backend health snapshots
   - Picks a backend using an Oxen-first, Tor-fallback policy

---

## Binaries

### 1) gold-dust-gateway (CLI)

Run help:

    cargo run --bin gold-dust-gateway -- --help

Examples:

    # Show backend health snapshot
    cargo run --bin gold-dust-gateway -- status

    # Ask which backend would be used for a given target
    cargo run --bin gold-dust-gateway -- route example.com:443

Config file path (optional):

    cargo run --bin gold-dust-gateway -- --config gold-dust-gateway.toml status

Notes:
- Backend health values are simulated for demonstration.
- Routing policy is Oxen-first, Tor-fallback when both are enabled.

---

### 2) dispatcher (HTTP CONNECT proxy)

Runs a minimal HTTP CONNECT proxy on 127.0.0.1:7777.

Routing behavior:

- Tor mode:
  - Uses SOCKS5 at 127.0.0.1:9050
- Direct mode:
  - Uses a direct TCP connection to the CONNECT target

Run it:

    cargo run --bin dispatcher

Example usage:

    curl -x http://127.0.0.1:7777 https://check.torproject.org/

Behavior details:
- Only CONNECT is supported. Non-CONNECT requests return 405.
- CONNECT header parsing stops at CRLFCRLF.
- Request headers over 8KB are rejected.
- If the flag file is missing, the proxy defaults to Tor mode.

---

### 3) dashboard (web UI)

Runs a small Axum-based dashboard on http://127.0.0.1:3000.

What it does:
- Shows whether the proxy is in Tor or Direct mode
- Toggles the mode by writing the flag file

Run it:

    cargo run --bin dashboard

Open:

    http://127.0.0.1:3000/

---

## Config and flags

Config file:

- gold-dust-gateway.toml

Current shape:

    [backends]
    oxen_enabled = false
    tor_enabled  = true

Proxy mode flag file (project root):

- gold-dust-tor.flag

Values:
- "on"  => Tor mode
- "off" => Direct mode

The dashboard reads/writes this flag.
The dispatcher reads it to decide routing.

---

## Why this project is relevant (systems / secure networking)

- Async TCP proxying with a clear routing boundary (Tor vs direct)
- Explicit parsing bounds and fail-closed behavior (CONNECT-only, header size limit)
- Local control surface with a minimal UI and a deterministic flag-based switch
- Simple routing policy simulator (Oxen-first, Tor-fallback) driven by a TOML config
