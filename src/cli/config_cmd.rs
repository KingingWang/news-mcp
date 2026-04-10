//! Config command implementation
//!
//! Handles the config subcommand for generating configuration files.

use crate::cli::ConfigCommand;
use crate::config::AppConfig;
use crate::error::Result;
use std::fs;
use tracing::info;

/// Run the config command
pub fn config_command(cmd: &ConfigCommand) -> Result<()> {
    info!(
        "Generating default configuration to {}",
        cmd.output.display()
    );

    let config = AppConfig::default();
    let content = toml::to_string_pretty(&config)?;

    fs::write(&cmd.output, content)?;

    println!("Configuration file generated at: {}", cmd.output.display());
    println!("Default configuration:");
    println!("- Server: news-mcp v0.1.0");
    println!("- Transport: stdio");
    println!("- Polling interval: 3600 seconds (1 hour)");
    println!("- Max articles per category: 100");
    println!("- Log level: info");

    Ok(())
}
