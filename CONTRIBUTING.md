# Contributing to News MCP / 贡献指南

Thank you for your interest in contributing to News MCP! This document provides guidelines for contributing to the project.

感谢您对 News MCP 的贡献兴趣！本文档提供贡献指南。

## Table of Contents / 目录

- [Code of Conduct](#code-of-conduct)
- [Development Setup](#development-setup)
- [Building & Testing](#building--testing)
- [Code Style](#code-style)
- [Pull Request Process](#pull-request-process)
- [Commit Messages](#commit-messages)
- [Adding New Features](#adding-new-features)

## Code of Conduct / 行为准则

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow

## Development Setup / 开发环境设置

### Prerequisites / 前置条件

- **Rust**: 1.75 or later ([Install Rust](https://rustup.rs/))
- **Git**: For version control
- **Editor**: VS Code, IntelliJ IDEA, or any Rust-compatible editor

### Quick Setup / 快速设置

```bash
# Clone the repository
git clone https://github.com/KingingWang/news-mcp
cd news-mcp

# Build the project
cargo build

# Run tests
cargo test

# Run the server (quick test)
cargo run -- serve --mode stdio
```

### Recommended Tools / 推荐工具

```bash
# Install rust-analyzer (VS Code extension recommended)
# Install clippy for linting
rustup component add clippy

# Install rustfmt for formatting
rustup component add rustfmt
```

## Building & Testing / 构建和测试

### Build Commands / 构建命令

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Build with all features
cargo build --all-features
```

### Test Commands / 测试命令

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --test unit

# Run e2e tests only
cargo test --test e2e

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

### Running the Server / 运行服务器

```bash
# stdio mode (for Claude Desktop)
cargo run -- serve --mode stdio

# HTTP mode
cargo run -- serve --mode http --port 8080

# HTTP mode with polling
cargo run -- serve --mode http --poll

# With custom config
cargo run -- serve --config custom.toml
```

## Code Style / 代码风格

### Formatting / 格式化

We use `rustfmt` for consistent formatting:

```bash
# Check formatting
cargo fmt -- --check

# Apply formatting
cargo fmt
```

### Linting / 代码检查

We use `clippy` for code quality:

```bash
# Run clippy (must pass in CI)
cargo clippy --all-targets -- -D warnings

# Run with all features
cargo clippy --all-features --all-targets -- -D warnings
```

### Code Guidelines / 代码准则

1. **Documentation**: Add doc comments (`//!` for modules, `///` for items)
2. **Error Handling**: Use `Result<T>` and the project's `Error` type
3. **Async**: Use `async_trait` for trait methods
4. **Thread Safety**: Use `Arc`, `RwLock` for shared state
5. **Testing**: Add tests for new functionality

**代码准则**:
1. **文档**: 添加文档注释（模块用 `//!`，项用 `///`）
2. **错误处理**: 使用 `Result<T>` 和项目的 `Error` 类型
3. **异步**: trait 方法使用 `async_trait`
4. **线程安全**: 共享状态使用 `Arc`, `RwLock`
5. **测试**: 为新功能添加测试

## Pull Request Process / PR 提交流程

### Before Submitting / 提交前检查

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Run clippy: `cargo clippy -- -D warnings`
6. Format code: `cargo fmt`
7. Commit changes

### PR Requirements / PR 要求

- [ ] Tests pass (`cargo test`)
- [ ] Clippy passes (`cargo clippy -- -D warnings`)
- [ ] Code formatted (`cargo fmt -- --check`)
- [ ] Documentation updated if needed
- [ ] CHANGELOG.md updated (if applicable)

### Review Process / 审核流程

1. Submit PR against `main` branch
2. CI checks must pass
3. At least one review required
4. Address review feedback
5. Merge when approved

## Commit Messages / 提交信息格式

Use conventional commit format:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types / 类型

| Type | Description |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation change |
| `style` | Code style (formatting, etc.) |
| `refactor` | Code refactoring |
| `test` | Adding/fixing tests |
| `chore` | Maintenance tasks |

### Examples / 示例

```
feat(service): add Reddit news source support

Implements NewsSource trait for Reddit API fetching.
Includes rate limiting and error handling.

Closes #42
```

```
fix(cache): resolve race condition in search function

The search function was not properly acquiring read lock.
Fixed by using read() consistently.
```

## Adding New Features / 添加新功能

### Adding a New News Source / 添加新新闻源

1. Create new service file in `src/service/`:
```rust
// src/service/my_source.rs

use crate::cache::{NewsArticle, NewsCategory};
use crate::error::Result;
use crate::service::NewsSource;
use async_trait::async_trait;
use std::collections::HashMap;

pub struct MySource {
    // Your configuration
}

impl MySource {
    pub fn new() -> Self {
        Self { }
    }
}

#[async_trait]
impl NewsSource for MySource {
    fn name(&self) -> &str {
        "My Source"
    }
    
    async fn fetch(&self) -> Result<HashMap<NewsCategory, Vec<NewsArticle>>> {
        // Fetch logic here
        Ok(HashMap::new())
    }
}
```

2. Export in `src/service/mod.rs`:
```rust
mod my_source;
pub use my_source::*;
```

3. Register in `src/cli/serve_cmd.rs`:
```rust
let sources: Vec<Arc<dyn NewsSource>> = vec![
    Arc::new(NewsService::with_config(config.clone())),
    Arc::new(HnService::new()),
    Arc::new(MySource::new()), // Add here
];
```

### Adding a New MCP Tool / 添加新 MCP 工具

1. Create tool file in `src/tools/`:
```rust
// src/tools/my_tool.rs

use crate::cache::NewsCache;
use crate::error::Result;

pub struct MyTool;

impl MyTool {
    pub async fn execute(&self, cache: &NewsCache) -> Result<String> {
        // Tool logic
        Ok("result".to_string())
    }
}
```

2. Export in `src/tools/mod.rs`:
```rust
mod my_tool;
pub use my_tool::*;
```

3. Register in handler (see `src/server/handler/standard.rs`)

### Adding a New Category / 添加新类别

1. Add to `NewsCategory` enum in `src/cache/news_cache.rs`:
```rust
pub enum NewsCategory {
    // ... existing categories
    MyNewCategory,
}
```

2. Add `FromStr` implementation:
```rust
"mynewcategory" | "新类别" => Ok(NewsCategory::MyNewCategory),
```

3. Add `display_name` and `description`:
```rust
NewsCategory::MyNewCategory => "新类别",
```

4. Add feed URLs in config or `src/utils/mod.rs`

## Getting Help / 获取帮助

- **Issues**: [GitHub Issues](https://github.com/KingingWang/news-mcp/issues)
- **Discussions**: [GitHub Discussions](https://github.com/KingingWang/news-mcp/discussions)

---

Thank you for contributing! 🎉

感谢您的贡献！🎉