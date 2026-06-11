# Build and Run MCP Server Container Image

## Build the image
```bash
podman build -t web-search-mcp-rust -f Build/Dockerfile .
```

## Run the container
```bash
podman run -p 4126:4126 web-search-mcp-rust
```
