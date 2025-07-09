//! Error types and utilities for MCP

use crate::types::JsonRpcError;

/// Standard JSON-RPC error codes
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
}

/// MCP-specific error codes (custom range)
pub mod mcp_error_codes {
    pub const UNSUPPORTED_PROTOCOL_VERSION: i32 = -32001;
    pub const TOOL_NOT_FOUND: i32 = -32002;
    pub const RESOURCE_NOT_FOUND: i32 = -32003;
    pub const PROMPT_NOT_FOUND: i32 = -32004;
    pub const EXTERNAL_API_ERROR: i32 = -32005;
}

/// Helper functions for creating common errors
impl JsonRpcError {
    /// Create a parse error
    pub fn parse_error() -> Self {
        Self {
            code: error_codes::PARSE_ERROR,
            message: "Parse error".to_string(),
            data: None,
        }
    }

    /// Create an invalid request error
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self {
            code: error_codes::INVALID_REQUEST,
            message: message.into(),
            data: None,
        }
    }

    /// Create a method not found error
    pub fn method_not_found(method: impl Into<String>) -> Self {
        let method = method.into();
        Self {
            code: error_codes::METHOD_NOT_FOUND,
            message: format!("Method not found: {}", method),
            data: Some(serde_json::json!({ "method": method })),
        }
    }

    /// Create an invalid params error
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self {
            code: error_codes::INVALID_PARAMS,
            message: message.into(),
            data: None,
        }
    }

    /// Create an internal error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self {
            code: error_codes::INTERNAL_ERROR,
            message: message.into(),
            data: None,
        }
    }

    /// Create an unsupported protocol version error
    pub fn unsupported_protocol_version(version: impl Into<String>) -> Self {
        let version = version.into();
        Self {
            code: mcp_error_codes::UNSUPPORTED_PROTOCOL_VERSION,
            message: format!("Unsupported protocol version: {}", version),
            data: Some(serde_json::json!({ 
                "requestedVersion": version,
                "supportedVersions": crate::types::SUPPORTED_PROTOCOL_VERSIONS
            })),
        }
    }

    /// Create a tool not found error
    pub fn tool_not_found(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            code: mcp_error_codes::TOOL_NOT_FOUND,
            message: format!("Tool not found: {}", name),
            data: Some(serde_json::json!({ "tool": name })),
        }
    }

    /// Create a resource not found error
    pub fn resource_not_found(uri: impl Into<String>) -> Self {
        let uri = uri.into();
        Self {
            code: mcp_error_codes::RESOURCE_NOT_FOUND,
            message: format!("Resource not found: {}", uri),
            data: Some(serde_json::json!({ "uri": uri })),
        }
    }

    /// Create a prompt not found error
    pub fn prompt_not_found(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            code: mcp_error_codes::PROMPT_NOT_FOUND,
            message: format!("Prompt not found: {}", name),
            data: Some(serde_json::json!({ "prompt": name })),
        }
    }

    /// Create an external API error
    pub fn external_api_error(message: impl Into<String>) -> Self {
        Self {
            code: mcp_error_codes::EXTERNAL_API_ERROR,
            message: message.into(),
            data: None,
        }
    }

    /// Add data to an error
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}

/// Result type for MCP operations
pub type McpResult<T> = Result<T, JsonRpcError>;