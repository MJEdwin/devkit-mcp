#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "==> Building devkit-mcp (release)"
cargo build --release -p cli

BIN="$ROOT_DIR/target/release/devkit-mcp"
echo "==> Built: $BIN"

"$BIN" --help >/dev/null
echo "==> OK"
echo
echo "Run locally:"
echo "  $BIN local"
echo "  $BIN remote --host 127.0.0.1 --port 8080"
