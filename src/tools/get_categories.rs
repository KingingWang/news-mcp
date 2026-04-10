//! get_categories tool implementation
//!
//! Lists available news categories.

use crate::cache::NewsCache;
use crate::tools::Tool;
use async_trait::async_trait;
use rust_mcp_sdk::macros;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Get categories tool parameters
#[macros::mcp_tool(
    name = "get_categories",
    title = "Get Categories",
    description = "Lists available news categories with article counts.",
    destructive_hint = false,
    idempotent_hint = true,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Clone, Deserialize, Serialize, macros::JsonSchema)]
pub struct GetCategoriesTool {
    // No parameters needed
}

/// Get categories tool implementation
pub struct GetCategoriesToolImpl {
    cache: Arc<NewsCache>,
}

impl GetCategoriesToolImpl {
    /// Create a new get_categories tool
    pub fn new(cache: Arc<NewsCache>) -> Self {
        Self { cache }
    }
}

#[async_trait]
impl Tool for GetCategoriesToolImpl {
    fn definition(&self) -> rust_mcp_sdk::schema::Tool {
        GetCategoriesTool::tool()
    }

    async fn execute(
        &self,
        _arguments: serde_json::Value,
    ) -> std::result::Result<
        rust_mcp_sdk::schema::CallToolResult,
        rust_mcp_sdk::schema::CallToolError,
    > {
        let categories = self.cache.get_all_categories().map_err(|e| {
            rust_mcp_sdk::schema::CallToolError::from_message(format!("Cache error: {}", e))
        })?;

        let mut output = String::new();
        output.push_str("# Available News Categories\n\n");

        for (category, count) in categories {
            output.push_str(&format!(
                "- **{}** ({} articles)\n",
                category.display_name(),
                count
            ));
            output.push_str(&format!("  {}\n\n", category.description()));
        }

        Ok(rust_mcp_sdk::schema::CallToolResult::text_content(vec![
            output.into(),
        ]))
    }
}
