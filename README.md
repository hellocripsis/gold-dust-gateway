# Gold Dust Gateway (control-plane MVP)

Gold Dust Gateway is a tiny Rust control-plane that decides **which backend to use** for outbound traffic:

- Prefer **Oxen** nodes when they’re healthy and enabled.
- Fall back to **Tor** when Oxen is disabled or unhealthy.
- Use simple TOML profiles so you can flip behavior without touching code.

## Usage

Check backend status:

```bash
cargo run -- status
