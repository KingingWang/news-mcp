# Docker Deployment Guide / Docker 部署指南

This guide shows how to deploy News MCP Server using Docker.

本指南展示如何使用 Docker 部署 News MCP Server。

## Quick Start / 快速开始

### Build Docker Image / 构建镜像

```bash
# Build from source
docker build -t news-mcp:latest .

# Or with specific tag
docker build -t news-mcp:v0.1.0 .
```

### Run Container / 运行容器

```bash
# Basic run
docker run -d -p 8080:8080 --name news-server news-mcp:latest

# With custom config
docker run -d -p 8080:8080 \
  -v /path/to/config.toml:/etc/news-mcp/config.toml \
  --name news-server news-mcp:latest

# With environment variables
docker run -d -p 8080:8080 \
  -e NEWS_MCP_INTERVAL=1800 \
  -e NEWS_MCP_LOG_LEVEL=debug \
  --name news-server news-mcp:latest
```

### Verify Deployment / 验证部署

```bash
# Check health
curl http://localhost:8080/health

# Initialize MCP session
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05"},"id":1}'
```

## Dockerfile Details / Dockerfile 详情

The project includes a Dockerfile:

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

## Docker Compose / Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  news-mcp:
    image: news-mcp:latest
    container_name: news-mcp-server
    ports:
      - "8080:8080"
    environment:
      - NEWS_MCP_INTERVAL=1800
      - NEWS_MCP_LOG_LEVEL=info
    volumes:
      - ./config.toml:/etc/news-mcp/config.toml
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

Run with:
```bash
docker-compose up -d
docker-compose logs -f news-mcp
docker-compose down
```

## Configuration Options / 配置选项

### Environment Variables / 环境变量

| Variable | Default | Description |
|----------|---------|-------------|
| `NEWS_MCP_PORT` | 8080 | Server port |
| `NEWS_MCP_HOST` | 0.0.0.0 | Server host |
| `NEWS_MCP_TRANSPORT` | http | Transport mode |
| `NEWS_MCP_INTERVAL` | 3600 | Poll interval (seconds) |
| `NEWS_MCP_LOG_LEVEL` | info | Log level |

### Volume Mounts / 卷挂载

```bash
# Mount config file
-v /path/to/config.toml:/etc/news-mcp/config.toml

# Mount for persistent cache (future feature)
-v /path/to/cache:/var/lib/news-mcp/cache
```

## Production Deployment / 生产部署

### Resource Limits / 资源限制

```bash
docker run -d \
  --name news-mcp \
  --memory="512m" \
  --cpus="1.0" \
  -p 8080:8080 \
  news-mcp:latest
```

### Docker Compose with Limits:

```yaml
services:
  news-mcp:
    image: news-mcp:latest
    deploy:
      resources:
        limits:
          cpus: '1.0'
          memory: 512M
        reservations:
          cpus: '0.5'
          memory: 256M
```

### Kubernetes Deployment / Kubernetes 部署

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: news-mcp
spec:
  replicas: 1
  selector:
    matchLabels:
      app: news-mcp
  template:
    metadata:
      labels:
        app: news-mcp
    spec:
      containers:
      - name: news-mcp
        image: news-mcp:latest
        ports:
        - containerPort: 8080
        env:
        - name: NEWS_MCP_INTERVAL
          value: "1800"
        resources:
          limits:
            memory: "512Mi"
            cpu: "1"
---
apiVersion: v1
kind: Service
metadata:
  name: news-mcp-service
spec:
  selector:
    app: news-mcp
  ports:
  - port: 8080
    targetPort: 8080
  type: LoadBalancer
```

### Health Check Configuration / 健康检查配置

```yaml
# Kubernetes
livenessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 30
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5
```

## Logging / 日志管理

### View Logs / 查看日志

```bash
# Docker logs
docker logs news-mcp

# Follow logs
docker logs -f news-mcp

# Last 100 lines
docker logs --tail 100 news-mcp
```

### Log Levels / 日志级别

```bash
# Debug mode
docker run -e NEWS_MCP_LOG_LEVEL=debug news-mcp:latest

# Production (info level)
docker run -e NEWS_MCP_LOG_LEVEL=info news-mcp:latest
```

## Troubleshooting / 故障排除

### Container won't start / 容器无法启动

```bash
# Check logs
docker logs news-mcp

# Check if port is available
docker run --rm -p 8080:8080 news-mcp:latest
```

### No articles fetched / 无文章获取

```bash
# Check poll interval
docker exec news-mcp env | grep INTERVAL

# Manual refresh via API
curl -X POST http://localhost:8080/mcp \
  -H "mcp-session-id: SESSION" \
  -d '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"refresh_news"},"id":1}'
```

### Network issues / 网络问题

```bash
# Test connectivity
docker exec news-mcp curl -I https://techcrunch.com/feed/

# Check DNS
docker exec news-mcp nslookup techcrunch.com
```

## Multi-architecture Support / 多架构支持

Build for multiple architectures:

```bash
# Buildx for multi-arch
docker buildx build --platform linux/amd64,linux/arm64 \
  -t news-mcp:latest .
```

---

*See [HTTP API Guide](http-api.md) for API usage after deployment.*