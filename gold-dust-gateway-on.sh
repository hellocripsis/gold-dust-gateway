#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"
echo "[gold-dust] starting dispatcher on 127.0.0.1:7777..."
cargo run --bin dispatcher
