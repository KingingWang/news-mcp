//! CLI module
//!
//! Provides command-line interface for the news-mcp server.

mod commands;
mod config_cmd;
mod serve_cmd;
mod test_cmd;

pub use commands::*;
pub use config_cmd::*;
pub use serve_cmd::*;
pub use test_cmd::*;
