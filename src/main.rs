//! News MCP Server
//!
//! Entry point for the news-mcp server.

use clap::Parser;
use news_mcp::cli::{config_command, serve_command, test_command, Cli, Commands};
use news_mcp::error::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Initialize logging
    news_mcp::utils::init_logging(&cli.log_level, true);

    // Handle commands
    match &cli.command {
        Commands::Serve(cmd) => {
            info!("Starting serve command");
            serve_command(cmd, cli.config.clone()).await?;
        }
        Commands::Test(cmd) => {
            info!("Starting test command");
            test_command(cmd)?;
        }
        Commands::Config(cmd) => {
            info!("Starting config command");
            config_command(cmd)?;
        }
    }

    info!("Command completed successfully");
    Ok(())
}
