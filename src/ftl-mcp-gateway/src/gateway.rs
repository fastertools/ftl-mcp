use serde::{Deserialize, Serialize};
use spin_sdk::http::{Method, Request, Response};
use spin_sdk::variables;

use crate::mcp_types::{
    CallToolRequest, ErrorCode, InitializeRequest, InitializeResponse, JsonRpcRequest,
    JsonRpcResponse, ListToolsResponse, McpProtocolVersion, ServerCapabilities, ServerInfo,
    ToolContent, ToolMetadata, ToolResponse,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub server_info: ServerInfo,
    #[serde(default = "default_validate_arguments")]
    pub validate_arguments: bool,
}

fn default_validate_arguments() -> bool {
    true
}

pub struct McpGateway {
    config: GatewayConfig,
}

impl McpGateway {
    pub fn new(config: GatewayConfig) -> Self {
        Self { config }
    }

    /// Convert `snake_case` to kebab-case for component names
    fn snake_to_kebab(name: &str) -> String {
        name.replace('_', "-")
    }

    /// Fetch metadata for a specific tool
    async fn fetch_tool_metadata(&self, tool_name: &str) -> Option<ToolMetadata> {
        let component_name = Self::snake_to_kebab(tool_name);
        let tool_url = format!("http://{component_name}.spin.internal/");

        let req = Request::builder()
            .method(Method::Get)
            .uri(&tool_url)
            .build();

        match spin_sdk::http::send::<_, spin_sdk::http::Response>(req).await {
            Ok(resp) => {
                if *resp.status() == 200 {
                    match serde_json::from_slice::<ToolMetadata>(resp.body()) {
                        Ok(tool) => Some(tool),
                        Err(e) => {
                            eprintln!("Failed to parse metadata from tool '{tool_name}': {e}");
                            None
                        }
                    }
                } else {
                    eprintln!(
                        "Tool '{}' returned status {} for metadata request",
                        tool_name,
                        resp.status()
                    );
                    None
                }
            }
            Err(e) => {
                eprintln!("Failed to fetch metadata from tool '{tool_name}': {e}");
                None
            }
        }
    }

    /// Validate tool arguments against the tool's input schema
    fn validate_arguments(
        tool_name: &str,
        schema: &serde_json::Value,
        arguments: &serde_json::Value,
    ) -> Result<(), String> {
        match jsonschema::validator_for(schema) {
            Ok(validator) => {
                // Use iter_errors which returns an iterator
                let errors: Vec<jsonschema::ValidationError<'_>> =
                    validator.iter_errors(arguments).collect();
                if errors.is_empty() {
                    Ok(())
                } else {
                    let error_messages: Vec<String> = errors
                        .iter()
                        .map(|error| {
                            format!("Validation error at {}: {}", error.instance_path, error)
                        })
                        .collect();
                    Err(format!(
                        "Invalid arguments for tool '{}': {}",
                        tool_name,
                        error_messages.join("; ")
                    ))
                }
            }
            Err(e) => Err(format!(
                "Failed to compile schema for tool '{tool_name}': {e}"
            )),
        }
    }

    pub async fn handle_request(&self, request: JsonRpcRequest) -> Option<JsonRpcResponse> {
        match request.method.as_str() {
            "initialize" => Some(self.handle_initialize(request)),
            "initialized" => {
                // This is a notification, no response needed
                None
            }
            "tools/list" => Some(self.handle_list_tools(request).await),
            "tools/call" => Some(self.handle_call_tool(request).await),
            "ping" => Some(Self::handle_ping(self, request)),
            _ => Some(JsonRpcResponse::error(
                request.id,
                ErrorCode::METHOD_NOT_FOUND.0,
                &format!("Method '{}' not found", request.method),
            )),
        }
    }

    fn handle_initialize(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let params: InitializeRequest = match request.params {
            Some(p) => match serde_json::from_value(p) {
                Ok(params) => params,
                Err(e) => {
                    return JsonRpcResponse::error(
                        request.id,
                        ErrorCode::INVALID_PARAMS.0,
                        &format!("Invalid initialize parameters: {e}"),
                    );
                }
            },
            None => {
                return JsonRpcResponse::error(
                    request.id,
                    ErrorCode::INVALID_PARAMS.0,
                    "Missing initialize parameters",
                );
            }
        };

        if params.protocol_version != McpProtocolVersion::V1 {
            return JsonRpcResponse::error(
                request.id,
                ErrorCode::INVALID_REQUEST.0,
                "Unsupported protocol version",
            );
        }

        let response = InitializeResponse {
            protocol_version: McpProtocolVersion::V1,
            capabilities: ServerCapabilities {
                tools: Some(serde_json::json!({})),
                resources: Some(serde_json::json!({})),
                prompts: Some(serde_json::json!({})),
            },
            server_info: self.config.server_info.clone(),
            instructions: Some(
                "This MCP server provides access to tools via WebAssembly components. \
                 Each tool is implemented as an independent component with its own \
                 capabilities and annotations."
                    .to_string(),
            ),
        };

        match serde_json::to_value(response) {
            Ok(value) => JsonRpcResponse::success(request.id, value),
            Err(e) => JsonRpcResponse::error(
                request.id,
                ErrorCode::INTERNAL_ERROR.0,
                &format!("Failed to serialize response: {e}"),
            ),
        }
    }

    async fn handle_list_tools(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        // Get the list of tool components from the spin variable
        let tool_components = match variables::get("tool_components") {
            Ok(components) => components,
            Err(e) => {
                return JsonRpcResponse::error(
                    request.id,
                    ErrorCode::INTERNAL_ERROR.0,
                    &format!("Failed to get tool components configuration: {e}"),
                );
            }
        };

        // Parse the comma-separated list of tool names
        let tool_names: Vec<&str> = tool_components.split(',').map(str::trim).collect();

        // Create futures for fetching metadata from all tools in parallel
        let metadata_futures: Vec<_> = tool_names
            .iter()
            .map(|tool_name| self.fetch_tool_metadata(tool_name))
            .collect();

        // Execute all futures concurrently and collect results
        let results = futures::future::join_all(metadata_futures).await;

        // Filter out None values and collect successful tool metadata
        let tools: Vec<ToolMetadata> = results.into_iter().flatten().collect();

        let response = ListToolsResponse { tools };
        match serde_json::to_value(response) {
            Ok(value) => JsonRpcResponse::success(request.id, value),
            Err(e) => JsonRpcResponse::error(
                request.id,
                ErrorCode::INTERNAL_ERROR.0,
                &format!("Failed to serialize response: {e}"),
            ),
        }
    }

    async fn handle_call_tool(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let params: CallToolRequest = match request.params {
            Some(p) => match serde_json::from_value(p) {
                Ok(params) => params,
                Err(e) => {
                    return JsonRpcResponse::error(
                        request.id,
                        ErrorCode::INVALID_PARAMS.0,
                        &format!("Invalid call tool parameters: {e}"),
                    );
                }
            },
            None => {
                return JsonRpcResponse::error(
                    request.id,
                    ErrorCode::INVALID_PARAMS.0,
                    "Missing call tool parameters",
                );
            }
        };

        // Validate arguments if validation is enabled
        let tool_arguments = params.arguments.unwrap_or_else(|| serde_json::json!({}));

        if self.config.validate_arguments {
            // Fetch tool metadata for validation
            if let Some(tool_metadata) = self.fetch_tool_metadata(&params.name).await {
                // Validate arguments against the tool's input schema
                if let Err(validation_error) = Self::validate_arguments(
                    &params.name,
                    &tool_metadata.input_schema,
                    &tool_arguments,
                ) {
                    return JsonRpcResponse::error(
                        request.id,
                        ErrorCode::INVALID_PARAMS.0,
                        &validation_error,
                    );
                }
            } else {
                // Tool metadata not available - log but continue
                // This allows the tool itself to handle validation
                eprintln!(
                    "Warning: Could not fetch metadata for tool '{}', skipping validation",
                    params.name
                );
            }
        }

        // Call the specific tool component
        // Convert snake_case to kebab-case for component names
        let component_name = Self::snake_to_kebab(&params.name);
        let tool_url = format!("http://{component_name}.spin.internal/");

        // Prepare the request body with just the arguments
        let tool_request_body = tool_arguments;

        let req = Request::builder()
            .method(Method::Post)
            .uri(&tool_url)
            .header("Content-Type", "application/json")
            .body(
                serde_json::to_vec(&tool_request_body)
                    .unwrap_or_else(|_| br#"{"error":"Failed to serialize request"}"#.to_vec()),
            )
            .build();

        match spin_sdk::http::send::<_, spin_sdk::http::Response>(req).await {
            Ok(resp) => {
                let status = resp.status();
                let body = resp.body();

                if *status == 200 {
                    // Success - tool must return MCP-formatted response
                    match serde_json::from_slice::<ToolResponse>(body) {
                        Ok(tool_response) => match serde_json::to_value(tool_response) {
                            Ok(value) => JsonRpcResponse::success(request.id, value),
                            Err(e) => JsonRpcResponse::error(
                                request.id,
                                ErrorCode::INTERNAL_ERROR.0,
                                &format!("Failed to serialize tool response: {e}"),
                            ),
                        },
                        Err(e) => JsonRpcResponse::error(
                            request.id,
                            ErrorCode::INTERNAL_ERROR.0,
                            &format!("Tool returned invalid response format: {e}"),
                        ),
                    }
                } else {
                    // Error response from tool
                    let error_text = String::from_utf8_lossy(body);
                    let tool_response = ToolResponse {
                        content: vec![ToolContent::Text {
                            text: format!("Tool execution failed (status {status}): {error_text}"),
                            annotations: None,
                        }],
                        structured_content: None,
                        is_error: Some(true),
                    };
                    match serde_json::to_value(tool_response) {
                        Ok(value) => JsonRpcResponse::success(request.id, value),
                        Err(e) => JsonRpcResponse::error(
                            request.id,
                            ErrorCode::INTERNAL_ERROR.0,
                            &format!("Failed to serialize tool response: {e}"),
                        ),
                    }
                }
            }
            Err(e) => JsonRpcResponse::error(
                request.id,
                ErrorCode::INTERNAL_ERROR.0,
                &format!("Failed to call tool '{}': {}", params.name, e),
            ),
        }
    }

    fn handle_ping(_gateway: &Self, request: JsonRpcRequest) -> JsonRpcResponse {
        JsonRpcResponse::success(request.id, serde_json::json!({}))
    }
}

pub async fn handle_mcp_request(req: Request) -> Response {
    // Handle CORS preflight
    if *req.method() == Method::Options {
        return Response::builder()
            .status(200)
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "POST, OPTIONS")
            .header("Access-Control-Allow-Headers", "Content-Type")
            .build();
    }

    // Only accept POST requests
    if *req.method() != Method::Post {
        return Response::builder()
            .status(405)
            .header("Allow", "POST, OPTIONS")
            .body("Method not allowed")
            .build();
    }

    // Parse JSON-RPC request
    let request: JsonRpcRequest = match serde_json::from_slice(req.body()) {
        Ok(r) => r,
        Err(e) => {
            let error_response = JsonRpcResponse::error(
                None,
                ErrorCode::PARSE_ERROR.0,
                &format!("Invalid JSON-RPC request: {e}"),
            );
            return Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(serde_json::to_vec(&error_response).unwrap_or_else(|_| {
                    br#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal serialization error"}}"#.to_vec()
                }))
                .build();
        }
    };

    // Create gateway with config
    let validate_arguments = variables::get("validate_arguments")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);

    let config = GatewayConfig {
        server_info: ServerInfo {
            name: "ftl-mcp-gateway".to_string(),
            version: "0.0.3".to_string(),
        },
        validate_arguments,
    };
    let gateway = McpGateway::new(config);

    // Handle the request
    gateway.handle_request(request).await.map_or_else(
        || {
            // Notification - return empty response
            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(Vec::new())
                .build()
        },
        |response| {
            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(serde_json::to_vec(&response).unwrap_or_else(|_| {
                    br#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal serialization error"}}"#.to_vec()
                }))
                .build()
        },
    )
}
