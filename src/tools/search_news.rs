//! search_news tool implementation
//!
//! Searches cached news by keyword.

use crate::cache::{NewsCache, NewsCategory};
use crate::tools::Tool;
use crate::utils::{format_articles_as_json, format_articles_as_markdown, format_articles_as_text};
use async_trait::async_trait;
use rust_mcp_sdk::macros;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Search news tool parameters
#[macros::mcp_tool(
    name = "search_news",
    title = "Search News",
    description = "Searches cached news by keyword in title and description.",
    destructive_hint = false,
    idempotent_hint = true,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Clone, Deserialize, Serialize, macros::JsonSchema)]
pub struct SearchNewsTool {
    /// Search query string
    #[json_schema(title = "Query", description = "Search query string")]
    pub query: String,

    /// Optional category filter
    #[json_schema(
        title = "Category",
        description = "Optional category filter",
        enum_values = ["technology", "business", "science", "health", "sports", "entertainment", "general", "world"]
    )]
    pub category: Option<String>,

    /// Number of results
    #[json_schema(
        title = "Limit",
        description = "Number of results (default: 10, max: 50)",
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

/// Search news tool implementation
pub struct SearchNewsToolImpl {
    cache: Arc<NewsCache>,
}

impl SearchNewsToolImpl {
    /// Create a new search_news tool
    pub fn new(cache: Arc<NewsCache>) -> Self {
        Self { cache }
    }
}

#[async_trait]
impl Tool for SearchNewsToolImpl {
    fn definition(&self) -> rust_mcp_sdk::schema::Tool {
        SearchNewsTool::tool()
    }

    async fn execute(
        &self,
        arguments: serde_json::Value,
    ) -> std::result::Result<
        rust_mcp_sdk::schema::CallToolResult,
        rust_mcp_sdk::schema::CallToolError,
    > {
        let params: SearchNewsTool = serde_json::from_value(arguments).map_err(|e| {
            rust_mcp_sdk::schema::CallToolError::invalid_arguments(
                "search_news",
                Some(format!("Invalid parameters: {}", e)),
            )
        })?;

        // Validate limit
        let limit = params.limit.unwrap_or(10).clamp(1, 50) as usize;

        // Parse category if provided
        let category = params
            .category
            .map(|c| c.parse::<NewsCategory>())
            .transpose()
            .map_err(|e| {
                rust_mcp_sdk::schema::CallToolError::from_message(format!(
                    "Invalid category: {}",
                    e
                ))
            })?;

        // Search articles
        let articles = self
            .cache
            .search(&params.query, category.as_ref())
            .map_err(|e| {
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
