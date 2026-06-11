use anyhow::Result;
use clap::Parser;
use reqwest::Client;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpService,
};
use rmcp::transport::stdio;
use rmcp::ServiceExt;
use web_search_mcp_rust::server::SearchServer;
use web_search_mcp_rust::models::SearchState;
use axum::Router;
use tracing_subscriber::{self, EnvFilter};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Address to listen on (if provided, starts TCP server)
    #[arg(short, long)]
    address: Option<String>,

    /// Port to listen on (if provided, starts TCP server)
    #[arg(short, long)]
    port: Option<u16>,

    /// Keep-alive timeout in seconds for SSE transport (default: 30)
    #[arg(short, long)]
    keep_alive: Option<u64>,

    /// Session inactivity timeout in seconds (disabled by default; set to 0 to disable explicitly)
    #[arg(short, long)]
    session_timeout: Option<u64>,

    /// User-Agent string to use for outgoing requests
    #[arg(short, long)]
    user_agent: Option<String>,

    /// Path to a configuration file (TOML)
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Minimum wait time for DDG requests (default: 11)
    #[arg(long, default_value_t = 11)]
    ddg_min_wait: u64,

    /// Maximum wait time for DDG requests (default: 18)
    #[arg(long, default_value_t = 18)]
    ddg_max_wait: u64,

    /// Minimum wait time after the previous DDG request finishes (default: 10)
    #[arg(long, default_value_t = 10)]
    ddg_post_wait: u64,
}

#[derive(Deserialize, Debug)]
struct Config {
    user_agent: Option<String>,
}

pub fn create_tcp_listener(addr: std::net::SocketAddr) -> std::io::Result<tokio::net::TcpListener> {
    let socket = socket2::Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::STREAM,
        Some(socket2::Protocol::TCP)
    )?;

    socket.set_reuse_address(true)?;

    let keepalive = socket2::TcpKeepalive::new()
        .with_time(std::time::Duration::from_secs(60))
        .with_interval(std::time::Duration::from_secs(10))
        .with_retries(5);
    socket.set_tcp_keepalive(&keepalive)?;

    socket.bind(&addr.into())?;
    socket.listen(128)?;
    socket.set_nonblocking(true)?;

    tokio::net::TcpListener::from_std(socket.into())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let args = Args::parse();

    let config_user_agent = if let Some(config_path) = &args.config {
        std::fs::read_to_string(config_path)
            .ok()
            .and_then(|content| toml::from_str::<Config>(&content).ok())
            .and_then(|config| config.user_agent)
    } else {
        None
    };

    let user_agent_str = args.user_agent
        .or(config_user_agent)
        .unwrap_or_else(|| format!("web-search-mcp-rust/{} (+https://github.com/k2ok/web-search-mcp-rust)", env!("CARGO_PKG_VERSION")));

    let user_agent = Arc::new(web_search_mcp_rust::user_agent::UserAgent::new(user_agent_str.clone()));

    let client = Client::builder()
        .user_agent(user_agent_str)
        .build()?;

    let state = Arc::new(Mutex::new(SearchState { 
        blocked_until: None,
        last_request_start: None,
        last_request_end: None,
        min_wait: args.ddg_min_wait,
        max_wait: args.ddg_max_wait,
        post_wait: args.ddg_post_wait,
    }));

    if let Some(port) = args.port {
        let address = args.address.as_deref().unwrap_or("127.0.0.1");
        let bind_address_str = format!("{}:{}", address, port);
        let bind_address: std::net::SocketAddr = bind_address_str.parse().expect("Invalid address");

        tracing::info!("Starting MCP HTTP server on http://{}/mcp", bind_address_str);

        let mut session_manager = LocalSessionManager::default();
        session_manager.session_config.keep_alive = args.session_timeout
            .map(|t| if t == 0 { None } else { Some(std::time::Duration::from_secs(t)) })
            .unwrap_or(None);

        let service = StreamableHttpService::new(
            move || Ok(SearchServer::new(client.clone(), state.clone(), user_agent.clone())),
            session_manager.into(),
            rmcp::transport::streamable_http_server::StreamableHttpServerConfig::default()
                .disable_allowed_hosts()
                .with_sse_keep_alive(Some(std::time::Duration::from_secs(
                    args.keep_alive.unwrap_or(30),
                ))),
        );

        let app = Router::new().nest_service("/mcp", service);

        let listener = create_tcp_listener(bind_address).expect("Failed to create TCP listener");
        axum::serve(listener, app).await?;
        Ok(())
    } else {
        tracing::info!("Starting MCP STDIO server");
        let server = SearchServer::new(client, state, user_agent);
        let service = server.serve(stdio()).await?;
        service.waiting().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[tokio::test]
    async fn test_create_tcp_listener() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
        let listener = create_tcp_listener(addr);
        assert!(listener.is_ok(), "Listener should be created successfully");
    }
}
