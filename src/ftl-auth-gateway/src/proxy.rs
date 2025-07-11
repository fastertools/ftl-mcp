use anyhow::Result;
use serde_json::Value;
use spin_sdk::http::{Request, Response};

use crate::auth::{AuthKitConfig, Claims};

/// Forward authenticated requests to the MCP gateway
pub async fn forward_to_mcp_gateway(
    req: Request,
    config: &AuthKitConfig,
    claims: Option<Claims>,
) -> Result<Response> {
    // Parse the request body to potentially inject user info
    let body = req.body();
    let mut request_data: Value = if body.is_empty() {
        // If there's no body, we shouldn't forward an empty object
        // Let's just forward the request as-is
        eprintln!("Warning: Empty request body received");
        serde_json::json!(null)
    } else {
        match serde_json::from_slice(body) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to parse request body as JSON: {}", e);
                eprintln!("Request body: {:?}", String::from_utf8_lossy(body));
                return Err(anyhow::anyhow!("Invalid JSON in request body: {}", e));
            }
        }
    };

    // If this is an initialize request and we have claims, inject user info
    if let Some(ref claims) = claims {
        if let Some(obj) = request_data.as_object_mut() {
            if let Some(method) = obj.get("method").and_then(|m| m.as_str()) {
                if method == "initialize" {
                    // Add user context to the request
                    if let Some(params) = obj.get_mut("params").and_then(|p| p.as_object_mut()) {
                        params.insert(
                            "_authContext".to_string(),
                            serde_json::json!({
                                "authenticated_user": claims.sub,
                                "email": claims.email,
                            }),
                        );
                    }
                }
            }
        }
    }

    // Build the request to forward to MCP gateway
    eprintln!("Forwarding request to: {}", &config.mcp_gateway_url);
    
    // Determine the body to forward
    let forward_body = if body.is_empty() {
        // Forward empty body as-is
        eprintln!("Forwarding empty request body");
        body.to_vec()
    } else if request_data == serde_json::json!(null) {
        // If we couldn't parse, forward original body
        body.to_vec()
    } else {
        // Forward modified JSON
        eprintln!("Request data: {}", serde_json::to_string_pretty(&request_data)?);
        serde_json::to_vec(&request_data)?
    };
    
    let forward_req = Request::builder()
        .method(req.method().clone())
        .uri(&config.mcp_gateway_url)
        .header("Content-Type", "application/json")
        .body(forward_body)
        .build();

    // Forward the request
    let resp: spin_sdk::http::Response = spin_sdk::http::send(forward_req).await?;

    // Parse the response to potentially inject auth info
    let resp_body = resp.body();
    let mut response_data: Value = if resp_body.is_empty() {
        serde_json::json!({})
    } else {
        match serde_json::from_slice(resp_body) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to parse MCP gateway response as JSON: {}", e);
                eprintln!("Response status: {}", resp.status());
                eprintln!("Response body: {:?}", String::from_utf8_lossy(resp_body));
                return Err(anyhow::anyhow!("Invalid JSON response from MCP gateway: {}", e));
            }
        }
    };

    // If this is an initialize response and we have claims, inject auth info into serverInfo
    if let Some(ref claims) = claims {
        if let Some(result) = response_data
            .as_object_mut()
            .and_then(|obj| obj.get_mut("result"))
            .and_then(|r| r.as_object_mut())
        {
            if let Some(server_info) = result
                .get_mut("serverInfo")
                .and_then(|si| si.as_object_mut())
            {
                server_info.insert(
                    "authInfo".to_string(),
                    serde_json::json!({
                        "authenticated_user": claims.sub,
                        "email": claims.email,
                    }),
                );
            }
        }
    }

    // Build the response to return
    if response_data == serde_json::json!(null) || resp_body.is_empty() {
        // Return the original response as-is
        Ok(Response::builder()
            .status(*resp.status())
            .body(resp_body.to_vec())
            .build())
    } else {
        // Return the modified JSON response
        Ok(Response::builder()
            .status(*resp.status())
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&response_data)?)
            .build())
    }
}