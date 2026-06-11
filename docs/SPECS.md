# Technical Specifications

This document outlines the technical specifications and current capabilities of the `web-search-mcp-rust` server.

## Architecture
 
The app provides both "STDIO" and "Streamable HTTP".
 
### Deployment & Configuration
  
The server can be started with the following arguments:
  
- `--address, -a`: Address to listen on (optional for HTTP server, defaults to 127.0.0.1).
- `--port, -p`: Port to listen on (required for HTTP server).
- `--keep-alive, -k`: Keep-alive heartbeat interval in seconds for SSE transport (default: 30).
- `--session-timeout, -s`: Session inactivity timeout in seconds. Set to `0` to disable. (default: disabled).
- `--user-agent, -u`: Custom User-Agent string for outgoing requests.
- `--config, -c`: Path to a configuration file (TOML).
- `--ddg-min-wait`: Minimum interval between DuckDuckGo requests in seconds (default: 11).
- `--ddg-max-wait`: Maximum interval between DuckDuckGo requests in seconds (default: 18).
- `--ddg-post-wait`: Minimum wait after the previous DuckDuckGo request finishes in seconds (default: 10).
  
#### Environment Variables
  
- `WEB_SEARCH_LOGGING_ENABLED`: Enable/disable raw response and parsed result logging (default: `false`).
- `WEB_SEARCH_LOG_DIR`: Directory where log files are stored (default: `logs`).
- `DDG_BLOCK_DURATION`: Cooldown period in seconds after a bot-detection response (default: `305`).
  
#### Configuration File (TOML)
The `User-Agent` can be configured in a TOML file:
```toml
user_agent = "custom-user-agent"
```

Example (HTTP):
`cargo run -- --port 4126 --keep-alive 30`

 
Example (STDIO):
`cargo run`
 
### Core Components
 
- **Transport Handler**: Manages the request-response cycle over STDIO (default) and HTTP (Streamable HTTP / SSE).
 
- **Tool Orchestrator**: Routes tool calls to their respective implementation modules.
- **External API Integrations**:
    - **DuckDuckGo**: For web and news search.
 
## Tool Capabilities
 
### 1. `search_web`
Performs a web search using DuckDuckGo Lite. Implements a shared request timer to avoid bot detection. If zero results are returned, the server will wait 30 seconds and retry the request once before returning an error.
- **Inputs**: 
    - `query` (required): The search string.
    - `search_type`: "text" or "news" (default: "text").
    - `max_results`: Number of results to return (default: 5).
- **Output**: A list of search results including title, URL, and snippet.
 
### 2. `fetch_page`
Extracts content from a given URL.
- **Inputs**:
    - `url` (required): The target URL.
    - `output_format`: Requested format (default: "txt").
    - `include_metadata`: Whether to include page metadata (default: false).
    - `max_length`: Maximum characters to return (default: 15000).
    - `timeout`: Request timeout in seconds (default: 30).
- **Output**: Extracted text content from the page.
 
### 3. `search_domain`
Performs a targeted search within a specific domain.
- **Inputs**:
    - `query` (required): The search string.
    - `domain`: The domain to restrict search to (default: "docs.python.org").
- **Output**: Search results filtered by the specified domain.
 

 
## DuckDuckGo (DDG) Search Implementation

### Shared Request Timer
To avoid bot detection and rate limiting by DuckDuckGo, the server implements a shared timer across all clients and sessions.

#### Throttling Logic
1. **Target Interval**: For each request, a random target interval $T$ is chosen between `min_wait` and `max_wait` seconds (default: 11-18s).
2. **Wait Constraints**:
   - The request must start at least $T$ seconds after the previous request started.
   - The request must start at least `post_wait` seconds after the previous request finished (default: 10s).
3. **Bypass**: If the time since the last request started is already greater than `max_wait` seconds, the $T$ constraint is ignored.
4. **Wait Calculation**:
   $\text{WaitTime} = \max(0, T - (t_{\text{now}} - t_{\text{start\_last}}), \text{post\_wait} - (t_{\text{now}} - t_{\text{end\_last}}))$
5. **Serialization**: The `last_request_start` is updated immediately after calculating the wait time (including the wait duration) to ensure subsequent concurrent requests are correctly queued.

### Penalty Handling
- When a "bot detected" response is received, the server enters a blocked state.
- The `blocked_until` timestamp is set using the `DDG_BLOCK_DURATION` environment variable (default: 305s).
- All subsequent requests are rejected immediately until the cooldown period expires.

## Protocol Compliance

 
The server implements the following MCP methods:
- `initialize`: Returns server information and capabilities.
- `tools/list`: Lists all available tools and their input schemas.
- `tools/call`: Executes a specific tool and returns the result.
 
## Future Improvements
- Support for more output formats in `fetch_page`.
