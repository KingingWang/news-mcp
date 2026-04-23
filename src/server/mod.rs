//! MCP server module
//!
//! Provides the MCP server implementation for news-mcp.

mod handler;
mod transport;

pub use handler::*;
pub use transport::*;

use crate::cache::{create_shared_article_cache, ArticleCache, NewsCache};
use crate::config::AppConfig;
use crate::tools::ToolRegistry;
use std::sync::Arc;

/// News MCP Server
#[derive(Clone)]
#[allow(dead_code)]
pub struct NewsMcpServer {
    /// Server configuration
    config: AppConfig,
    /// News cache
    cache: Arc<NewsCache>,
    /// Article content cache
    article_cache: Arc<ArticleCache>,
    /// Tool registry
    tool_registry: Arc<ToolRegistry>,
}

impl NewsMcpServer {
    /// Create a new server instance
    pub fn new(config: AppConfig, cache: Arc<NewsCache>) -> Self {
        let article_cache = create_shared_article_cache(100);
        let tool_registry = Arc::new(ToolRegistry::new());

        Self {
            config,
            cache,
            article_cache,
            tool_registry,
        }
    }

    /// Create server with default tools registered
    pub fn with_default_tools(config: AppConfig, cache: Arc<NewsCache>) -> Self {
        let article_cache = create_shared_article_cache(100);

        let tool_registry = Arc::new(crate::tools::create_default_registry(
            cache.clone(),
            article_cache.clone(),
            config.article_fetch.clone(),
            config.feeds.clone(),
        ));

        Self {
            config,
            cache,
            article_cache,
            tool_registry,
        }
    }

    /// Get server info for MCP protocol
    pub fn server_info(&self) -> rust_mcp_sdk::schema::InitializeResult {
        use rust_mcp_sdk::schema::{
            Implementation, InitializeResult, ProtocolVersion, ServerCapabilities,
            ServerCapabilitiesTools,
        };

        InitializeResult {
            server_info: Implementation {
                name: self.config.server.name.clone(),
                version: self.config.server.version.clone(),
                title: Some("News MCP Server".to_string()),
                description: Some("MCP server for fetching news from RSS feeds".to_string()),
                icons: vec![],
                website_url: None,
            },
            capabilities: ServerCapabilities {
                tools: Some(ServerCapabilitiesTools { list_changed: None }),
                resources: None,
                prompts: None,
                experimental: None,
                completions: None,
                logging: None,
                tasks: None,
            },
            protocol_version: ProtocolVersion::V2025_11_25.into(),
            instructions: Some("Use this server to fetch news from RSS feeds. Supports multiple categories and formats.".to_string()),
            meta: None,
        }
    }

    /// Get the cache reference
    pub fn cache(&self) -> &Arc<NewsCache> {
        &self.cache
    }

    /// Get the tool registry reference
    pub fn tool_registry(&self) -> &Arc<ToolRegistry> {
        &self.tool_registry
    }

    /// Get the configuration
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// Initialize logging
    pub fn init_logging(&self) {
        crate::utils::init_logging(
            &self.config.logging.level,
            self.config.logging.enable_console,
        );
    }
}
