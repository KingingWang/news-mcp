//! CLI commands
//!
//! Defines the CLI commands and options.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// News MCP Server - A Rust MCP server for fetching news from RSS feeds
#[derive(Debug, Parser)]
#[command(name = "news-mcp")]
#[command(version = "0.1.0")]
#[command(about = "Rust MCP server for news fetching with background polling")]
pub struct Cli {
    /// Configuration file path
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, global = true, default_value = "info")]
    pub log_level: String,

    /// Subcommand
    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Start the MCP server
    Serve(ServeCommand),

    /// Test the server functionality
    Test(TestCommand),

    /// Generate default configuration file
    Config(ConfigCommand),
}

/// Serve command options
#[derive(Debug, clap::Args)]
pub struct ServeCommand {
    /// Transport mode (stdio, http, sse, hybrid)
    #[arg(short, long, default_value = "stdio")]
    pub mode: String,

    /// Server host for HTTP/SSE modes
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// Server port for HTTP/SSE modes
    #[arg(long, default_value_t = 8080)]
    pub port: u16,

    /// Enable background polling
    #[arg(long, default_value_t = true)]
    pub poll: bool,

    /// Polling interval in seconds
    #[arg(long, default_value_t = 3600)]
    pub poll_interval: u64,

    /// Maximum articles per category
    #[arg(long, default_value_t = 100)]
    pub max_articles: usize,
}

/// Test command options
#[derive(Debug, clap::Args)]
pub struct TestCommand {
    /// Test type (cache, poll, tools, all)
    #[arg(short, long, default_value = "all")]
    pub test_type: String,
}

/// Config command options
#[derive(Debug, clap::Args)]
pub struct ConfigCommand {
    /// Output file path for generated config
    #[arg(short, long, default_value = "config.toml")]
    pub output: PathBuf,
}
