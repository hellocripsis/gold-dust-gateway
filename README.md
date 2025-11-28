# Gold Dust VPN (control plane)

Gold Dust is a **Rust control-plane prototype** for a VPN-style router:

- **Oxen-first, Tor-fallback** policy
- Health-checks simulated in Rust
- Simple CLI to show routing decisions and backend status
- Minimal TCP proxy to actually move HTTP traffic

It is **not** a full VPN tunnel. It’s the brain that decides *which* backend path to use.

---

## Features

- Config file: `gold-dust-vpn.toml` controls:
  - Whether Oxen and Tor are enabled
  - Basic latency / failure-rate modeling
- CLI commands:
  - `status` – show backend health
  - `route <host:port>` – pick the best backend for a target
  - `proxy <host:port>` – stream real TCP traffic through the chosen backend
- Oxen-first policy:
  - Prefer Oxen when enabled and “healthy”
  - Fall back to Tor if Oxen is disabled or looks degraded

---

## Quickstart

From the repo root:

```bash
cargo run -- status
