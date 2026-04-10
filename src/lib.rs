//! News MCP Server
//!
//! A Rust MCP server for fetching news from RSS feeds with background polling.

pub mod cache;
pub mod cli;
pub mod config;
pub mod error;
pub mod poller;
pub mod server;
pub mod service;
pub mod tools;
pub mod utils;

pub use error::{Error, Result};
