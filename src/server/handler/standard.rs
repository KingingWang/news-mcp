//! Standard MCP handler implementation
//!
//! Implements the ServerHandler trait for handling MCP requests.

use crate::tools::ToolRegistry;
use async_trait::async_trait;
use rust_mcp_sdk::{
    mcp_server::ServerHandler,
    schema::{
        CallToolError, CallToolRequestParams, CallToolResult, GetPromptRequestParams,
        GetPromptResult, ListPromptsResult, ListResourcesResult, ListToolsResult,
        PaginatedRequestParams, ReadResourceRequestParams, ReadResourceResult, RpcError,
    },
    McpServer,
};
use std::sync::Arc;
use tracing::{debug, info, info_span, Instrument};
use uuid::Uuid;

/// News MCP handler
pub struct NewsMcpHandler {
    /// Tool registry
    tool_registry: Arc<ToolRegistry>,
    /// Server info (reserved for future use)
    _server_info: rust_mcp_sdk::schema::Implementation,
}

impl NewsMcpHandler {
    /// Create a new handler
    pub fn new(server: Arc<crate::server::NewsMcpServer>) -> Self {
        let server_info_result = server.server_info();
        Self {
            tool_registry: server.tool_registry().clone(),
            _server_info: server_info_result.server_info,
        }
    }
}

#[async_trait]
impl ServerHandler for NewsMcpHandler {
    /// Handle list tools request
    async fn handle_list_tools_request(
        &self,
        _request: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ListToolsResult, RpcError> {
        let trace_id = Uuid::new_v4().to_string();
        let span = info_span!("list_tools", trace_id = %trace_id);

        async {
            debug!("Listing available tools");
            let tools = self.tool_registry.get_tools();
            debug!("Found {} tools", tools.len());
            Ok(ListToolsResult {
                tools,
                meta: None,
                next_cursor: None,
            })
        }
        .instrument(span)
        .await
    }

    /// Handle call tool request
    async fn handle_call_tool_request(
        &self,
        params: CallToolRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        let trace_id = Uuid::new_v4().to_string();
        let tool_name = params.name.clone();
        let span = info_span!("execute_tool", trace_id = %trace_id, tool = %tool_name);

        async {
            info!("Executing tool: {}", tool_name);
            let start = std::time::Instant::now();

            let arguments = params
                .arguments
                .map_or_else(|| serde_json::Value::Null, serde_json::Value::Object);

            let result = self
                .tool_registry
                .execute_tool(&tool_name, arguments)
                .await?;

            let duration = start.elapsed();
            info!("Tool {} executed successfully in {:?}", tool_name, duration);

            Ok(result)
        }
        .instrument(span)
        .await
    }

    /// Handle list resources request
    async fn handle_list_resources_request(
        &self,
        _request: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ListResourcesResult, RpcError> {
        Ok(ListResourcesResult {
            resources: vec![],
            meta: None,
            next_cursor: None,
        })
    }

    /// Handle read resource request
    async fn handle_read_resource_request(
        &self,
        _params: ReadResourceRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ReadResourceResult, RpcError> {
        Err(RpcError::invalid_request().with_message("Resource not found".to_string()))
    }

    /// Handle list prompts request
    async fn handle_list_prompts_request(
        &self,
        _request: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ListPromptsResult, RpcError> {
        Ok(ListPromptsResult {
            prompts: vec![],
            meta: None,
            next_cursor: None,
        })
    }

    /// Handle get prompt request
    async fn handle_get_prompt_request(
        &self,
        _params: GetPromptRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<GetPromptResult, RpcError> {
        Err(RpcError::invalid_request().with_message("Prompt not found".to_string()))
    }
}
