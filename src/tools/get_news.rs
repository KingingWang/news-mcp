//! get_news tool implementation
//!
//! Fetches latest news by category from cache.

use crate::cache::{NewsCache, NewsCategory};
use crate::tools::Tool;
use crate::utils::{format_articles_as_json, format_articles_as_markdown, format_articles_as_text};
use async_trait::async_trait;
use rust_mcp_sdk::macros;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Get news tool parameters
#[macros::mcp_tool(
    name = "get_news",
    title = "Get News",
    description = "Fetches latest news by category from memory cache.",
    destructive_hint = false,
    idempotent_hint = true,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Clone, Deserialize, Serialize, macros::JsonSchema)]
pub struct GetNewsTool {
    /// News category
    #[json_schema(
        title = "Category",
        description = "News category (technology, business, science, health, sports, entertainment, general, world)",
        default = "technology"
    )]
    pub category: Option<String>,

    /// Number of articles to return
    #[json_schema(
        title = "Limit",
        description = "Number of articles to return (default: 10, max: 50)",
        default = 10,
        minimum = 1,
        maximum = 50
    )]
    pub limit: Option<u32>,

    /// Output format
    #[json_schema(
        title = "Format",
        description = "Output format (markdown, json, text)",
        default = "markdown",
        enum_values = ["markdown", "json", "text"]
    )]
    pub format: Option<String>,
}

/// Get news tool implementation
pub struct GetNewsToolImpl {
    cache: Arc<NewsCache>,
}

impl GetNewsToolImpl {
    /// Create a new get_news tool
    pub fn new(cache: Arc<NewsCache>) -> Self {
        Self { cache }
    }
}

#[async_trait]
impl Tool for GetNewsToolImpl {
    fn definition(&self) -> rust_mcp_sdk::schema::Tool {
        GetNewsTool::tool()
    }

    async fn execute(
        &self,
        arguments: serde_json::Value,
    ) -> std::result::Result<
        rust_mcp_sdk::schema::CallToolResult,
        rust_mcp_sdk::schema::CallToolError,
    > {
        let params: GetNewsTool = serde_json::from_value(arguments).map_err(|e| {
            rust_mcp_sdk::schema::CallToolError::invalid_arguments(
                "get_news",
                Some(format!("Invalid parameters: {}", e)),
            )
        })?;

        // Validate limit
        let limit = params.limit.unwrap_or(10).clamp(1, 50) as usize;

        // Parse category
        let category_str = params.category.unwrap_or_else(|| "technology".to_string());
        let category: NewsCategory = category_str.parse().map_err(|e| {
            rust_mcp_sdk::schema::CallToolError::from_message(format!("Invalid category: {}", e))
        })?;

        // Get articles from cache
        let articles = self.cache.get_category_news(&category).map_err(|e| {
            rust_mcp_sdk::schema::CallToolError::from_message(format!("Cache error: {}", e))
        })?;
        let limited_articles: Vec<_> = articles.into_iter().take(limit).collect();

        // Format output
        let format = params.format.unwrap_or_else(|| "markdown".to_string());
        let content = match format.to_lowercase().as_str() {
            "markdown" => format_articles_as_markdown(&limited_articles),
            "json" => format_articles_as_json(&limited_articles),
            "text" => format_articles_as_text(&limited_articles),
            _ => {
                return Err(rust_mcp_sdk::schema::CallToolError::from_message(format!(
                    "Invalid format: {}",
                    format
                )))
            }
        };

        Ok(rust_mcp_sdk::schema::CallToolResult::text_content(vec![
            content.into(),
        ]))
    }
}
