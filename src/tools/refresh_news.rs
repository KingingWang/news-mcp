//! refresh_news tool implementation
//!
//! Manually triggers a refresh of the news cache.
//! Returns current cached data immediately, updates cache in background.

use crate::cache::{NewsCache, NewsCategory};
use crate::service::NewsService;
use crate::tools::Tool;
use async_trait::async_trait;
use rust_mcp_sdk::macros;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

/// Refresh news tool parameters
#[macros::mcp_tool(
    name = "refresh_news",
    title = "Refresh News",
    description = "Manually triggers a refresh of the news cache. Optionally refresh a specific category. Returns current cached data immediately, updates in background.",
    destructive_hint = false,
    idempotent_hint = false,
    open_world_hint = true,
    read_only_hint = false
)]
#[derive(Debug, Clone, Deserialize, Serialize, macros::JsonSchema)]
pub struct RefreshNewsTool {
    /// Optional specific category to refresh
    #[json_schema(
        title = "Category",
        description = "Optional specific category to refresh",
        enum_values = ["technology", "business", "science", "health", "sports", "entertainment", "general", "world"]
    )]
    pub category: Option<String>,
}

/// Refresh news tool implementation
pub struct RefreshNewsToolImpl {
    cache: Arc<NewsCache>,
}

impl RefreshNewsToolImpl {
    /// Create a new refresh_news tool
    pub fn new(cache: Arc<NewsCache>) -> Self {
        Self { cache }
    }
}

#[async_trait]
impl Tool for RefreshNewsToolImpl {
    fn definition(&self) -> rust_mcp_sdk::schema::Tool {
        RefreshNewsTool::tool()
    }

    async fn execute(
        &self,
        arguments: serde_json::Value,
    ) -> std::result::Result<
        rust_mcp_sdk::schema::CallToolResult,
        rust_mcp_sdk::schema::CallToolError,
    > {
        let params: RefreshNewsTool = if arguments.is_null() {
            RefreshNewsTool { category: None }
        } else {
            serde_json::from_value(arguments).map_err(|e| {
                rust_mcp_sdk::schema::CallToolError::invalid_arguments(
                    "refresh_news",
                    Some(format!("Invalid parameters: {}", e)),
                )
            })?
        };

        let mut output = String::new();
        output.push_str("# Refresh News Status\n\n");

        // Get current cached data first (before refresh)
        let current_data = if let Some(category_str) = &params.category {
            let category: NewsCategory = category_str.parse().map_err(|e| {
                rust_mcp_sdk::schema::CallToolError::from_message(format!(
                    "Invalid category: {}",
                    e
                ))
            })?;
            output.push_str(&format!(
                "Category: {}\n\n",
                category.display_name()
            ));
            self.cache.get_category_news(&category).unwrap_or_default()
        } else {
            output.push_str("All categories\n\n");
            vec![]
        };

        output.push_str(&format!(
            "Current cached articles: {}\n\n",
            current_data.len()
        ));
        output.push_str("Status: Refresh triggered in background\n");
        output.push_str("Note: New data will be available on next request\n");

        // Spawn background task to refresh cache
        let cache = self.cache.clone();
        let category_param = params.category.clone();

        tokio::spawn(async move {
            let service = NewsService::new();

            if let Some(category_str) = category_param {
                match category_str.parse::<NewsCategory>() {
                    Ok(category) => {
                        info!("Background refresh started for category: {}", category.display_name());
                        match service.fetch_category(category).await {
                            Ok(articles) => {
                                let count = articles.len();
                                if let Err(e) = cache.set_category_news(category, articles) {
                                    tracing::error!("Failed to update cache: {}", e);
                                } else {
                                    info!("Background refresh completed: {} articles for {}", count, category.display_name());
                                }
                            }
                            Err(e) => {
                                tracing::error!("Background refresh failed: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Invalid category in background refresh: {}", e);
                    }
                }
            } else {
                info!("Background refresh started for all categories");
                match service.fetch_all_categories().await {
                    Ok(results) => {
                        let mut total = 0;
                        for (category, articles) in results {
                            let count = articles.len();
                            total += count;
                            if let Err(e) = cache.set_category_news(category, articles) {
                                tracing::error!("Failed to update cache for {}: {}", category, e);
                            }
                        }
                        info!("Background refresh completed: {} total articles", total);
                    }
                    Err(e) => {
                        tracing::error!("Background refresh failed: {}", e);
                    }
                }
            }
        });

        Ok(rust_mcp_sdk::schema::CallToolResult::text_content(vec![
            output.into(),
        ]))
    }
}
