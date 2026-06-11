# Testing the Web Search MCP Server

This project uses a TypeScript-based test suite to verify the Model Context Protocol (MCP) implementation.

## Test Framework
The tests are implemented in TypeScript using [Vitest](https://vitest.dev/). This allows for fast execution and easy interaction with the server via both stdio and HTTP.

## How to Run Tests
1. Ensure Node.js and npm are installed in your environment.
2. Build the Rust server:
   ```bash
   cargo build
   ```
3. Run the tests:
   ```bash
   cd tests
   npm test
   ```

## Test Coverage
The test suite covers:
- **Protocol Initialization**: Verifies that the server responds correctly to the `initialize` request in both stdio and TCP modes.
- **Tool Listing**: Ensures that all available tools are correctly reported by the server.
- **Tool Execution**: Tests each implemented tool (`search_web`, `fetch_page`, `search_domain`) by sending `tools/call` requests and verifying the response format.
- **Error Handling**: Verifies that the server returns an error for unknown tools or missing required parameters.

## Server Modes
The server supports two communication modes:
- **Stdio Mode**: Default mode. Used by most MCP clients (e.g., Claude Desktop).
- **HTTP Mode**: Activated by providing the `--port` argument. Useful for remote deployments.
