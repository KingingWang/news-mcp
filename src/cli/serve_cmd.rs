//! Serve command implementation
//!
//! Handles the serve subcommand for starting the MCP server.

use crate::cache::create_shared_cache;
use crate::cli::ServeCommand;
use crate::config::{AppConfig, PollerConfig, ServerConfig};
use crate::error::Result;
use crate::server::{run_server, start_poller};
use tracing::{info, warn};

/// Run the serve command
pub async fn serve_command(
    cmd: &ServeCommand,
    config_path: Option<std::path::PathBuf>,
) -> Result<()> {
    info!("Starting news-mcp server");

    // Load or create configuration
    let mut config = if let Some(path) = config_path {
        AppConfig::from_path(path)?
    } else {
        create_config_from_cmd(cmd)
    };

    // Apply environment variable overrides
    config.apply_env_overrides();

    // Create cache
    let cache = create_shared_cache(config.cache.max_articles_per_category);

    // Start poller if enabled
    if config.poller.enabled {
        let poller = start_poller(&config, cache.clone());
        info!(
            "Background poller started with {} second interval",
            config.poller.interval_secs
        );

        // Wait for initial poll to complete before accepting requests
        // This ensures the cache is populated before clients can query
        info!("Waiting for initial news poll to complete...");
        let timeout_secs = 60; // Allow up to 60 seconds for initial poll
        if poller.wait_for_initial_poll(timeout_secs).await {
            info!("Initial poll completed, cache is now populated");
        } else {
            warn!("Initial poll did not complete within {} seconds, starting server with potentially empty cache", timeout_secs);
        }
    }

    // Run server with specified transport mode
    info!(
        "Starting server with {} transport",
        config.server.transport_mode
    );
    run_server(config, cache.clone()).await?;

    info!("Server stopped");
    Ok(())
}

/// Create configuration from serve command options
fn create_config_from_cmd(cmd: &ServeCommand) -> AppConfig {
    AppConfig {
        server: ServerConfig {
            name: "news-mcp".to_string(),
            version: "0.1.0".to_string(),
            host: cmd.host.clone(),
            port: cmd.port,
            transport_mode: cmd.mode.clone(),
        },
        poller: PollerConfig {
            interval_secs: cmd.poll_interval,
            enabled: cmd.poll,
        },
        cache: crate::config::CacheConfig {
            max_articles_per_category: cmd.max_articles,
        },
        logging: crate::config::LoggingConfig {
            level: "info".to_string(),
            enable_console: true,
        },
        feeds: crate::config::AppConfig::default().feeds,
    }
}
