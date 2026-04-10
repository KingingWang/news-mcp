//! Configuration module
//!
//! Provides configuration structures for the news-mcp server.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// Server configuration
    pub server: ServerConfig,
    /// Poller configuration
    pub poller: PollerConfig,
    /// Cache configuration
    pub cache: CacheConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
}

impl AppConfig {
    /// Load configuration from a TOML file
    pub fn from_file(path: &Path) -> crate::error::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load configuration from a file path string
    pub fn from_path(path: impl AsRef<Path>) -> crate::error::Result<Self> {
        Self::from_file(path.as_ref())
    }

    /// Create default configuration
    pub fn default_config() -> Self {
        Self::default()
    }
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server name
    #[serde(default = "default_server_name")]
    pub name: String,

    /// Server version
    #[serde(default = "default_server_version")]
    pub version: String,

    /// Server host
    #[serde(default = "default_host")]
    pub host: String,

    /// Server port
    #[serde(default = "default_port")]
    pub port: u16,

    /// Transport mode: stdio, http, sse, hybrid
    #[serde(default = "default_transport_mode")]
    pub transport_mode: String,
}

fn default_server_name() -> String {
    "news-mcp".to_string()
}

fn default_server_version() -> String {
    "0.1.0".to_string()
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_transport_mode() -> String {
    "stdio".to_string()
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            name: default_server_name(),
            version: default_server_version(),
            host: default_host(),
            port: default_port(),
            transport_mode: default_transport_mode(),
        }
    }
}

/// Poller configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollerConfig {
    /// Polling interval in seconds
    #[serde(default = "default_poll_interval")]
    pub interval_secs: u64,

    /// Enable background polling
    #[serde(default = "default_poll_enabled")]
    pub enabled: bool,
}

fn default_poll_interval() -> u64 {
    3600 // 1 hour
}

fn default_poll_enabled() -> bool {
    true
}

impl Default for PollerConfig {
    fn default() -> Self {
        Self {
            interval_secs: default_poll_interval(),
            enabled: default_poll_enabled(),
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum articles per category
    #[serde(default = "default_max_articles")]
    pub max_articles_per_category: usize,
}

fn default_max_articles() -> usize {
    100
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_articles_per_category: default_max_articles(),
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Enable console logging
    #[serde(default = "default_console_enabled")]
    pub enable_console: bool,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_console_enabled() -> bool {
    true
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            enable_console: default_console_enabled(),
        }
    }
}

/// Transport mode enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportMode {
    /// Standard input/output mode
    Stdio,
    /// HTTP mode (Streamable HTTP)
    Http,
    /// Server-Sent Events mode
    Sse,
    /// Hybrid mode (HTTP + SSE)
    Hybrid,
}

impl std::str::FromStr for TransportMode {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> crate::error::Result<Self> {
        match s.to_lowercase().as_str() {
            "stdio" => Ok(TransportMode::Stdio),
            "http" => Ok(TransportMode::Http),
            "sse" => Ok(TransportMode::Sse),
            "hybrid" => Ok(TransportMode::Hybrid),
            _ => Err(crate::error::Error::config(
                "transport_mode",
                format!("invalid transport mode: {}", s),
            )),
        }
    }
}

impl std::fmt::Display for TransportMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportMode::Stdio => write!(f, "stdio"),
            TransportMode::Http => write!(f, "http"),
            TransportMode::Sse => write!(f, "sse"),
            TransportMode::Hybrid => write!(f, "hybrid"),
        }
    }
}
