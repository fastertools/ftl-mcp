use anyhow::Result;
use spin_sdk::http::{IntoResponse, Method, Request, Response};
use spin_sdk::variables;

mod auth;
mod jwks;
mod metadata;
mod proxy;

use auth::{AuthKitConfig, verify_request};
use metadata::handle_metadata_request;
use proxy::forward_to_mcp_gateway;

/// Main entry point for the authentication gateway
#[spin_sdk::http_component]
async fn handle_request(req: Request) -> Result<impl IntoResponse> {
    // Load AuthKit configuration from Spin variables
    let config = AuthKitConfig {
        issuer: variables::get("authkit_issuer")
            .unwrap_or_else(|_| "https://example.authkit.app".to_string()),
        jwks_uri: variables::get("authkit_jwks_uri")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                let issuer = variables::get("authkit_issuer")
                    .unwrap_or_else(|_| "https://example.authkit.app".to_string());
                format!("{}/oauth2/jwks", issuer)
            }),
        audience: variables::get("authkit_audience")
            .ok()
            .filter(|s| !s.is_empty()),
        mcp_gateway_url: variables::get("mcp_gateway_url")
            .unwrap_or_else(|_| "http://ftl-mcp-gateway.spin.internal/mcp-internal".to_string()),
    };

    let path = req.path();
    let method = req.method();

    // Extract host header for metadata endpoints
    let host = req
        .headers()
        .find(|(name, _)| name.eq_ignore_ascii_case("host"))
        .and_then(|(_, value)| value.as_str());

    // Handle metadata endpoints (no auth required)
    if matches!(
        path,
        "/.well-known/oauth-protected-resource" | "/.well-known/oauth-authorization-server"
    ) {
        return handle_metadata_request(path, &config, host);
    }

    // Handle OPTIONS requests (CORS preflight)
    if *method == Method::Options {
        return Ok(Response::builder()
            .status(204)
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
            .header("Access-Control-Allow-Headers", "Content-Type, Authorization")
            .header("Access-Control-Max-Age", "86400")
            .build());
    }

    // All other requests require authentication
    match verify_request(&req, &config, host).await {
        Ok(claims) => {
            eprintln!("Authentication successful, forwarding to MCP gateway");
            // Forward authenticated request to MCP gateway
            match forward_to_mcp_gateway(req, &config, Some(claims)).await {
                Ok(response) => Ok(response),
                Err(e) => {
                    eprintln!("Failed to forward request to MCP gateway: {}", e);
                    Ok(Response::builder()
                        .status(502)
                        .body(format!("Gateway error: {}", e))
                        .build())
                }
            }
        }
        Err(auth_error) => {
            eprintln!("Authentication failed, returning 401");
            Ok(auth_error)
        }
    }
}