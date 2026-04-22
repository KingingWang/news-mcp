//! Transport module for MCP server
//!
//! Provides different transport modes: stdio, HTTP, SSE, and hybrid.

use crate::cache::NewsCache;
use crate::config::{AppConfig, TransportMode};
use crate::error::{Error, Result};
use crate::poller::NewsPoller;
use crate::server::{NewsMcpHandler, NewsMcpServer};
use crate::service::{HnService, NewsNowService, NewsService, NewsSource};
use rust_mcp_sdk::{
    error::McpSdkError,
    event_store,
    mcp_server::{hyper_server, server_runtime, HyperServerOptions, McpServerOptions},
    McpServer, StdioTransport, ToMcpServerHandler, TransportOptions,
};
use std::sync::Arc;
use tracing::info;

/// Run the server with the specified transport mode
pub async fn run_server(config: AppConfig, cache: Arc<NewsCache>) -> Result<()> {
    let server = NewsMcpServer::with_default_tools(config.clone(), cache.clone());

    let transport_mode: TransportMode = config.server.transport_mode.parse()?;

    match transport_mode {
        TransportMode::Stdio => run_stdio_server(&config, &server).await,
        TransportMode::Http => run_http_server(&config, &server).await,
        TransportMode::Sse => run_sse_server(&config, &server).await,
        TransportMode::Hybrid => run_hybrid_server(&config, &server).await,
    }
}

/// Run stdio transport server
pub async fn run_stdio_server(_config: &AppConfig, server: &NewsMcpServer) -> Result<()> {
    info!("Starting MCP server in stdio mode");

    let server_info = server.server_info();
    let handler = NewsMcpHandler::new(Arc::new(server.clone()));

    // Create Stdio transport
    let transport = StdioTransport::new(TransportOptions::default())
        .map_err(|e| Error::mcp("transport", e.to_string()))?;

    // Create MCP server
    let mcp_server: Arc<rust_mcp_sdk::mcp_server::ServerRuntime> =
        server_runtime::create_server(McpServerOptions {
            server_details: server_info,
            transport,
            handler: handler.to_mcp_server_handler(),
            task_store: None,
            client_task_store: None,
            message_observer: None,
        });

    info!("Stdio MCP server started, waiting for connections...");
    mcp_server
        .start()
        .await
        .map_err(|e: McpSdkError| Error::mcp("server_start", e.to_string()))?;

    Ok(())
}

/// Run HTTP transport server
pub async fn run_http_server(config: &AppConfig, server: &NewsMcpServer) -> Result<()> {
    info!(
        "Starting MCP server in HTTP mode on {}:{}",
        config.server.host, config.server.port
    );

    let server_info = server.server_info();
    let handler = NewsMcpHandler::new(Arc::new(server.clone()));

    // Create Hyper server options
    let options = HyperServerOptions {
        host: config.server.host.clone(),
        port: config.server.port,
        transport_options: Arc::new(TransportOptions::default()),
        sse_support: false,
        event_store: Some(Arc::new(event_store::InMemoryEventStore::default())),
        task_store: None,
        client_task_store: None,
        allowed_hosts: Some(vec!["*".to_string()]),
        allowed_origins: Some(vec!["*".to_string()]),
        health_endpoint: Some("/health".to_string()),
        ..Default::default()
    };

    // Create HTTP server
    let mcp_server =
        hyper_server::create_server(server_info, handler.to_mcp_server_handler(), options);

    info!(
        "HTTP MCP server started, listening on {}:{}",
        config.server.host, config.server.port
    );
    mcp_server
        .start()
        .await
        .map_err(|e: McpSdkError| Error::mcp("server_start", e.to_string()))?;

    Ok(())
}

/// Run SSE transport server
pub async fn run_sse_server(config: &AppConfig, server: &NewsMcpServer) -> Result<()> {
    info!(
        "Starting MCP server in SSE mode on {}:{}",
        config.server.host, config.server.port
    );

    let server_info = server.server_info();
    let handler = NewsMcpHandler::new(Arc::new(server.clone()));

    // Create Hyper server options with SSE support
    let options = HyperServerOptions {
        host: config.server.host.clone(),
        port: config.server.port,
        transport_options: Arc::new(TransportOptions::default()),
        sse_support: true,
        event_store: Some(Arc::new(event_store::InMemoryEventStore::default())),
        task_store: None,
        client_task_store: None,
        allowed_hosts: Some(vec!["*".to_string()]),
        allowed_origins: Some(vec!["*".to_string()]),
        health_endpoint: Some("/health".to_string()),
        ..Default::default()
    };

    // Create SSE server
    let mcp_server =
        hyper_server::create_server(server_info, handler.to_mcp_server_handler(), options);

    info!(
        "SSE MCP server started, listening on {}:{}",
        config.server.host, config.server.port
    );
    mcp_server
        .start()
        .await
        .map_err(|e: McpSdkError| Error::mcp("server_start", e.to_string()))?;

    Ok(())
}

/// Run hybrid transport server (HTTP + SSE)
pub async fn run_hybrid_server(config: &AppConfig, server: &NewsMcpServer) -> Result<()> {
    info!(
        "Starting MCP server in hybrid mode on {}:{}",
        config.server.host, config.server.port
    );

    let server_info = server.server_info();
    let handler = NewsMcpHandler::new(Arc::new(server.clone()));

    // Create Hyper server options with SSE support for hybrid mode
    let options = HyperServerOptions {
        host: config.server.host.clone(),
        port: config.server.port,
        transport_options: Arc::new(TransportOptions::default()),
        sse_support: true,
        event_store: Some(Arc::new(event_store::InMemoryEventStore::default())),
        task_store: None,
        client_task_store: None,
        allowed_hosts: Some(vec!["*".to_string()]),
        allowed_origins: Some(vec!["*".to_string()]),
        health_endpoint: Some("/health".to_string()),
        ..Default::default()
    };

    // Create hybrid server
    let mcp_server =
        hyper_server::create_server(server_info, handler.to_mcp_server_handler(), options);

    info!(
        "Hybrid MCP server started, listening on {}:{} (HTTP + SSE)",
        config.server.host, config.server.port
    );
    mcp_server
        .start()
        .await
        .map_err(|e: McpSdkError| Error::mcp("server_start", e.to_string()))?;

    Ok(())
}

/// Start the poller in background
pub fn start_poller(config: &AppConfig, cache: Arc<NewsCache>) -> Arc<NewsPoller> {
    let sources: Vec<Arc<dyn NewsSource>> = vec![
        Arc::new(NewsService::with_config(Arc::new(config.clone()))),
        Arc::new(HnService::new()),
        Arc::new(NewsNowService::new()),
    ];
    let poller = Arc::new(NewsPoller::new(sources, cache, config.poller.clone()));

    // Spawn poller task
    tokio::spawn({
        let poller_clone = poller.clone();
        async move {
            poller_clone.start().await;
        }
    });

    poller
}
