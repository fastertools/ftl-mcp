use spin_sdk::http::{IntoResponse, Request};
use spin_sdk::{http_component, variables};
use wasmcp::{
    parse_jsonrpc_request, build_jsonrpc_response, build_jsonrpc_error, 
    jsonrpc_to_http_response, JsonRpcError, JsonRpcRequest, JsonRpcResponse,
    InitializeParams, InitializeResult, Tool, ServerCapabilities, ServerInfo
};
use serde_json::Value;
use std::io::Write;

// Simple file-based logging function that avoids broken pipe issues
fn log(msg: &str) {
    // Try multiple log paths in case /tmp isn't writable
    let log_paths = ["/tmp/wasmcp.log", "./wasmcp.log", "/var/tmp/wasmcp.log"];
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    
    let log_msg = format!("[{}] ROUTER: {}\n", timestamp, msg);
    
    for path in &log_paths {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path) 
        {
            let _ = file.write_all(log_msg.as_bytes());
            let _ = file.flush();
            break; // Stop at first successful write
        }
    }
}

/// Plugin information
#[derive(Clone, Debug)]
struct PluginInfo {
    name: String,
    endpoint: String,
    tools: Vec<String>,
}

/// Get the list of registered plugins from Spin variables
fn get_plugins() -> Vec<PluginInfo> {
    println!("ROUTER: Loading plugins from Spin variables");
    let mut plugins = Vec::new();
    
    // Read weather plugin configuration from Spin variables
    println!("ROUTER: Attempting to read weather plugin variables");
    let weather_name = variables::get("weather_plugin_name");
    let weather_endpoint = variables::get("weather_plugin_endpoint");
    let weather_tools = variables::get("weather_plugin_tools");
    
    if let (Ok(name), Ok(endpoint), Ok(tools_str)) = (weather_name, weather_endpoint, weather_tools) {
        let tools: Vec<String> = tools_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
            
        plugins.push(PluginInfo {
            name: name.clone(),
            endpoint,
            tools: tools.clone(),
        });
        
        log(&format!("Loaded weather plugin: {} with {} tools", name, tools.len()));
    } else {
        log("No weather plugin variables configured");
    }
    
    // Read activity plugin configuration from Spin variables
    println!("ROUTER: Attempting to read activity plugin variables");
    let activity_name = variables::get("activity_plugin_name");
    let activity_endpoint = variables::get("activity_plugin_endpoint");
    let activity_tools = variables::get("activity_plugin_tools");
    
    if let (Ok(name), Ok(endpoint), Ok(tools_str)) = (activity_name, activity_endpoint, activity_tools) {
        let tools: Vec<String> = tools_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
            
        plugins.push(PluginInfo {
            name: name.clone(),
            endpoint,
            tools: tools.clone(),
        });
        
        log(&format!("Loaded activity plugin: {} with {} tools", name, tools.len()));
    } else {
        log("No activity plugin variables configured");
    }
    
    log(&format!("Total plugins loaded: {}", plugins.len()));
    plugins
}

/// Forward a request to a plugin
async fn forward_to_plugin(
    endpoint: &str,
    request: &JsonRpcRequest,
) -> Result<JsonRpcResponse, String> {
    log(&format!("Forwarding request to plugin: {}", endpoint));
    
    let body = serde_json::to_vec(request).map_err(|e| e.to_string())?;
    let req = spin_sdk::http::Request::builder()
        .method(spin_sdk::http::Method::Post)
        .uri(endpoint)
        .header("Content-Type", "application/json")
        .body(body)
        .build();
    
    let response: spin_sdk::http::Response = match spin_sdk::http::send(req).await
    {
        Ok(resp) => resp,
        Err(e) => return Err(format!("Failed to forward request: {:?}", e)),
    };
    
    let status = response.status();
    let body = response.body();
    
    if *status != 200 {
        return Err(format!("Plugin returned status {}: {:?}", status, std::str::from_utf8(&body)));
    }
    
    serde_json::from_slice(&body)
        .map_err(|e| format!("Failed to parse plugin response: {}", e))
}

/// MCP Router component
#[http_component]
async fn handle_mcp_router(req: Request) -> anyhow::Result<impl IntoResponse> {
    log("ROUTER: Component started - handling request");
    log(&format!("Received request to: {}", req.path()));
    log(&format!("Method: {}", req.method()));
    
    // Parse the request body
    let body = req.body();
    log(&format!("Body length: {} bytes", body.len()));
    
    let request = match parse_jsonrpc_request(body) {
        Ok(req) => {
            log("Successfully parsed request");
            req
        },
        Err(e) => {
            log("Failed to parse request, returning error");
            return Ok(jsonrpc_to_http_response(build_jsonrpc_error(None, e)));
        }
    };
    
    log(&format!("Processing method: {}", request.method));
    
    // Handle the request based on method
    let response = match request.method.as_str() {
        "initialize" => {
            log("Handling initialize request");
            
            // Parse params
            let params: InitializeParams = match request.params {
                Some(ref p) => match serde_json::from_value(p.clone()) {
                    Ok(params) => params,
                    Err(e) => {
                        return Ok(jsonrpc_to_http_response(build_jsonrpc_error(
                            request.id,
                            JsonRpcError {
                                code: -32602,
                                message: format!("Invalid params: {}", e),
                                data: None,
                            }
                        )));
                    }
                },
                None => {
                    return Ok(jsonrpc_to_http_response(build_jsonrpc_error(
                        request.id,
                        JsonRpcError {
                            code: -32602,
                            message: "Missing params".to_string(),
                            data: None,
                        }
                    )));
                }
            };
            
            log(&format!("Protocol version: {}", params.protocol_version));
            
            // Initialize all plugins
            let plugins = get_plugins();
            for plugin in &plugins {
                log(&format!("Initializing plugin: {}", plugin.name));
                match forward_to_plugin(&plugin.endpoint, &request).await {
                    Ok(_) => log(&format!("Plugin {} initialized successfully", plugin.name)),
                    Err(e) => log(&format!("Failed to initialize plugin {}: {}", plugin.name, e)),
                }
            }
            
            // Return router's initialize response
            let result = InitializeResult {
                protocol_version: params.protocol_version.clone(),
                capabilities: ServerCapabilities {
                    tools: Some(serde_json::json!({})),
                    resources: Some(serde_json::json!({})),
                    prompts: Some(serde_json::json!({})),
                    logging: None,
                },
                server_info: ServerInfo {
                    name: "mcp-router".to_string(),
                    version: "0.1.0".to_string(),
                },
            };
            
            build_jsonrpc_response(
                request.id,
                serde_json::to_value(result).unwrap()
            )
        },
        
        "ping" => {
            log("Handling ping request");
            build_jsonrpc_response(
                request.id,
                serde_json::json!({})
            )
        },
        
        "tools/list" => {
            log("Handling tools/list request");
            
            let mut all_tools = Vec::new();
            let plugins = get_plugins();
            
            // Collect tools from all plugins
            for plugin in plugins {
                log(&format!("Querying tools from plugin: {}", plugin.name));
                match forward_to_plugin(&plugin.endpoint, &request).await {
                    Ok(response) => {
                        if let Some(result) = response.result {
                            if let Ok(tools_response) = serde_json::from_value::<serde_json::Value>(result) {
                                if let Some(tools) = tools_response.get("tools") {
                                    if let Ok(tools_vec) = serde_json::from_value::<Vec<Tool>>(tools.clone()) {
                                        log(&format!("Plugin {} returned {} tools", plugin.name, tools_vec.len()));
                                        all_tools.extend(tools_vec);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log(&format!("Failed to get tools from plugin {}: {}", plugin.name, e));
                    }
                }
            }
            
            build_jsonrpc_response(
                request.id,
                serde_json::json!({
                    "tools": all_tools
                })
            )
        },
        
        "tools/call" => {
            log("Handling tools/call request");
            
            // Extract tool name from params
            let tool_name = match request.params.as_ref() {
                Some(params) => match params.get("name") {
                    Some(Value::String(name)) => name.clone(),
                    _ => {
                        return Ok(jsonrpc_to_http_response(build_jsonrpc_error(
                            request.id,
                            JsonRpcError {
                                code: -32602,
                                message: "Missing or invalid tool name".to_string(),
                                data: None,
                            }
                        )));
                    }
                },
                None => {
                    return Ok(jsonrpc_to_http_response(build_jsonrpc_error(
                        request.id,
                        JsonRpcError {
                            code: -32602,
                            message: "Missing params".to_string(),
                            data: None,
                        }
                    )));
                }
            };
            
            log(&format!("Looking for plugin to handle tool: {}", tool_name));
            
            // Find the plugin that handles this tool
            let plugins = get_plugins();
            let plugin = plugins.iter().find(|p| p.tools.contains(&tool_name));
            
            match plugin {
                Some(p) => {
                    log(&format!("Forwarding to plugin: {}", p.name));
                    match forward_to_plugin(&p.endpoint, &request).await {
                        Ok(response) => response,
                        Err(e) => build_jsonrpc_error(
                            request.id,
                            JsonRpcError {
                                code: -32603,
                                message: format!("Plugin error: {}", e),
                                data: None,
                            }
                        )
                    }
                },
                None => {
                    log(&format!("No plugin found for tool: {}", tool_name));
                    build_jsonrpc_error(
                        request.id,
                        JsonRpcError {
                            code: -32601,
                            message: format!("Unknown tool: {}", tool_name),
                            data: None,
                        }
                    )
                }
            }
        },
        
        method => {
            log(&format!("Unknown method: {}", method));
            build_jsonrpc_error(
                request.id,
                JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {}", method),
                    data: None,
                }
            )
        }
    };
    
    log("Returning response");
    Ok(jsonrpc_to_http_response(response))
}