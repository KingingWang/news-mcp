//! Error handling module
//!
//! Provides a unified error type for the news-mcp server.

use thiserror::Error;

/// Main error type for the news-mcp server
#[derive(Debug, Error)]
pub enum Error {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// HTTP middleware error
    #[error("HTTP middleware error: {0}")]
    HttpMiddleware(#[from] reqwest_middleware::Error),

    /// RSS feed parsing failed
    #[error("RSS feed parsing failed: {0}")]
    RssParse(String),

    /// Cache operation failed
    #[error("Cache operation failed: {0}")]
    Cache(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// MCP protocol error
    #[error("MCP protocol error in '{context}': {message}")]
    Mcp {
        /// Context where error occurred
        context: String,
        /// Error message
        message: String,
    },

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML parsing error (deserialization)
    #[error("TOML parsing error: {0}")]
    TomlDe(#[from] toml::de::Error),

    /// TOML serialization error
    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    /// Invalid category
    #[error("Invalid category: {0}")]
    InvalidCategory(String),

    /// Tool execution error
    #[error("Tool execution error: {0}")]
    Tool(String),
}

impl Error {
    /// Create a configuration error with a field name and message
    pub fn config(field: &str, message: impl Into<String>) -> Self {
        Error::Config(format!("{}: {}", field, message.into()))
    }

    /// Create a cache error with a message
    pub fn cache(message: impl Into<String>) -> Self {
        Error::Cache(message.into())
    }

    /// Create an MCP error with context and message
    pub fn mcp(context: impl Into<String>, message: impl Into<String>) -> Self {
        Error::Mcp {
            context: context.into(),
            message: message.into(),
        }
    }

    /// Create a tool execution error with a message
    pub fn tool(message: impl Into<String>) -> Self {
        Error::Tool(message.into())
    }

    /// Create an RSS parsing error with a message
    pub fn rss(message: impl Into<String>) -> Self {
        Error::RssParse(message.into())
    }

    /// Create an invalid category error
    pub fn invalid_category(category: impl Into<String>) -> Self {
        Error::InvalidCategory(category.into())
    }
}

/// Result type alias for news-mcp operations
pub type Result<T> = std::result::Result<T, Error>;
