# Usage Guide

> This is a quick-reference guide. For complete documentation see the [README](./README.md).

## Build

```bash
cargo build --release
```

The binary will be at `./target/release/web-search-mcp-rust`.

## Running the Server

### STDIO mode (default)

Used by most MCP clients (e.g. Claude Desktop):

```bash
./target/release/web-search-mcp-rust
```

### HTTP mode

```bash
./target/release/web-search-mcp-rust --port 4126
```

The MCP endpoint will be available at `http://127.0.0.1:4126/mcp`.

> **Security note:** The server has no built-in authentication. In HTTP mode,
> restrict access at the network level (firewall, reverse proxy, etc.) unless
> the server is bound to loopback only. See [README § Security](./README.md#security-considerations).

## Integration with Claude Desktop

**STDIO (recommended):**

```json
{
  "mcpServers": {
    "web-search-rust": {
      "command": "/path/to/web-search-mcp-rust/target/release/web-search-mcp-rust"
    }
  }
}
```

**HTTP:**

```json
{
  "mcpServers": {
    "web-search-rust": {
      "type": "streamable-http",
      "url": "http://127.0.0.1:4126/mcp"
    }
  }
}
```

## Common Command-line Options

| Option | Default | Description |
|---|---|---|
| `-p`, `--port` | — | Port number — enables HTTP mode |
| `-a`, `--address` | `127.0.0.1` | Bind address |
| `-u`, `--user-agent` | built-in | Custom User-Agent for outgoing requests |
| `-c`, `--config` | — | Path to a TOML configuration file |
| `--ddg-min-wait` | `11` | Minimum interval between DDG requests (seconds) |
| `--ddg-max-wait` | `18` | Maximum interval between DDG requests (seconds) |
| `--ddg-post-wait` | `10` | Minimum wait after a DDG request completes (seconds) |

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `WEB_SEARCH_LOGGING_ENABLED` | `false` | Set to `true` or `1` to enable debug logging |
| `WEB_SEARCH_LOG_DIR` | `logs` | Directory for log files |
| `DDG_BLOCK_DURATION` | `305` | Cooldown period (seconds) after bot detection |

## Quick Test (STDIO)

Send an `initialize` request to confirm the server is responding:

```bash
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}},"id":1}' \
  | ./target/release/web-search-mcp-rust
```
