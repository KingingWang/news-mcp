# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Comprehensive documentation suite (ARCHITECTURE.md, CONTRIBUTING.md)
- GitHub Actions CI/CD workflow
- Examples directory with usage guides
- MIT License file

## [0.1.0] - 2026-04-11

### Added

- Initial release of News MCP Server
- RSS feed fetching support with configurable sources
  - Technology: TechCrunch, Ars Technica, The Verge
  - Science: ScienceDaily
- Hacker News API integration via newswrap crate
- China News (chinanews.com.cn) support with 21 categories:
  - Instant
  - Headlines
  - Politics
  - East-West Dialogue
  - Society
  - Finance
  - Life
  - Health
  - Greater Bay Area
  - Overseas Chinese
  - Video
  - Photo
  - Creative
  - Live
  - Education
  - Law
  - United Front
  - Ethnic Unity
  - Belt and Road
  - Theory
  - ASEAN Trade
- Background polling with configurable intervals
- Multiple transport modes:
  - stdio (for Claude Desktop integration)
  - HTTP (Streamable HTTP protocol)
  - SSE (Server-Sent Events)
  - hybrid (HTTP + SSE combined)
- MCP tools:
  - `get_news`: Fetch articles by category with format options (markdown, json, text)
  - `search_news`: Search cached articles by keyword
  - `get_categories`: List available categories with article counts
  - `health_check`: Server status and cache statistics
  - `refresh_news`: Manual cache refresh trigger
- In-memory cache with thread-safe `RwLock` implementation
- Configurable maximum articles per category
- TOML configuration file support
- Environment variable overrides:
  - `NEWS_MCP_PORT`: Server port
  - `NEWS_MCP_HOST`: Server host
  - `NEWS_MCP_TRANSPORT`: Transport mode
  - `NEWS_MCP_INTERVAL`: Polling interval
  - `NEWS_MCP_LOG_LEVEL`: Log level
- Pluggable `NewsSource` trait for extensibility
- HTTP client with retry middleware (exponential backoff)
- Concurrent RSS feed fetching using futures
- Structured logging with tracing
- CLI with clap for command-line parsing
- Docker deployment support with provided Dockerfile

### Technical Details

- Built with Rust 1.75+
- Uses tokio async runtime
- MCP protocol via rust-mcp-sdk
- RSS/Atom parsing via feed-rs
- Hacker News API via newswrap

---

## Version History Summary

| Version | Date | Highlights |
|---------|------|------------|
| 0.1.0 | 2026-04-11 | Initial release with RSS, HN, China News support |

---

For more details on each release, see the [GitHub Releases](https://github.com/KingingWang/news-mcp/releases) page.