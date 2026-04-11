//! search_news tool implementation
//!
//! Searches cached news by keyword.

use crate::cache::{NewsCache, NewsCategory};
use crate::config::FeedSourceConfig;
use crate::tools::Tool;
use crate::utils::{format_articles_as_json, format_articles_as_markdown, format_articles_as_text};
use async_trait::async_trait;
use rust_mcp_sdk::macros;
use rust_mcp_sdk::schema::{ToolAnnotations, ToolInputSchema};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    feeds: HashMap<String, FeedSourceConfig>,
}

impl SearchNewsToolImpl {
    /// Create a new search_news tool
    pub fn new(cache: Arc<NewsCache>, feeds: HashMap<String, FeedSourceConfig>) -> Self {
        Self { cache, feeds }
    }

    /// Build dynamic category description from feeds configuration
    fn build_category_description(&self) -> String {
        if self.feeds.is_empty() {
            "Optional category filter".to_string()
        } else {
            let categories: Vec<String> = self
                .feeds
                .iter()
                .filter(|(_, config)| config.enabled)
                .map(|(key, config)| {
                    config
                        .display_name
                        .as_ref()
                        .map(|name| format!("{} ({})", name, key))
                        .unwrap_or_else(|| key.clone())
                })
                .collect();
            format!("Optional category filter. Available: {}", categories.join(", "))
        }
    }

    /// Build enum values from feeds configuration
    fn build_category_enum_values(&self) -> Vec<String> {
        self.feeds
            .iter()
            .filter(|(_, config)| config.enabled)
            .map(|(key, _)| key.clone())
            .collect()
    }

    /// Build properties for the tool input schema
    fn build_input_schema_properties(&self) -> std::collections::BTreeMap<String, serde_json::Map<String, serde_json::Value>> {
        let mut properties = std::collections::BTreeMap::new();

        // Query property (required)
        let mut query_prop = serde_json::Map::new();
        query_prop.insert("type".to_string(), serde_json::json!("string"));
        query_prop.insert("title".to_string(), serde_json::json!("Query"));
        query_prop.insert(
            "description".to_string(),
            serde_json::json!("Search query string"),
        );
        properties.insert("query".to_string(), query_prop);

        // Category property (optional)
        let mut category_prop = serde_json::Map::new();
        category_prop.insert("type".to_string(), serde_json::json!("string"));
        category_prop.insert("title".to_string(), serde_json::json!("Category"));
        category_prop.insert(
            "description".to_string(),
            serde_json::json!(self.build_category_description()),
        );
        let enum_values = self.build_category_enum_values();
        if !enum_values.is_empty() {
            category_prop.insert("enum".to_string(), serde_json::json!(enum_values));
        }
        properties.insert("category".to_string(), category_prop);

        // Limit property
        let mut limit_prop = serde_json::Map::new();
        limit_prop.insert("type".to_string(), serde_json::json!("integer"));
        limit_prop.insert("title".to_string(), serde_json::json!("Limit"));
        limit_prop.insert(
            "description".to_string(),
            serde_json::json!("Number of results (default: 10, max: 50)"),
        );
        limit_prop.insert("default".to_string(), serde_json::json!(10));
        limit_prop.insert("minimum".to_string(), serde_json::json!(1));
        limit_prop.insert("maximum".to_string(), serde_json::json!(50));
        properties.insert("limit".to_string(), limit_prop);

        // Format property
        let mut format_prop = serde_json::Map::new();
        format_prop.insert("type".to_string(), serde_json::json!("string"));
        format_prop.insert("title".to_string(), serde_json::json!("Format"));
        format_prop.insert(
            "description".to_string(),
            serde_json::json!("Output format (markdown, json, text)"),
        );
        format_prop.insert("default".to_string(), serde_json::json!("markdown"));
        format_prop.insert(
            "enum".to_string(),
            serde_json::json!(["markdown", "json", "text"]),
        );
        properties.insert("format".to_string(), format_prop);

        properties
    }
}

#[async_trait]
impl Tool for SearchNewsToolImpl {
    fn definition(&self) -> rust_mcp_sdk::schema::Tool {
        rust_mcp_sdk::schema::Tool {
            name: "search_news".to_string(),
            title: Some("Search News".to_string()),
            description: Some("Searches cached news by keyword in title and description.".to_string()),
            input_schema: ToolInputSchema::new(
                vec!["query".to_string()], // query is required
                Some(self.build_input_schema_properties()),
                None,
            ),
            annotations: Some(ToolAnnotations {
                title: Some("Search News".to_string()),
                read_only_hint: Some(true),
                destructive_hint: Some(false),
                idempotent_hint: Some(true),
                open_world_hint: Some(false),
            }),
            output_schema: None,
            icons: vec![],
            meta: None,
            execution: None,
        }
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
