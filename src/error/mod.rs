//! Error handling module
//!
//! Provides a unified error type for the news-mcp server with detailed error context
//! and error codes for better debugging and user experience.

use thiserror::Error;

/// Error codes for categorizing errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// Network-related errors
    Network,
    /// RSS feed parsing errors
    RssParse,
    /// Cache operation errors
    Cache,
    /// Configuration errors
    Config,
    /// MCP protocol errors
    Mcp,
    /// IO errors
    Io,
    /// JSON serialization errors
    Json,
    /// Invalid category
    InvalidCategory,
    /// Tool execution errors
    Tool,
    /// Rate limiting errors
    RateLimit,
    /// Timeout errors
    Timeout,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::Network => write!(f, "NETWORK_ERROR"),
            ErrorCode::RssParse => write!(f, "RSS_PARSE_ERROR"),
            ErrorCode::Cache => write!(f, "CACHE_ERROR"),
            ErrorCode::Config => write!(f, "CONFIG_ERROR"),
            ErrorCode::Mcp => write!(f, "MCP_ERROR"),
            ErrorCode::Io => write!(f, "IO_ERROR"),
            ErrorCode::Json => write!(f, "JSON_ERROR"),
            ErrorCode::InvalidCategory => write!(f, "INVALID_CATEGORY"),
            ErrorCode::Tool => write!(f, "TOOL_ERROR"),
            ErrorCode::RateLimit => write!(f, "RATE_LIMIT_ERROR"),
            ErrorCode::Timeout => write!(f, "TIMEOUT_ERROR"),
        }
    }
}

/// Main error type for the news-mcp server
#[derive(Debug, Error)]
pub enum Error {
    /// HTTP request failed
    #[error("[{code}] {source}: {message}")]
    Http {
        /// Error code
        code: ErrorCode,
        /// Error message
        message: String,
        /// Source error
        #[source]
        source: Box<reqwest::Error>,
    },

    /// HTTP middleware error
    #[error("[{code}] {message}")]
    HttpMiddleware {
        /// Error code
        code: ErrorCode,
        /// Error message
        message: String,
        /// Source error
        #[source]
        source: Box<reqwest_middleware::Error>,
    },

    /// RSS feed parsing failed
    #[error("[{code}] {message}", code = ErrorCode::RssParse)]
    RssParse {
        /// Error message
        message: String,
        /// URL that failed
        url: Option<String>,
    },

    /// Cache operation failed
    #[error("[{code}] {operation}: {message}", code = ErrorCode::Cache)]
    Cache {
        /// Operation that failed
        operation: &'static str,
        /// Error message
        message: String,
    },

    /// Configuration error
    #[error("[{code}] {field}: {message}", code = ErrorCode::Config)]
    Config {
        /// Field that has the error
        field: String,
        /// Error message
        message: String,
        /// Expected value description
        expected: Option<String>,
    },

    /// MCP protocol error
    #[error("[{code}] {context}: {message}", code = ErrorCode::Mcp)]
    Mcp {
        /// Context where error occurred
        context: String,
        /// Error message
        message: String,
        /// Tool name if applicable
        tool_name: Option<String>,
    },

    /// IO error
    #[error("[{code}] {operation}: {message}", code = ErrorCode::Io)]
    Io {
        /// Operation that failed
        operation: &'static str,
        /// Error message
        message: String,
        /// Source error
        #[source]
        source: Box<std::io::Error>,
    },

    /// JSON serialization/deserialization error
    #[error("[{code}] {context}", code = ErrorCode::Json)]
    Json {
        /// Context of the error
        context: String,
        /// Source error
        #[source]
        source: Box<serde_json::Error>,
    },

    /// TOML parsing error (deserialization)
    #[error("[{code}] {file}: {message}", code = ErrorCode::Config)]
    TomlParse {
        /// File path
        file: String,
        /// Error message
        message: String,
        /// Source error
        #[source]
        source: Box<toml::de::Error>,
    },

    /// TOML serialization error
    #[error("[{code}] {context}", code = ErrorCode::Config)]
    TomlSerialize {
        /// Context
        context: String,
        /// Source error
        #[source]
        source: Box<toml::ser::Error>,
    },

    /// Invalid category
    #[error("[{code}] '{category}' is not a valid category. Valid categories: {valid_categories}", code = ErrorCode::InvalidCategory)]
    InvalidCategory {
        /// The invalid category string
        category: String,
        /// List of valid categories
        valid_categories: String,
    },

    /// Tool execution error
    #[error("[{code}] Tool '{tool}': {message}", code = ErrorCode::Tool)]
    Tool {
        /// Tool name
        tool: String,
        /// Error message
        message: String,
        /// Error code if applicable
        tool_code: Option<String>,
    },

    /// Rate limiting error
    #[error("[{code}] Rate limit exceeded for {source_name}. Retry after {retry_after_secs} seconds", code = ErrorCode::RateLimit)]
    RateLimit {
        /// Source name
        source_name: String,
        /// Seconds until retry
        retry_after_secs: u64,
    },

    /// Timeout error
    #[error("[{code}] Operation '{operation}' timed out after {timeout_secs} seconds", code = ErrorCode::Timeout)]
    Timeout {
        /// Operation that timed out
        operation: String,
        /// Timeout duration in seconds
        timeout_secs: u64,
    },

    /// Generic error with message
    #[error("{0}")]
    Message(String),
}

impl Error {
    /// Create a configuration error with a field name and message
    pub fn config(field: impl Into<String>, message: impl Into<String>) -> Self {
        Error::Config {
            field: field.into(),
            message: message.into(),
            expected: None,
        }
    }

    /// Create a cache error with a message
    pub fn cache(message: impl Into<String>) -> Self {
        Error::Cache {
            operation: "cache",
            message: message.into(),
        }
    }

    /// Create an MCP error with context and message
    pub fn mcp(context: impl Into<String>, message: impl Into<String>) -> Self {
        Error::Mcp {
            context: context.into(),
            message: message.into(),
            tool_name: None,
        }
    }

    /// Create a tool execution error with tool name and message
    pub fn tool(tool: impl Into<String>, message: impl Into<String>) -> Self {
        Error::Tool {
            tool: tool.into(),
            message: message.into(),
            tool_code: None,
        }
    }

    /// Create a tool execution error with only a message (tool name defaults to "unknown")
    pub fn tool_msg(message: impl Into<String>) -> Self {
        Error::Tool {
            tool: "unknown".to_string(),
            message: message.into(),
            tool_code: None,
        }
    }

    /// Create an RSS parsing error with a message
    pub fn rss(message: impl Into<String>) -> Self {
        Error::RssParse {
            message: message.into(),
            url: None,
        }
    }

    /// Create an RSS parsing error with URL context
    pub fn rss_with_url(message: impl Into<String>, url: impl Into<String>) -> Self {
        Error::RssParse {
            message: message.into(),
            url: Some(url.into()),
        }
    }

    /// Create an invalid category error
    pub fn invalid_category(category: impl Into<String>) -> Self {
        let cat = category.into();
        let valid: Vec<&str> = vec![
            "technology", "business", "science", "health", "sports",
            "entertainment", "general", "world", "hackernews",
            "instant", "headlines", "politics", "eastwest", "society",
            "finance", "life", "wellness", "greaterbayarea", "chinese",
            "video", "photo", "creative", "live", "education", "law",
            "unitedfront", "ethnicunity", "beltandroad", "theory", "asean",
        ];
        Error::InvalidCategory {
            category: cat,
            valid_categories: valid.join(", "),
        }
    }

    /// Create a timeout error
    pub fn timeout(operation: impl Into<String>, timeout_secs: u64) -> Self {
        Error::Timeout {
            operation: operation.into(),
            timeout_secs,
        }
    }

    /// Create a rate limit error
    pub fn rate_limit(source_name: impl Into<String>, retry_after_secs: u64) -> Self {
        Error::RateLimit {
            source_name: source_name.into(),
            retry_after_secs,
        }
    }

    /// Get the error code for this error
    pub fn code(&self) -> ErrorCode {
        match self {
            Error::Http { .. } => ErrorCode::Network,
            Error::HttpMiddleware { .. } => ErrorCode::Network,
            Error::RssParse { .. } => ErrorCode::RssParse,
            Error::Cache { .. } => ErrorCode::Cache,
            Error::Config { .. } => ErrorCode::Config,
            Error::Mcp { .. } => ErrorCode::Mcp,
            Error::Io { .. } => ErrorCode::Io,
            Error::Json { .. } => ErrorCode::Json,
            Error::TomlParse { .. } | Error::TomlSerialize { .. } => ErrorCode::Config,
            Error::InvalidCategory { .. } => ErrorCode::InvalidCategory,
            Error::Tool { .. } => ErrorCode::Tool,
            Error::RateLimit { .. } => ErrorCode::RateLimit,
            Error::Timeout { .. } => ErrorCode::Timeout,
            Error::Message(_) => ErrorCode::Tool,
        }
    }
}

// --- From implementations for seamless ? operator usage ---

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Http {
            code: ErrorCode::Network,
            message: err.to_string(),
            source: Box::new(err),
        }
    }
}

impl From<reqwest_middleware::Error> for Error {
    fn from(err: reqwest_middleware::Error) -> Self {
        Error::HttpMiddleware {
            code: ErrorCode::Network,
            message: err.to_string(),
            source: Box::new(err),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io {
            operation: "io",
            message: err.to_string(),
            source: Box::new(err),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json {
            context: err.to_string(),
            source: Box::new(err),
        }
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::TomlParse {
            file: "unknown".to_string(),
            message: err.to_string(),
            source: Box::new(err),
        }
    }
}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Self {
        Error::TomlSerialize {
            context: err.to_string(),
            source: Box::new(err),
        }
    }
}

/// Result type alias for news-mcp operations
pub type Result<T> = std::result::Result<T, Error>;
