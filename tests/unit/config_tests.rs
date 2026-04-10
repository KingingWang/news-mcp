//! Configuration unit tests

use news_mcp::config::{
    AppConfig, CacheConfig, LoggingConfig, PollerConfig, ServerConfig, TransportMode,
};
use std::str::FromStr;

#[test]
fn test_default_config() {
    let config = AppConfig::default();

    assert_eq!(config.server.name, "news-mcp");
    assert_eq!(config.server.version, "0.1.0");
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.server.transport_mode, "stdio");

    assert_eq!(config.poller.interval_secs, 3600);
    assert!(config.poller.enabled);

    assert_eq!(config.cache.max_articles_per_category, 100);

    assert_eq!(config.logging.level, "info");
    assert!(config.logging.enable_console);
}

#[test]
fn test_transport_mode_from_str() {
    assert_eq!(
        TransportMode::from_str("stdio").unwrap(),
        TransportMode::Stdio
    );
    assert_eq!(
        TransportMode::from_str("http").unwrap(),
        TransportMode::Http
    );
    assert_eq!(TransportMode::from_str("sse").unwrap(), TransportMode::Sse);
    assert_eq!(
        TransportMode::from_str("hybrid").unwrap(),
        TransportMode::Hybrid
    );
    assert!(TransportMode::from_str("invalid").is_err());
}

#[test]
fn test_transport_mode_display() {
    assert_eq!(TransportMode::Stdio.to_string(), "stdio");
    assert_eq!(TransportMode::Http.to_string(), "http");
    assert_eq!(TransportMode::Sse.to_string(), "sse");
    assert_eq!(TransportMode::Hybrid.to_string(), "hybrid");
}

#[test]
fn test_config_serialization() {
    let config = AppConfig::default();
    let serialized = toml::to_string(&config).unwrap();
    assert!(serialized.contains("[server]"));
    assert!(serialized.contains("name = \"news-mcp\""));

    let deserialized: AppConfig = toml::from_str(&serialized).unwrap();
    assert_eq!(deserialized.server.name, config.server.name);
}

#[test]
fn test_server_config_default() {
    let server_config = ServerConfig::default();
    assert_eq!(server_config.name, "news-mcp");
    assert_eq!(server_config.transport_mode, "stdio");
}

#[test]
fn test_poller_config_default() {
    let poller_config = PollerConfig::default();
    assert_eq!(poller_config.interval_secs, 3600);
    assert!(poller_config.enabled);
}

#[test]
fn test_cache_config_default() {
    let cache_config = CacheConfig::default();
    assert_eq!(cache_config.max_articles_per_category, 100);
}

#[test]
fn test_logging_config_default() {
    let logging_config = LoggingConfig::default();
    assert_eq!(logging_config.level, "info");
    assert!(logging_config.enable_console);
}
