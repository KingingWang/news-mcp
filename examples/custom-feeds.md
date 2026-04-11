# Custom Feed Configuration / 自定义新闻源配置

This guide shows how to add custom RSS feed sources.

本指南展示如何添加自定义 RSS 新闻源。

## Configuration File / 配置文件

The `config.toml` file supports custom feed definitions:

```toml
# Server configuration
[server]
name = "news-mcp"
port = 8080
transport_mode = "http"

# Custom feed sources
[feeds.mytech]
display_name = "My Tech News"
description = "Custom technology news sources"
urls = [
    "https://mytechblog.com/rss.xml",
    "https://anothersource.com/feed",
]
enabled = true

[feeds.mynews]
display_name = "我的新闻"
description = "自定义新闻源"
urls = ["https://custom-source.com/rss"]
enabled = true
```

## Built-in Categories / 内置类别

The following categories are built-in with default sources:

| Category | Default Sources |
|----------|-----------------|
| technology | TechCrunch, Ars Technica, The Verge |
| science | ScienceDaily |
| hackernews | Hacker News API (no RSS) |
| instant | chinanews.com.cn scroll-news |
| headlines | chinanews.com.cn importnews |
| ... | ... |

To override built-in sources, define the same category name:

```toml
[feeds.technology]
display_name = "Technology"
urls = [
    "https://my-custom-tech-source.com/rss.xml",
]
enabled = true
```

## Adding New Categories / 添加新类别

### Step 1: Add to config.toml

```toml
[feeds.reddit-tech]
display_name = "Reddit Tech"
description = "Technology discussions from Reddit"
urls = [
    "https://www.reddit.com/r/technology/.rss",
    "https://www.reddit.com/r/programming/.rss",
]
enabled = true
```

### Step 2: Add Category Enum (if needed)

For completely new categories not in the built-in list, you may need to:

1. Add to `NewsCategory` enum in `src/cache/news_cache.rs`
2. Add `FromStr` implementation for parsing
3. Add display name and description

**Note**: The system will attempt to parse the category name from config automatically.

## Environment Variables / 环境变量

Override configuration with environment variables:

```bash
# Server settings
export NEWS_MCP_PORT=9090
export NEWS_MCP_HOST=0.0.0.0
export NEWS_MCP_TRANSPORT=http
export NEWS_MCP_INTERVAL=1800  # Poll every 30 minutes
export NEWS_MCP_LOG_LEVEL=debug

# Run with custom config
./target/release/news-mcp serve --config custom.toml
```

## Complete Example / 完整示例

```toml
# config.toml - Custom configuration

[server]
name = "news-mcp"
version = "0.1.0"
host = "127.0.0.1"
port = 8080
transport_mode = "http"

[poller]
interval_secs = 1800  # 30 minutes
enabled = true

[cache]
max_articles_per_category = 50

[logging]
level = "info"
enable_console = true

# Override technology category
[feeds.technology]
display_name = "Technology"
description = "精选技术新闻"
urls = [
    "https://techcrunch.com/feed/",
    "https://www.theverge.com/rss/index.xml",
    "https://www.reddit.com/r/programming/.rss",
]
enabled = true

# New custom category
[feeds.crypto]
display_name = "Crypto News"
description = "加密货币新闻"
urls = [
    "https://coindesk.com/arc/outboundfeeds/rss/",
    "https://cointelegraph.com/rss",
]
enabled = true

# New custom category (Chinese)
[feeds.startup]
display_name = "创业资讯"
description = "Startup and entrepreneurship news"
urls = [
    "https://36kr.com/feed",
    "https://www.huxiu.com/rss",
]
enabled = true

# Disable a built-in category
[feeds.science]
enabled = false
```

## Testing Custom Config / 测试自定义配置

```bash
# Validate config
./target/release/news-mcp config --check

# Run with custom config
./target/release/news-mcp serve --config custom.toml

# Test via HTTP
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05"},"id":1}'

# Get custom category
curl -X POST http://localhost:8080/mcp \
  -H "mcp-session-id: SESSION" \
  -d '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"get_news","arguments":{"category":"crypto"}},"id":2}'
```

## Programmatic Custom Sources / 编程添加自定义源

For advanced use cases, implement the `NewsSource` trait:

```rust
use crate::cache::{NewsArticle, NewsCategory};
use crate::service::NewsSource;
use async_trait::async_trait;
use std::collections::HashMap;

pub struct MyCustomSource {
    api_key: String,
}

impl MyCustomSource {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl NewsSource for MyCustomSource {
    fn name(&self) -> &str {
        "My Custom API"
    }

    async fn fetch(&self) -> Result<HashMap<NewsCategory, Vec<NewsArticle>>> {
        // Fetch from your custom API
        let response = reqwest::get("https://api.mysource.com/news")
            .await?
            .json::<Vec<Article>>()
            .await?;

        // Convert to NewsArticle
        let articles = response.iter().map(|a| {
            NewsArticle::new(
                a.title.clone(),
                Some(a.summary.clone()),
                a.url.clone(),
                "My Source".to_string(),
                NewsCategory::Custom,
                Some(a.published),
                Some(a.author.clone()),
            )
        }).collect();

        let mut map = HashMap::new();
        map.insert(NewsCategory::Custom, articles);
        Ok(map)
    }
}
```

See [ARCHITECTURE.md](../ARCHITECTURE.md) for more on extending the system.

---

*See [HTTP API Guide](http-api.md) for API usage.*