#!/bin/sh
set -e

case "$1" in
  executable)
    cargo build --release
    ;;
  container)
    podman build -t web-search-mcp-rust -f Build/Dockerfile .
    ;;
  run)
    podman run -p 4126:4126 web-search-mcp-rust
    ;;
  test)
    cargo test
    ;;
  *)
    echo "Usage: $0 {executable|container|run|test}"
    exit 1
    ;;
esac
