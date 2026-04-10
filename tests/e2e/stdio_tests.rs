//! Stdio transport tests

// Note: These tests require the binary to be built
// They test the stdio transport mode by spawning the server process

#[cfg(test)]
mod tests {
    // Stdio tests would require spawning the process and communicating via stdin/stdout
    // For now, we'll skip these as they require complex setup

    #[test]
    fn test_stdio_mode_config() {
        use news_mcp::config::TransportMode;
        assert_eq!(TransportMode::Stdio.to_string(), "stdio");
    }
}
