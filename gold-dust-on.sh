#!/usr/bin/env bash
cd ~/dev/gold-dust-vpn || exit 1
echo "[gold-dust] starting dispatcher on 127.0.0.1:7777..."
cargo run --bin dispatcher
