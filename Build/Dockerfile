# Build stage
FROM rust:alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static

WORKDIR /usr/src/app
COPY . .

# Build for release
RUN cargo build --release

# Final stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

WORKDIR /app
COPY --from=builder /usr/src/app/target/release/web-search-mcp-rust .

# Expose the port
EXPOSE 4126

# Run the server
ENTRYPOINT ["./web-search-mcp-rust"]
CMD ["--address", "0.0.0.0", "--port", "4126"]
