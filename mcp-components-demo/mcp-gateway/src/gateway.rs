use crate::mcp_types::*;
use serde::{Deserialize, Serialize};
use spin_sdk::http::{Method, Request, Response};

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

    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request),
            "tools/list" => self.handle_list_tools(request).await,
            "tools/call" => self.handle_call_tool(request).await,
            _ => JsonRpcResponse::error(
                request.id,
                ErrorCode::METHOD_NOT_FOUND.0,
                &format!("Method {} not found", request.method),
            ),
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
            },
            server_info: self.config.server_info.clone(),
        };

        JsonRpcResponse::success(request.id, serde_json::to_value(response).unwrap())
    }

    async fn handle_list_tools(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        // Call the tools-list component to get available tools
        let tools_list_url = "http://tools-list.spin.internal/";
        
        let req = Request::builder()
            .method(Method::Get)
            .uri(tools_list_url)
            .build();

        match spin_sdk::http::send::<_, spin_sdk::http::Response>(req).await {
            Ok(resp) => {
                if *resp.status() != 200 {
                    return JsonRpcResponse::error(
                        request.id,
                        ErrorCode::INTERNAL_ERROR.0,
                        "Failed to fetch tools list",
                    );
                }

                match serde_json::from_slice::<ListToolsResponse>(resp.body()) {
                    Ok(tools_response) => {
                        JsonRpcResponse::success(request.id, serde_json::to_value(tools_response).unwrap())
                    }
                    Err(e) => JsonRpcResponse::error(
                        request.id,
                        ErrorCode::INTERNAL_ERROR.0,
                        &format!("Invalid tools list response: {}", e),
                    ),
                }
            }
            Err(e) => JsonRpcResponse::error(
                request.id,
                ErrorCode::INTERNAL_ERROR.0,
                &format!("Failed to fetch tools list: {}", e),
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
                    // Success - wrap the response in MCP format
                    match serde_json::from_slice::<serde_json::Value>(body) {
                        Ok(result) => {
                            // Wrap the result in MCP tool response format
                            let tool_response = CallToolResponse {
                                content: vec![ToolContent::Text {
                                    text: serde_json::to_string_pretty(&result).unwrap_or(result.to_string()),
                                }],
                            };
                            JsonRpcResponse::success(request.id, serde_json::to_value(tool_response).unwrap())
                        }
                        Err(e) => JsonRpcResponse::error(
                            request.id,
                            ErrorCode::INTERNAL_ERROR.0,
                            &format!("Invalid tool response: {}", e),
                        ),
                    }
                } else {
                    // Error - check if we have a structured error response
                    match serde_json::from_slice::<ToolError>(body) {
                        Ok(tool_error) => JsonRpcResponse::error(
                            request.id,
                            ErrorCode::INTERNAL_ERROR.0,
                            &tool_error.error,
                        ),
                        Err(_) => {
                            // Fallback to generic error
                            let error_text = String::from_utf8_lossy(body);
                            JsonRpcResponse::error(
                                request.id,
                                ErrorCode::INTERNAL_ERROR.0,
                                &format!("Tool execution failed (status {}): {}", status, error_text),
                            )
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
    let response = gateway.handle_request(request).await;

    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(serde_json::to_vec(&response).unwrap())
        .build()
}