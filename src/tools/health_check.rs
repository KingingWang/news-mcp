//! health_check tool implementation
//!
//! Checks server status and cache statistics.

use crate::cache::NewsCache;
use crate::tools::Tool;
use async_trait::async_trait;
use rust_mcp_sdk::macros;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Health check tool parameters
#[macros::mcp_tool(
    name = "health_check",
    title = "Health Check",
    description = "Checks server status, cache statistics, and last poll time.",
    destructive_hint = false,
    idempotent_hint = true,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Clone, Deserialize, Serialize, macros::JsonSchema)]
pub struct HealthCheckTool {
    /// Check type
    #[json_schema(
        title = "Check Type",
        description = "Check type (all, internal, external)",
        default = "all",
        enum_values = ["all", "internal", "external"]
    )]
    pub check_type: Option<String>,

    /// Show detailed information
    #[json_schema(
        title = "Verbose",
        description = "Show detailed information",
        default = false
    )]
    pub verbose: Option<bool>,
}

/// Health check tool implementation
pub struct HealthCheckToolImpl {
    cache: Arc<NewsCache>,
}

impl HealthCheckToolImpl {
    /// Create a new health_check tool
    pub fn new(cache: Arc<NewsCache>) -> Self {
        Self { cache }
    }
}

#[async_trait]
impl Tool for HealthCheckToolImpl {
    fn definition(&self) -> rust_mcp_sdk::schema::Tool {
        HealthCheckTool::tool()
    }

    async fn execute(
        &self,
        arguments: serde_json::Value,
    ) -> std::result::Result<
        rust_mcp_sdk::schema::CallToolResult,
        rust_mcp_sdk::schema::CallToolError,
    > {
        let params: HealthCheckTool = if arguments.is_null() {
            HealthCheckTool {
                check_type: None,
                verbose: None,
            }
        } else {
            serde_json::from_value(arguments).map_err(|e| {
                rust_mcp_sdk::schema::CallToolError::invalid_arguments(
                    "health_check",
                    Some(format!("Invalid parameters: {}", e)),
                )
            })?
        };

        let verbose = params.verbose.unwrap_or(false);

        let mut output = String::new();
        output.push_str("# Health Check Report\n\n");

        // Server status
        output.push_str("## Server Status\n\n");
        output.push_str("- **Status**: Healthy\n");
        output.push_str("- **Version**: 0.1.0\n\n");

        // Cache statistics
        output.push_str("## Cache Statistics\n\n");
        let categories = self.cache.get_all_categories().map_err(|e| {
            rust_mcp_sdk::schema::CallToolError::from_message(format!("Cache error: {}", e))
        })?;
        let total_count = self.cache.total_article_count().map_err(|e| {
            rust_mcp_sdk::schema::CallToolError::from_message(format!("Cache error: {}", e))
        })?;

        output.push_str(&format!("- **Total Articles**: {}\n", total_count));
        output.push_str("- **Categories**:\n");

        for (category, count) in categories {
            if verbose || count > 0 {
                output.push_str(&format!(
                    "  - {}: {} articles\n",
                    category.display_name(),
                    count
                ));
            }
        }
        output.push('\n');

        // Last update times
        if verbose {
            output.push_str("## Last Update Times\n\n");
            for category in crate::cache::NewsCategory::all() {
                if let Some(last_updated) = self.cache.get_last_updated(&category).map_err(|e| {
                    rust_mcp_sdk::schema::CallToolError::from_message(format!("Cache error: {}", e))
                })? {
                    output.push_str(&format!(
                        "- {}: {}\n",
                        category.display_name(),
                        last_updated.format("%Y-%m-%d %H:%M UTC")
                    ));
                }
            }
            output.push('\n');
        }

        Ok(rust_mcp_sdk::schema::CallToolResult::text_content(vec![
            output.into(),
        ]))
    }
}
