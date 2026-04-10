# News MCP Server

一个基于 Rust 的 MCP (Model Context Protocol) 服务器，用于获取新闻 RSS 源，支持后台轮询、内存缓存和多种传输模式。

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## 功能特性

- **后台新闻轮询** - 定时从 RSS 源获取新闻并缓存
- **多种传输模式** - 支持 HTTP、SSE、stdio 和混合模式
- **MCP 工具** - 提供 `get_news`, `search_news`, `health_check` 等工具
- **多类别支持** - Technology, Business, Science, Health, Sports, Entertainment, General, World
- **内存缓存** - 高性能文章缓存，支持搜索功能
- **重试机制** - RSS 源获取失败自动重试

## 快速开始

### 安装

```bash
# 克隆仓库
git clone https://github.com/KingingWang/news-mcp
cd news-mcp

# 构建
cargo build --release

# 运行测试
cargo test
```

### 运行服务器

```bash
# 使用默认配置运行
./target/release/news-mcp serve

# HTTP 模式
./target/release/news-mcp serve --mode http --port 8080

# stdio 模式（用于 Claude Desktop 等）
./target/release/news-mcp serve --mode stdio

# 启用后台轮询
./target/release/news-mcp serve --mode http --poll
```

## 配置

创建 `config.toml` 文件：

```toml
[server]
name = "news-mcp"
version = "0.1.0"
host = "127.0.0.1"
port = 8080
transport_mode = "http"  # Options: stdio, http, sse, hybrid

[poller]
interval_secs = 3600  # 每小时轮询
enabled = true

[cache]
max_articles_per_category = 100

[logging]
level = "info"        # trace, debug, info, warn, error
enable_console = true
```

## MCP 工具

### get_news

获取指定类别的新闻。

**参数：**
- `category` - 新闻类别 (technology, business, science, health, sports, entertainment, general, world)
- `limit` - 返回文章数量（默认 10，最大 50）
- `format` - 输出格式 (markdown, json, text)

**示例：**
```json
{
  "category": "technology",
  "limit": 5,
  "format": "markdown"
}
```

### search_news

搜索缓存的新闻。

**参数：**
- `query` - 搜索关键词
- `category` - 可选类别过滤
- `limit` - 结果数量

**示例：**
```json
{
  "query": "AI",
  "category": "technology",
  "limit": 10
}
```

### health_check

检查服务器状态和缓存统计。

**参数：**
- `check_type` - 检查类型 (all, internal, external)
- `verbose` - 是否显示详细信息

### get_categories

获取可用的新闻类别列表。

### refresh_news

手动刷新新闻缓存。

**参数：**
- `category` - 可选指定类别刷新

## HTTP API

### MCP 端点

```bash
# 初始化会话
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

# 调用工具
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

### 健康检查

```bash
curl http://localhost:8080/health
```

## 与 Claude Desktop 集成

在 `claude_desktop_config.json` 中添加：

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

## 新闻源

| 类别 | 来源 |
|------|------|
| Technology | TechCrunch, Ars Technica, The Verge |
| Business | BBC Business |
| Science | ScienceDaily |
| Health | BBC Health |
| Sports | BBC Sport |
| Entertainment | BBC Entertainment |
| General | BBC News |
| World | BBC World |

## 项目结构

```
src/
├── main.rs           # 入口
├── lib.rs            # 库根
├── cache/            # 新闻缓存
├── cli/              # 命令行
├── config/           # 配置
├── error/            # 错误处理
├── poller/           # 后台轮询
├── server/           # MCP 服务器
├── tools/            # MCP 工具
├── service/          # 新闻服务
└── utils/            # 工具函数
```

## 开发

```bash
# 运行测试
cargo test
cargo test --test unit
cargo test --test e2e

# 代码格式化
cargo fmt

# 静态检查
cargo clippy

# 生成文档
cargo doc --open
```

## Docker 部署

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

构建运行：

```bash
docker build -t news-mcp .
docker run -p 8080:8080 news-mcp
```

## 许可证

MIT License - 详见 [LICENSE](LICENSE)

## 致谢

- [rust-mcp-sdk](https://github.com/rust-mcp-stack/rust-mcp-sdk) - MCP SDK
- [feed-rs](https://github.com/feed-rs/feed-rs) - RSS/Atom 解析
- [tokio](https://tokio.rs) - 异步运行时