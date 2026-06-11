# tests

TypeScript integration tests for the MCP server, using [Vitest](https://vitest.dev/).

## Install dependencies

```bash
npm install
```

## Run tests

Build the Rust server first (from the project root):

```bash
cargo build
```

Then run the tests:

```bash
npm test
```

The test suite starts and stops the server process automatically.

---

## Manual Testing & Debugging

### MCP Inspector
The [MCP Inspector](https://github.com/modelcontextprotocol/inspector) is the recommended tool for manually interacting with the server.

**To test in STDIO mode:**
```bash
npx @modelcontextprotocol/inspector ../target/release/web-search-mcp-rust
```

**To test in HTTP mode:**
First, start the server:
```bash
../target/release/web-search-mcp-rust --port 4126
```
Then, run the inspector:
```bash
npx @modelcontextprotocol/inspector http://127.0.0.1:4126/mcp
```

### Using launch_server.sh
The `launch_server.sh` script is a helper for testing the **Streamable HTTP Transport**. It launches the server in the background on port 4126.

```bash
./launch_server.sh
```

Once launched, you can connect to the server using the MCP Inspector (as shown above) or any other MCP-compatible HTTP client without needing to keep a separate terminal window open for the server process.
