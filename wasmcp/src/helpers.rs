//! Helper functions for MCP protocol handling

use crate::types::*;
use crate::traits::McpHandler;
use serde_json::Value;
use spin_sdk::http::Response;
use std::io::Write;

// Simple file-based logging function for the SDK
fn log(msg: &str) {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    
    let log_msg = format!("[{}] WASMCP: {}\n", timestamp, msg);
    
    // Try to append to log file, ignore errors to avoid breaking the main flow
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/wasmcp.log") 
    {
        let _ = file.write_all(log_msg.as_bytes());
        let _ = file.flush();
    }
}

/// Parse a JSON-RPC request from bytes
pub fn parse_jsonrpc_request(body: &[u8]) -> Result<JsonRpcRequest, JsonRpcError> {
    log(&format!("Parsing JSON-RPC request, body length: {}", body.len()));
    log(&format!("Body content: {:?}", std::str::from_utf8(body).unwrap_or("<invalid utf8>")));
    
    match serde_json::from_slice::<JsonRpcRequest>(body) {
        Ok(req) => {
            log(&format!("Successfully parsed request: method={}, id={:?}", req.method, req.id));
            Ok(req)
        }
        Err(e) => {
            log(&format!("Failed to parse JSON-RPC request: {}", e));
            Err(JsonRpcError::parse_error())
        }
    }
}

/// Build a JSON-RPC response
pub fn build_jsonrpc_response(id: Option<JsonRpcId>, result: Value) -> JsonRpcResponse {
    log(&format!("Building JSON-RPC response with id={:?}", id));
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(result),
        error: None,
        id,
    };
    log("Response built successfully");
    response
}

/// Build a JSON-RPC error response
pub fn build_jsonrpc_error(id: Option<JsonRpcId>, error: JsonRpcError) -> JsonRpcResponse {
    log(&format!("Building JSON-RPC error response: {}", error.message));
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: None,
        error: Some(error),
        id,
    }
}

/// Convert a JSON-RPC response to an HTTP response
pub fn jsonrpc_to_http_response(response: JsonRpcResponse) -> Response {
    log("Converting JSON-RPC response to HTTP");
    let json_bytes = match serde_json::to_vec(&response) {
        Ok(bytes) => {
            log(&format!("Serialized response: {} bytes", bytes.len()));
            bytes
        }
        Err(e) => {
            log(&format!("Failed to serialize response: {}", e));
            panic!("Failed to serialize JSON-RPC response: {}", e);
        }
    };
    
    let http_response = Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(json_bytes)
        .build();
    
    log("HTTP response built successfully");
    http_response
}

/// Handle a JSON-RPC request using an McpHandler
pub async fn handle_jsonrpc_request<H: McpHandler>(
    handler: &H,
    request: JsonRpcRequest,
) -> JsonRpcResponse {
    let id = request.id.clone();
    
    let result = match request.method.as_str() {
        "initialize" => {
            match serde_json::from_value::<InitializeParams>(request.params.unwrap_or_default()) {
                Ok(params) => handler.handle_initialize(params).await
                    .map(|r| serde_json::to_value(r).unwrap()),
                Err(_) => Err(JsonRpcError::invalid_params("Invalid initialize parameters")),
            }
        }
        "ping" => handler.handle_ping().await,
        "tools/list" => {
            handler.list_tools().await
                .map(|tools| serde_json::json!({ "tools": tools }))
        }
        "tools/call" => {
            match serde_json::from_value::<ToolCallParams>(request.params.unwrap_or_default()) {
                Ok(params) => handler.call_tool(&params.name, params.arguments).await
                    .map(|r| serde_json::to_value(r).unwrap()),
                Err(_) => Err(JsonRpcError::invalid_params("Invalid tool call parameters")),
            }
        }
        "resources/list" => {
            handler.list_resources().await
                .map(|resources| serde_json::to_value(ResourcesListResult { items: resources }).unwrap())
        }
        "resources/read" => {
            match serde_json::from_value::<ResourceReadParams>(request.params.unwrap_or_default()) {
                Ok(params) => handler.read_resource(&params.uri).await
                    .map(|r| serde_json::to_value(r).unwrap()),
                Err(_) => Err(JsonRpcError::invalid_params("Invalid resource read parameters")),
            }
        }
        "prompts/list" => {
            handler.list_prompts().await
                .map(|prompts| serde_json::to_value(PromptsListResult { items: prompts }).unwrap())
        }
        "prompts/get" => {
            match serde_json::from_value::<PromptGetParams>(request.params.unwrap_or_default()) {
                Ok(params) => handler.get_prompt(&params.name, params.arguments).await
                    .map(|messages| serde_json::to_value(messages).unwrap()),
                Err(_) => Err(JsonRpcError::invalid_params("Invalid prompt get parameters")),
            }
        }
        _ => Err(JsonRpcError::method_not_found(&request.method)),
    };

    match result {
        Ok(value) => build_jsonrpc_response(id, value),
        Err(error) => build_jsonrpc_error(id, error),
    }
}

/// Extract a specific type from JSON-RPC params
pub fn extract_params<T: serde::de::DeserializeOwned>(
    params: Option<Value>,
) -> Result<T, JsonRpcError> {
    serde_json::from_value(params.unwrap_or_default())
        .map_err(|_| JsonRpcError::invalid_params("Invalid parameters"))
}

/// Check if a protocol version is supported
pub fn is_protocol_version_supported(version: &str) -> bool {
    SUPPORTED_PROTOCOL_VERSIONS.contains(&version)
}

/// Get metadata from Spin component (placeholder for future implementation)
pub fn extract_mcp_metadata() -> Option<ComponentMetadata> {
    // TODO: This will be implemented to read from Spin's component metadata
    // For now, return None
    None
}

/// Component metadata structure
#[derive(Debug, Clone)]
pub struct ComponentMetadata {
    pub mcp_route: String,
    pub mcp_tools: Vec<String>,
    pub mcp_resources: Vec<String>,
    pub mcp_prompts: Vec<String>,
}