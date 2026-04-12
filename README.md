# News MCP Server

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/KingingWang/news-mcp/workflows/CI/badge.svg)](https://github.com/KingingWang/news-mcp/actions)

A Rust-based MCP (Model Context Protocol) server for fetching news from RSS feeds, with background polling, in-memory caching, and multiple transport modes.

## Features

- **Background Polling** - Periodically fetches news from RSS sources and caches locally
- **Multiple Transport Modes** - Supports HTTP, SSE, stdio, and hybrid modes
- **MCP Tools** - Provides `get_news`, `search_news`, `health_check`, `get_categories`, `refresh_news`
- **Multiple Categories** - Categories are dynamically generated from config, including Technology, Science, HackerNews, and 21 China News categories
- **Pluggable Sources** - Extensible `NewsSource` trait for adding custom data sources
- **In-memory Cache** - High-performance article cache with search functionality
- **Retry Mechanism** - Automatic retry for failed RSS fetch requests

## Quick Start

### Installation

```bash
# Clone repository
git clone https://github.com/KingingWang/news-mcp
cd news-mcp

# Build
cargo build --release

# Run tests
cargo test
```

### Run Server

```bash
# Run with default config
./target/release/news-mcp serve

# HTTP mode
./target/release/news-mcp serve --mode http --port 8080

# stdio mode (for Claude Desktop)
./target/release/news-mcp serve --mode stdio

# With background polling
./target/release/news-mcp serve --mode http --poll
```

## Configuration

Create `config.toml` file:

```toml
[server]
name = "news-mcp"
version = "0.1.0"
host = "127.0.0.1"
port = 8080
transport_mode = "http"  # Options: stdio, http, sse, hybrid

[poller]
interval_secs = 3600  # Poll every hour
enabled = true

[cache]
max_articles_per_category = 100

[logging]
level = "info"        # trace, debug, info, warn, error
enable_console = true
```

## MCP Tools

### get_news

Fetch articles by category. Categories are dynamically generated from config.

**Parameters:**
- `category` - News category (dynamically generated, default includes: technology, science, hackernews, instant, headlines, politics, etc.)
- `limit` - Number of articles to return (default 10, max 50)
- `format` - Output format (markdown, json, text)

**Example:**
```json
{
  "category": "technology",
  "limit": 5,
  "format": "markdown"
}
```

### search_news

Search cached articles by keyword.

**Parameters:**
- `query` - Search keyword
- `category` - Optional category filter
- `limit` - Number of results

**Example:**
```json
{
  "query": "AI",
  "category": "technology",
  "limit": 10
}
```

### health_check

Check server status and cache statistics.

**Parameters:**
- `check_type` - Check type (all, internal, external)
- `verbose` - Show detailed information

### get_categories

List available news categories with article counts.

### refresh_news

Manually refresh the news cache.

**Parameters:**
- `category` - Optional specific category to refresh

## HTTP API

### MCP Endpoint

```bash
# Initialize session
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "capabilities": {},
      "clientInfo": {"name": "test", "version": "1.0"}
    },
    "id": 1
  }'

# Call tool
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -H "mcp-session-id: <session-id>" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "get_news",
      "arguments": {"category": "technology", "limit": 5}
    },
    "id": 2
  }'
```

### Health Check

```bash
curl http://localhost:8080/health
```

## Claude Desktop Integration

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "news": {
      "command": "/path/to/news-mcp",
      "args": ["serve", "--mode", "stdio"]
    }
  }
}
```

## News Sources

### International News

| Category | Source |
|----------|--------|
| Technology | TechCrunch, Ars Technica, The Verge |
| Science | ScienceDaily |

### China News (chinanews.com.cn)

| Category | Name | RSS |
|----------|------|-----|
| instant | Instant News | scroll-news.xml |
| headlines | Headlines | importnews.xml |
| politics | Politics | china.xml |
| eastwest | East-West Dialogue | dxw.xml |
| society | Society | society.xml |
| finance | Finance | finance.xml |
| life | Life | life.xml |
| wellness | Health | jk.xml |
| greaterbayarea | Greater Bay Area | dwq.xml |
| chinese | Overseas Chinese | chinese.xml |
| video | Video | sp.xml |
| photo | Photo | photo.xml |
| creative | Creative | chuangyi.xml |
| live | Live | zhibo.xml |
| education | Education | edu.xml |
| law | Law | fz.xml |
| unitedfront | United Front | tx.xml |
| ethnicunity | Ethnic Unity | mz.xml |
| beltandroad | Belt and Road | ydyl.xml |
| theory | Theory | theory.xml |
| asean | ASEAN Trade | aseaninfo.xml |

**Usage:**
```json
{
  "category": "instant",
  "limit": 5,
  "format": "markdown"
}
```

## Project Structure

```
src/
├── main.rs           # Entry point
├── lib.rs            # Library root
├── cache/            # News cache
├── cli/              # Command line
├── config/           # Configuration
├── error/            # Error handling
├── poller/           # Background polling
├── server/           # MCP server
├── tools/            # MCP tools
├── service/          # News service
└── utils/            # Utilities
```

## Development

```bash
# Run tests
cargo test
cargo test --test unit
cargo test --test e2e

# Code formatting
cargo fmt

# Static analysis
cargo clippy

# Generate documentation
cargo doc --open
```

## Docker Deployment

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/news-mcp /usr/local/bin/
COPY config.toml /etc/news-mcp/
EXPOSE 8080
CMD ["news-mcp", "serve", "--mode", "http"]
```

Build and run:

```bash
docker build -t news-mcp .
docker run -p 8080:8080 news-mcp
```

## Documentation

- [Architecture](ARCHITECTURE.md) - System design and component overview
- [Contributing](CONTRIBUTING.md) - Development guidelines
- [Changelog](CHANGELOG.md) - Version history
- [Examples](examples/) - Usage guides for Claude Desktop, HTTP API, custom feeds, Docker

## License

MIT License - see [LICENSE](LICENSE)

## Acknowledgments

- [rust-mcp-sdk](https://github.com/rust-mcp-stack/rust-mcp-sdk) - MCP SDK
- [feed-rs](https://github.com/feed-rs/feed-rs) - RSS/Atom parsing
- [tokio](https://tokio.rs) - Async runtime