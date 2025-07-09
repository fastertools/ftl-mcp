use crate::mcp_types::*;
use serde::{Deserialize, Serialize};
use spin_sdk::http::{Method, Request, Response};
use spin_sdk::variables;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub server_info: ServerInfo,
}

pub struct McpGateway {
    config: GatewayConfig,
}

impl McpGateway {
    pub fn new(config: GatewayConfig) -> Self {
        Self { config }
    }

    pub async fn handle_request(&self, request: JsonRpcRequest) -> Option<JsonRpcResponse> {
        match request.method.as_str() {
            "initialize" => Some(self.handle_initialize(request)),
            "initialized" => {
                // This is a notification, no response needed
                None
            },
            "tools/list" => Some(self.handle_list_tools(request).await),
            "tools/call" => Some(self.handle_call_tool(request).await),
            "ping" => Some(self.handle_ping(request)),
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
                        &format!("Invalid initialize parameters: {}", e),
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
                 capabilities and annotations.".to_string()
            ),
        };

        JsonRpcResponse::success(request.id, serde_json::to_value(response).unwrap())
    }

    async fn handle_list_tools(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        // Get the list of tool components from the spin variable
        let tool_components = match variables::get("tool_components") {
            Ok(components) => components,
            Err(e) => {
                return JsonRpcResponse::error(
                    request.id,
                    ErrorCode::INTERNAL_ERROR.0,
                    &format!("Failed to get tool components configuration: {}", e),
                );
            }
        };

        // Parse the comma-separated list of tool names
        let tool_names: Vec<&str> = tool_components.split(',').map(|s| s.trim()).collect();
        let mut tools = Vec::new();

        // Fetch metadata from each tool component
        for tool_name in tool_names {
            let tool_url = format!("http://{}.spin.internal/", tool_name);
            
            let req = Request::builder()
                .method(Method::Get)
                .uri(&tool_url)
                .build();

            match spin_sdk::http::send::<_, spin_sdk::http::Response>(req).await {
                Ok(resp) => {
                    if *resp.status() == 200 {
                        match serde_json::from_slice::<Tool>(resp.body()) {
                            Ok(tool) => tools.push(tool),
                            Err(e) => {
                                eprintln!("Failed to parse metadata from tool '{}': {}", tool_name, e);
                                // Continue with other tools even if one fails
                            }
                        }
                    } else {
                        eprintln!("Tool '{}' returned status {} for metadata request", tool_name, resp.status());
                    }
                }
                Err(e) => {
                    eprintln!("Failed to fetch metadata from tool '{}': {}", tool_name, e);
                    // Continue with other tools even if one fails
                }
            }
        }

        let response = ListToolsResponse { tools };
        JsonRpcResponse::success(request.id, serde_json::to_value(response).unwrap())
    }

    async fn handle_call_tool(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let params: CallToolRequest = match request.params {
            Some(p) => match serde_json::from_value(p) {
                Ok(params) => params,
                Err(e) => {
                    return JsonRpcResponse::error(
                        request.id,
                        ErrorCode::INVALID_PARAMS.0,
                        &format!("Invalid call tool parameters: {}", e),
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

        // Call the specific tool component
        let tool_url = format!("http://{}.spin.internal/", params.name);
        
        // Prepare the request body with just the arguments
        let tool_request_body = params.arguments.unwrap_or(serde_json::json!({}));
        
        let req = Request::builder()
            .method(Method::Post)
            .uri(&tool_url)
            .header("Content-Type", "application/json")
            .body(serde_json::to_vec(&tool_request_body).unwrap())
            .build();

        match spin_sdk::http::send::<_, spin_sdk::http::Response>(req).await {
            Ok(resp) => {
                let status = resp.status();
                let body = resp.body();

                if *status == 200 {
                    // Success - check if tool returned MCP-formatted response or raw JSON
                    match serde_json::from_slice::<CallToolResponse>(body) {
                        Ok(tool_response) => {
                            // Tool returned proper MCP response format
                            JsonRpcResponse::success(request.id, serde_json::to_value(tool_response).unwrap())
                        }
                        Err(_) => {
                            // Check if tool returned simple content array format
                            #[derive(Deserialize)]
                            struct SimpleResponse {
                                content: Vec<serde_json::Value>,
                            }
                            
                            match serde_json::from_slice::<SimpleResponse>(body) {
                                Ok(simple_response) => {
                                    // Convert simple response to MCP format
                                    let tool_response = CallToolResponse {
                                        content: simple_response.content.into_iter().filter_map(|v| {
                                            serde_json::from_value::<ToolContent>(v).ok()
                                        }).collect(),
                                        structured_content: None,
                                        is_error: None,
                                    };
                                    JsonRpcResponse::success(request.id, serde_json::to_value(tool_response).unwrap())
                                }
                                Err(_) => {
                                    // Tool returned raw JSON - wrap it in MCP format
                                    match serde_json::from_slice::<serde_json::Value>(body) {
                                        Ok(result) => {
                                            let tool_response = CallToolResponse {
                                                content: vec![ToolContent::Text {
                                                    text: serde_json::to_string_pretty(&result).unwrap_or(result.to_string()),
                                                    annotations: None,
                                                }],
                                                structured_content: None,
                                                is_error: None,
                                            };
                                            JsonRpcResponse::success(request.id, serde_json::to_value(tool_response).unwrap())
                                        }
                                        Err(e) => JsonRpcResponse::error(
                                            request.id,
                                            ErrorCode::INTERNAL_ERROR.0,
                                            &format!("Invalid tool response: {}", e),
                                        ),
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Error - check if we have an MCP-formatted error response
                    match serde_json::from_slice::<CallToolResponse>(body) {
                        Ok(tool_response) if tool_response.is_error.unwrap_or(false) => {
                            // Tool returned error in MCP format
                            JsonRpcResponse::success(request.id, serde_json::to_value(tool_response).unwrap())
                        }
                        _ => {
                            // Check for legacy error format or plain text
                            match serde_json::from_slice::<ToolError>(body) {
                                Ok(tool_error) => {
                                    // Convert legacy error to MCP format
                                    let tool_response = CallToolResponse {
                                        content: vec![ToolContent::Text {
                                            text: tool_error.error.clone(),
                                            annotations: None,
                                        }],
                                        structured_content: None,
                                        is_error: Some(true),
                                    };
                                    JsonRpcResponse::success(request.id, serde_json::to_value(tool_response).unwrap())
                                }
                                Err(_) => {
                                    // Fallback to wrapping plain text error
                                    let error_text = String::from_utf8_lossy(body);
                                    let tool_response = CallToolResponse {
                                        content: vec![ToolContent::Text {
                                            text: format!("Tool execution failed (status {}): {}", status, error_text),
                                            annotations: None,
                                        }],
                                        structured_content: None,
                                        is_error: Some(true),
                                    };
                                    JsonRpcResponse::success(request.id, serde_json::to_value(tool_response).unwrap())
                                }
                            }
                        }
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
    
    fn handle_ping(&self, request: JsonRpcRequest) -> JsonRpcResponse {
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
                &format!("Invalid JSON-RPC request: {}", e),
            );
            return Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(serde_json::to_vec(&error_response).unwrap())
                .build();
        }
    };

    // Create gateway with config
    let config = GatewayConfig {
        server_info: ServerInfo {
            name: "mcp-gateway".to_string(),
            version: "0.1.0".to_string(),
        },
    };
    let gateway = McpGateway::new(config);

    // Handle the request
    match gateway.handle_request(request).await {
        Some(response) => {
            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(serde_json::to_vec(&response).unwrap())
                .build()
        }
        None => {
            // Notification - return empty response
            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(Vec::new())
                .build()
        }
    }
}