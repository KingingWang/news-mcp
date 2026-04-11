# HTTP API Usage Guide / HTTP API 使用指南

This guide shows how to use the News MCP Server HTTP API.

本指南展示如何使用 News MCP Server HTTP API。

## Starting HTTP Server / 启动 HTTP 服务器

```bash
# Start HTTP server
./target/release/news-mcp serve --mode http --port 8080

# With background polling
./target/release/news-mcp serve --mode http --port 8080 --poll
```

## MCP Protocol Endpoints / MCP 协议端点

### Initialize Session / 初始化会话

**Request**:
```bash
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "capabilities": {},
      "clientInfo": {"name": "test-client", "version": "1.0"}
    },
    "id": 1
  }'
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {"tools": {}},
    "serverInfo": {"name": "news-mcp", "version": "0.1.0"}
  },
  "id": 1
}
```

**Important**: Save the `mcp-session-id` from response headers for subsequent requests.

### Call Tools / 调用工具

Use the session ID from the initialize response:

```bash
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -H "mcp-session-id: YOUR_SESSION_ID" \
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

## Tool Examples / 工具示例

### get_news - Get News by Category / 获取新闻

```bash
# Get technology news
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -H "mcp-session-id: SESSION_ID" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "get_news",
      "arguments": {
        "category": "technology",
        "limit": 5,
        "format": "markdown"
      }
    },
    "id": 3
  }'
```

**Parameters**:
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| category | string | Yes | - | News category (technology, science, hackernews, etc.) |
| limit | number | No | 10 | Number of articles (max 50) |
| format | string | No | markdown | Output format (markdown, json, text) |

### search_news - Search Articles / 搜索新闻

```bash
# Search for AI articles
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -H "mcp-session-id: SESSION_ID" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "search_news",
      "arguments": {
        "query": "artificial intelligence",
        "category": "technology",
        "limit": 10
      }
    },
    "id": 4
  }'
```

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| query | string | Yes | Search keyword |
| category | string | No | Filter by category |
| limit | number | No | Max results (default 10) |

### get_categories - List Categories / 获取类别

```bash
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -H "mcp-session-id: SESSION_ID" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "get_categories",
      "arguments": {}
    },
    "id": 5
  }'
```

### health_check - Server Status / 健康检查

```bash
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -H "mcp-session-id: SESSION_ID" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "health_check",
      "arguments": {
        "check_type": "all",
        "verbose": true
      }
    },
    "id": 6
  }'
```

**Parameters**:
| Name | Type | Description |
|------|------|-------------|
| check_type | string | "all", "internal", or "external" |
| verbose | boolean | Show detailed info |

### refresh_news - Refresh Cache / 刷新缓存

```bash
# Refresh all categories
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -H "mcp-session-id: SESSION_ID" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "refresh_news",
      "arguments": {}
    },
    "id": 7
  }'

# Refresh specific category
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -H "mcp-session-id: SESSION_ID" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "refresh_news",
      "arguments": {"category": "technology"}
    },
    "id": 8
  }'
```

## Health Endpoint / 健康端点

Simple HTTP health check (no MCP session required):

```bash
curl http://localhost:8080/health
```

**Response**:
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "articles_cached": 1234,
  "categories": 25
}
```

## Available Categories / 可用类别

| Category | Description | Sources |
|----------|-------------|---------|
| technology | Tech news | TechCrunch, Ars Technica, The Verge |
| science | Science news | ScienceDaily |
| hackernews | Hacker News | HN API |
| instant | 即时新闻 | chinanews.com.cn |
| headlines | 要闻导读 | chinanews.com.cn |
| politics | 时政新闻 | chinanews.com.cn |
| finance | 败经新闻 | chinanews.com.cn |
| ... | ... | ... |

See [README.md](../README.md) for full category list.

## Error Handling / 错误处理

### Common Errors / 常见错误

**Invalid Session**:
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32600,
    "message": "Invalid session"
  },
  "id": 2
}
```

**Invalid Category**:
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32602,
    "message": "Invalid category: unknown"
  },
  "id": 3
}
```

**No Articles**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [{"type": "text", "text": "No articles found for category: unknown"}]
  },
  "id": 4
}
```

## Python Client Example / Python 客户端示例

```python
import requests
import json

class NewsMCPClient:
    def __init__(self, base_url="http://localhost:8080"):
        self.base_url = base_url
        self.session_id = None

    def initialize(self):
        resp = requests.post(
            f"{self.base_url}/mcp",
            json={
                "jsonrpc": "2.0",
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {"name": "python-client", "version": "1.0"}
                },
                "id": 1
            }
        )
        self.session_id = resp.headers.get("mcp-session-id")
        return resp.json()

    def get_news(self, category, limit=10):
        resp = requests.post(
            f"{self.base_url}/mcp",
            headers={"mcp-session-id": self.session_id},
            json={
                "jsonrpc": "2.0",
                "method": "tools/call",
                "params": {
                    "name": "get_news",
                    "arguments": {"category": category, "limit": limit}
                },
                "id": 2
            }
        )
        return resp.json()

# Usage
client = NewsMCPClient()
client.initialize()
news = client.get_news("technology", 5)
print(news)
```

---

*See [Claude Desktop Guide](claude-desktop.md) for Claude Desktop integration.*