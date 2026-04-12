# News MCP 服务器

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/KingingWang/news-mcp/workflows/CI/badge.svg)](https://github.com/KingingWang/news-mcp/actions)

一个基于 Rust 的 MCP (Model Context Protocol) 服务器，用于获取新闻 RSS 源，支持后台轮询、内存缓存和多种传输模式。

## 功能特性

- **后台新闻轮询** - 定时从 RSS 源获取新闻并缓存
- **多种传输模式** - 支持 HTTP、SSE、stdio 和混合模式
- **MCP 工具** - 提供 `get_news`、`search_news`、`health_check`、`get_categories`、`refresh_news`
- **多类别支持** - 类别根据配置文件动态生成，默认包含 Technology、Science、HackerNews 及 21 个中国新闻网分类
- **可插拔新闻源** - 通过 `NewsSource` trait 轻松添加自定义数据源
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

# stdio 模式（用于 Claude Desktop）
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
transport_mode = "http"  # 选项：stdio, http, sse, hybrid

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

获取指定类别的新闻。类别列表根据配置文件动态生成。

**参数：**
- `category` - 新闻类别（根据配置动态生成，默认包含：technology、science、hackernews、instant、headlines、politics 等）
- `limit` - 返回文章数量（默认 10，最大 50）
- `format` - 输出格式（markdown、json、text）

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
- `check_type` - 检查类型（all、internal、external）
- `verbose` - 是否显示详细信息

### get_categories

获取可用的新闻类别列表（包含文章数量）。

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

## Claude Desktop 集成

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

### 国外新闻

| 类别 | 来源 |
|------|------|
| Technology | TechCrunch、Ars Technica、The Verge |
| Science | ScienceDaily |

### 中国新闻网（chinanews.com.cn）

| 类别 | 名称 | RSS |
|------|------|-----|
| instant | 即时新闻 | scroll-news.xml |
| headlines | 要闻导读 | importnews.xml |
| politics | 时政新闻 | china.xml |
| eastwest | 东西问 | dxw.xml |
| society | 社会新闻 | society.xml |
| finance | 财经新闻 | finance.xml |
| life | 生活 | life.xml |
| wellness | 健康 | jk.xml |
| greaterbayarea | 大湾区 | dwq.xml |
| chinese | 华人 | chinese.xml |
| video | 视频 | sp.xml |
| photo | 图片 | photo.xml |
| creative | 创意 | chuangyi.xml |
| live | 直播 | zhibo.xml |
| education | 教育 | edu.xml |
| law | 法治 | fz.xml |
| unitedfront | 同心 | tx.xml |
| ethnicunity | 铸牢中华民族共同体意识 | mz.xml |
| beltandroad | 一带一路 | ydyl.xml |
| theory | 理论 | theory.xml |
| asean | 中国—东盟商贸资讯平台 | aseaninfo.xml |

**使用方式：**
```json
{
  "category": "instant",
  "limit": 5,
  "format": "markdown"
}
```

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

## 文档

- [架构设计](ARCHITECTURE.md) - 系统设计和组件概述
- [贡献指南](CONTRIBUTING.md) - 开发指南
- [更新日志](CHANGELOG.md) - 版本历史
- [示例](examples/) - Claude Desktop、HTTP API、自定义源、Docker 使用指南

## 许可证

MIT License - 详见 [LICENSE](LICENSE)

## 致谢

- [rust-mcp-sdk](https://github.com/rust-mcp-stack/rust-mcp-sdk) - MCP SDK
- [feed-rs](https://github.com/feed-rs/feed-rs) - RSS/Atom 解析
- [tokio](https://tokio.rs) - 异步运行时