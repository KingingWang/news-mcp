# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
# Build
cargo build
cargo build --release

# Test
cargo test                          # Run all tests
cargo test --test unit              # Run unit tests only
cargo test --test e2e               # Run e2e tests only
cargo test test_name                # Run single test

# Lint & Format
cargo fmt
cargo clippy

# Run server
cargo run -- serve                          # Default mode (from config.toml)
cargo run -- serve --mode stdio             # stdio mode for Claude Desktop
cargo run -- serve --mode http --port 8080  # HTTP mode
cargo run -- serve --mode http --poll       # HTTP with background polling

# Direct binary
./target/release/news-mcp serve --mode http --port 9090 --poll
```

## Architecture Overview

This is a Rust MCP (Model Context Protocol) server that fetches news from RSS feeds.

### Core Components

**Cache Layer** (`src/cache/news_cache.rs`)
- Thread-safe in-memory cache using `RwLock<HashMap<...>>`
- Stores `NewsArticle` structs by `NewsCategory`
- Supports search across title/description

**Poller** (`src/poller/news_poller.rs`)
- Background task that polls RSS feeds at configured intervals
- `initial_poll_completed` AtomicBool tracks first poll status
- `wait_for_initial_poll()` blocks until cache is populated

**Server** (`src/server/`)
- `NewsMcpServer`: Core server struct with config, cache, tool_registry
- `NewsMcpHandler`: Implements MCP protocol handlers
- Transport modes: stdio, HTTP, SSE, hybrid (configured in `config.toml`)

**Tools** (`src/tools/`)
- `get_news`: Fetch articles by category from cache
- `search_news`: Search articles by keyword
- `get_categories`: List available categories with counts
- `health_check`: Server status and cache stats
- `refresh_news`: Manual cache refresh trigger

**Service** (`src/service/news_service.rs`)
- `NewsService`: HTTP client with retry middleware for fetching RSS feeds
- Uses `feed-rs` for RSS/Atom parsing

### Key Flows

1. **Server Startup** (`src/cli/serve_cmd.rs`)
   - Load config → Create cache → Start poller (if enabled) → Wait for initial poll → Start transport

2. **MCP Tool Execution** (`src/server/handler/standard.rs`)
   - Handler receives request → ToolRegistry dispatches to tool → Tool reads from cache → Returns result

3. **Background Polling**
   - Poller fetches all categories concurrently → Parses RSS → Updates cache → Sets `initial_poll_completed`

### Configuration

`config.toml` controls server behavior:
```toml
[server]
transport_mode = "http"  # stdio | http | sse | hybrid
port = 8080

[poller]
interval_secs = 3600
enabled = true

[cache]
max_articles_per_category = 100
```

### Testing Structure

- `tests/unit/`: Cache, config, service, tool tests
- `tests/e2e/`: Transport mode integration tests
- Uses `wiremock` for HTTP mocking, `tempfile` for config tests

### RSS Feed Sources

Currently configured feeds (see `src/utils/mod.rs`):

**Technology**: TechCrunch, Ars Technica, The Verge
**Science**: ScienceDaily

**China News Categories** (21 categories from chinanews.com.cn):
- instant (Instant News)
- headlines (Headlines)
- politics (Politics)
- eastwest (East-West Dialogue)
- society (Society)
- finance (Finance)
- life (Life)
- wellness (Health)
- greaterbayarea (Greater Bay Area)
- chinese (Overseas Chinese)
- video (Video)
- photo (Photo)
- creative (Creative)
- live (Live)
- education (Education)
- law (Law)
- unitedfront (United Front)
- ethnicunity (Ethnic Unity)
- beltandroad (Belt and Road)
- theory (Theory)
- asean (ASEAN Trade)

### MCP Protocol Notes

- HTTP mode requires session initialization before tool calls
- Session ID returned in `initialize` response must be included in subsequent requests via `mcp-session-id` header
- Health endpoint available at `GET /health` when using HTTP transport