//! Traits for implementing MCP handlers

use crate::types::*;
use crate::errors::McpResult;
use async_trait::async_trait;
use serde_json::Value;

/// Main trait for implementing MCP handlers
/// 
/// Plugins should implement this trait to handle MCP protocol methods.
/// Default implementations are provided for all methods that return appropriate errors.
#[async_trait(?Send)]
pub trait McpHandler: Send + Sync {
    /// Get server information
    fn server_info(&self) -> ServerInfo {
        ServerInfo {
            name: "mcp-plugin".to_string(),
            version: "0.1.0".to_string(),
        }
    }

    /// Get server capabilities
    fn capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            tools: if self.has_tools() { Some(Value::Object(Default::default())) } else { None },
            resources: if self.has_resources() { Some(Value::Object(Default::default())) } else { None },
            prompts: if self.has_prompts() { Some(Value::Object(Default::default())) } else { None },
            logging: None,
        }
    }

    /// Check if this handler provides tools
    fn has_tools(&self) -> bool {
        false
    }

    /// Check if this handler provides resources
    fn has_resources(&self) -> bool {
        false
    }

    /// Check if this handler provides prompts
    fn has_prompts(&self) -> bool {
        false
    }

    /// Handle initialize request
    async fn handle_initialize(&self, params: InitializeParams) -> McpResult<InitializeResult> {
        // Check protocol version
        if !SUPPORTED_PROTOCOL_VERSIONS.contains(&params.protocol_version.as_str()) {
            return Err(JsonRpcError::unsupported_protocol_version(&params.protocol_version));
        }

        Ok(InitializeResult {
            protocol_version: params.protocol_version,
            capabilities: self.capabilities(),
            server_info: self.server_info(),
        })
    }

    /// Handle ping request
    async fn handle_ping(&self) -> McpResult<Value> {
        Ok(Value::Object(Default::default()))
    }

    /// List available tools
    async fn list_tools(&self) -> McpResult<Vec<Tool>> {
        Ok(vec![])
    }

    /// Call a tool
    async fn call_tool(&self, name: &str, _arguments: Option<Value>) -> McpResult<ToolResult> {
        Err(JsonRpcError::tool_not_found(name))
    }

    /// List available resources
    async fn list_resources(&self) -> McpResult<Vec<Resource>> {
        Ok(vec![])
    }

    /// Read a resource
    async fn read_resource(&self, uri: &str) -> McpResult<ResourceContent> {
        Err(JsonRpcError::resource_not_found(uri))
    }

    /// List available prompts
    async fn list_prompts(&self) -> McpResult<Vec<Prompt>> {
        Ok(vec![])
    }

    /// Get a prompt
    async fn get_prompt(&self, name: &str, _arguments: Option<Value>) -> McpResult<Vec<PromptMessage>> {
        Err(JsonRpcError::prompt_not_found(name))
    }
}

/// Trait for components that can provide metadata
pub trait McpMetadata {
    /// Get the MCP route for this component
    fn mcp_route(&self) -> &str;

    /// Get the list of tools this component provides
    fn mcp_tools(&self) -> Vec<String> {
        vec![]
    }

    /// Get the list of resources this component provides
    fn mcp_resources(&self) -> Vec<String> {
        vec![]
    }

    /// Get the list of prompts this component provides
    fn mcp_prompts(&self) -> Vec<String> {
        vec![]
    }
}