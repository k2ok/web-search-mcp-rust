# web-search-mcp-rust

A high-performance [Model Context Protocol (MCP)](https://spec.modelcontextprotocol.io/) server written in Rust, providing LLM agents with real-time web search and information retrieval capabilities.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

---

## Origin

This project is a **Rust reimplementation** of [sydasif/web-search-mcp](https://github.com/sydasif/web-search-mcp), a Python/FastAPI-based MCP server. All credit for the original concept and tool design goes to the upstream project.

This reimplementation was written against [a specific upstream commit](https://github.com/sydasif/web-search-mcp/tree/0abb9c94a777ee4b343ee0e8c3e5bd8e572c7480) and does not track subsequent upstream changes. Background on the original design can be found in the [original project's issues](https://github.com/sydasif/web-search-mcp/issues).

The original Python implementation provides a solid foundation. This project takes those ideas and rebuilds them in Rust, adding:

- **Resource efficiency** — a single shared `reqwest::Client` and `SearchState` replace per-session allocations, eliminating memory leaks (verified with Valgrind).
- **Responsible access throttling** — server-wide request throttling built in from the ground up, to avoid overloading DuckDuckGo and other web services.
- **Streamable HTTP transport** — full compliance with the latest MCP specification using SSE (Server-Sent Events), in addition to STDIO.

The Streamable HTTP transport implementation also owes a great deal to [arapower/webfetch-mcp-server](https://github.com/arapower/webfetch-mcp-server), another Rust MCP server. That project served as an essential reference; completing this implementation without it would have been extremely difficult.

---

## ⚠️ Important: Responsible Use of Web Services

Web services such as DuckDuckGo are experiencing **serious operational harm** from the flood of AI agents making excessive automated requests. In response, these services are increasingly implementing rate limits and access restrictions that specifically target automated traffic.

**This program implements conservative access controls** to reduce the burden it places on these services:

- A random delay of **11–18 seconds** is enforced between DuckDuckGo requests.
- A minimum of **10 seconds** after the previous request completes is enforced before starting a new one.
- When a "bot detected" response is received, the server enters a global cooldown of **305 seconds**, during which all DDG requests are rejected.

**Users of this program are expected to respect these limits and not circumvent them.** Please do not reduce the built-in delays, modify the throttling parameters to be more aggressive than the defaults, or otherwise configure the program in a way that increases the load on these services.

> **Warning:** If this repository receives a takedown or removal request from DuckDuckGo or any other affected service — or if there is evidence that this program is being used in a manner that causes harm to those services — **public availability of this repository may be suspended without prior notice.**

We all depend on these shared services. Please use them responsibly.

---

## Features

### Shared with the original Python implementation

The following tools are functionally equivalent to those in [sydasif/web-search-mcp](https://github.com/sydasif/web-search-mcp). Refer to the upstream project for a detailed description of the original design.

| Tool | Description |
|------|-------------|
| `search_web` | Web and news search via DuckDuckGo Lite |
| `fetch_page` | Extracts readable text content from a given URL |
| `search_domain` | Targeted search restricted to a specific domain (e.g., documentation sites) |

### Additions in this Rust reimplementation

#### Streamable HTTP Transport

In addition to STDIO, this server implements the **Streamable HTTP** transport defined in the latest MCP specification. Server-to-client messages are delivered over SSE; client-to-server messages use HTTP POST. Both transport modes are supported in the same binary and selected at startup via command-line arguments.

This implementation was made possible by referencing [arapower/webfetch-mcp-server](https://github.com/arapower/webfetch-mcp-server), another Rust-based MCP server that tackles the same transport layer. It served as an indispensable guide; completing this work without that reference would have been extremely difficult.

#### Shared Request Throttling (DDG Anti-Bot Mitigation)

All requests to DuckDuckGo pass through a single, server-wide shared timer. Concurrent requests from multiple sessions are serialized, not parallelized — meaning the throttle applies globally, not per-session.

The throttling algorithm:

1. A random target interval *T* is chosen in the range [`--ddg-min-wait`, `--ddg-max-wait`] seconds (default: 11–18s).
2. The next request waits until at least *T* seconds have passed since the last request *started*.
3. The next request also waits until at least `--ddg-post-wait` seconds (default: 10s) have passed since the last request *completed*.
4. If more than `--ddg-max-wait` seconds have already elapsed since the last request, the wait is skipped entirely.
5. The `last_request_start` timestamp is updated before sleeping, so concurrent requests are correctly queued without race conditions.

If zero results are returned on the first attempt, the server waits 30 seconds and retries once before returning an error.

#### Penalty Tracking

When DuckDuckGo returns a "bot detected" page, the server records this globally and enters a blocked state:

- All subsequent DDG requests are immediately rejected with an informative error message.
- The blocked state expires after `DDG_BLOCK_DURATION` seconds (default: 305).
- The block duration is configurable via the `DDG_BLOCK_DURATION` environment variable.

#### `get_version` Tool

Returns the current server version string as defined in `Cargo.toml`.

---

## About This Project

This project was developed by **[k2ok](https://github.com/k2ok)**, a self-described **Rust beginner**. The implementation was assisted by [OpenCode](https://github.com/anomalyco/opencode), an AI-powered coding assistant. The specific AI model used during development is not disclosed.

### Bug reports and implementation feedback

Given the author's limited Rust experience, there are likely rough edges in the code — whether in error handling, idiomatic patterns, async usage, or overall architecture. **Bug reports and constructive feedback are very welcome.** Please open an issue if you find something incorrect, unsafe, or unnecessarily un-idiomatic.

### Feature requests

**Feature requests will not be addressed.** This project is published in its current form for those who find it useful. If you need additional functionality, please **fork the repository** and implement it yourself. Pull requests that add new features are unlikely to be merged, but forks are encouraged.

---

## Requirements

- Rust stable toolchain (edition 2021)
- Internet access

## Installation

```bash
git clone https://github.com/k2ok/web-search-mcp-rust.git
cd web-search-mcp-rust
cargo build --release
```

The compiled binary will be available at `./target/release/web-search-mcp-rust`.

## Usage

### STDIO mode (default)

```bash
./target/release/web-search-mcp-rust
```

### HTTP mode

```bash
./target/release/web-search-mcp-rust --port 4126
```

The MCP endpoint will be available at `http://<address>:<port>/mcp`.

### Integration with OpenCode

Add the following to your `opencode.json` (e.g., `~/.config/opencode/opencode.json` for a global config, or a project-local `opencode.json`).

For STDIO mode:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "web-search-rust": {
      "type": "local",
      "command": ["/path/to/web-search-mcp-rust/target/release/web-search-mcp-rust"],
      "enabled": true
    }
  }
}
```

For HTTP mode (server started separately, on the same machine):

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "web-search-rust": {
      "type": "remote",
      "url": "http://127.0.0.1:4126/mcp",
      "enabled": true
    }
  }
}
```

> `127.0.0.1` only works if the server and OpenCode are running on the same machine. If you run the server on a different host, change `--address` accordingly and review the [Security Considerations](#security-considerations) section below.

### Integration with Claude Desktop

> **Note:** The author primarily uses this server with OpenCode and has not verified the configuration below with Claude Desktop. If you try it and run into issues, bug reports are welcome.

For STDIO mode, add the following to your Claude Desktop configuration:

```json
{
  "mcpServers": {
    "web-search-rust": {
      "command": "/path/to/web-search-mcp-rust/target/release/web-search-mcp-rust"
    }
  }
}
```

> Claude Desktop's `claude_desktop_config.json` does not support adding HTTP/Streamable HTTP servers directly. To use HTTP mode with Claude Desktop, add the server via Settings > Integrations (custom connector) instead.

---

## Configuration

### Command-line arguments

| Argument | Default | Description |
|---|---|---|
| `-a`, `--address` | `127.0.0.1` | Bind address (optional) |
| `-p`, `--port` | — | Port number — enables HTTP mode |
| `-k`, `--keep-alive` | `30` | SSE keep-alive heartbeat interval (seconds) |
| `-s`, `--session-timeout` | disabled | Session inactivity timeout (seconds); set `0` to disable |
| `-u`, `--user-agent` | built-in | Custom User-Agent string for all outgoing requests |
| `-c`, `--config` | — | Path to a TOML configuration file |
| `--ddg-min-wait` | `11` | Minimum interval between DuckDuckGo requests (seconds) |
| `--ddg-max-wait` | `18` | Maximum interval between DuckDuckGo requests (seconds) |
| `--ddg-post-wait` | `10` | Minimum wait after a DDG request completes (seconds) |

### Environment variables

| Variable | Default | Description |
|---|---|---|
| `WEB_SEARCH_LOGGING_ENABLED` | `false` | Set to `true` or `1` to enable raw request/response logging to disk |
| `WEB_SEARCH_LOG_DIR` | `logs` | Directory for log files (created automatically if needed) |
| `DDG_BLOCK_DURATION` | `305` | Cooldown period in seconds after a bot-detection response |

### Configuration file (TOML)

A TOML file can be used to configure the User-Agent string:

```toml
user_agent = "my-custom-agent/1.0"
```

Load it with `--config path/to/config.toml`. Command-line `--user-agent` takes precedence over the config file.

### Logging example

```bash
WEB_SEARCH_LOGGING_ENABLED=true WEB_SEARCH_LOG_DIR=./debug_logs \
  ./target/release/web-search-mcp-rust
```

---

## Security Considerations

### HTTP mode

This server implements no authentication. When running in HTTP mode (`--port`),
**any client that can reach the server's network address can call its tools.**

- The default bind address is `127.0.0.1` (loopback only). Do not change this
  to `0.0.0.0` on an untrusted network without additional access controls.
- If you need to expose the server beyond localhost, place it behind a reverse
  proxy (e.g., nginx, Caddy) with authentication and network-level restrictions.
- The `fetch_page` tool fetches arbitrary URLs from the server's own network
  context. In environments where the server has access to internal services,
  restrict access accordingly.

**STDIO mode** (the default) runs as a subprocess of the MCP client and is
not network-accessible, so these concerns do not apply.

### Debug logging

When `WEB_SEARCH_LOGGING_ENABLED=true`, raw HTTP responses are written to disk.
These files may contain sensitive data. Ensure the log directory is appropriately
protected and cleared when no longer needed.

---

## Testing

This project includes a TypeScript test suite using [Vitest](https://vitest.dev/). See [`docs/testing.md`](docs/testing.md) for details.

---

## License

[MIT License](./LICENSE) — Copyright (c) 2026 k2ok
