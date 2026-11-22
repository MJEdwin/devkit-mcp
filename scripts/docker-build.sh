#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

IMAGE="${IMAGE:-devkit-mcp:latest}"
PORT="${PORT:-8080}"

echo "==> Building Docker image: $IMAGE"
docker build --tag "$IMAGE" .

echo
echo "==> Built: $IMAGE"
echo "    HTTP MCP: http://127.0.0.1:${PORT}/mcp"
echo "    Health:   http://127.0.0.1:${PORT}/health"
echo
echo "Run:"
echo "  docker run --rm -e MCP_PORT=${PORT} -p ${PORT}:${PORT} $IMAGE"

if [[ "${1:-}" == "--run" ]]; then
    echo
    echo "==> Starting container on host port ${PORT}"
    exec docker run --rm -e "MCP_PORT=${PORT}" -p "${PORT}:${PORT}" "$IMAGE"
fi
