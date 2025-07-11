use anyhow::Result;
use spin_sdk::http::Response;

use crate::auth::AuthKitConfig;

/// Handle OAuth metadata endpoints
pub fn handle_metadata_request(
    path: &str,
    config: &AuthKitConfig,
    host: Option<&str>,
) -> Result<Response> {
    match path {
        "/.well-known/oauth-protected-resource" => {
            // The resource should be this server's MCP endpoint URL
            // Use the exact host header sent by the client
            let resource_url = match host {
                Some(h) => {
                    // Host header doesn't include protocol, so we need to determine it
                    let protocol = if h.contains(":443") { "https" } else { "http" };
                    format!("{}://{}/mcp", protocol, h)
                }
                None => "http://127.0.0.1:3000/mcp".to_string(), // Default to 127.0.0.1 as that's what clients typically use
            };

            let metadata = serde_json::json!({
                "resource": resource_url,
                "authorization_servers": [&config.issuer],
                "bearer_methods_supported": ["header"]
            });

            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(metadata.to_string())
                .build())
        }
        "/.well-known/oauth-authorization-server" => {
            // For legacy clients, proxy to AuthKit's metadata
            // In a real implementation, we would fetch this from AuthKit
            // For now, return a standard response
            let metadata = serde_json::json!({
                "issuer": &config.issuer,
                "authorization_endpoint": format!("{}/oauth2/authorize", &config.issuer),
                "token_endpoint": format!("{}/oauth2/token", &config.issuer),
                "jwks_uri": &config.jwks_uri,
                "registration_endpoint": format!("{}/oauth2/register", &config.issuer),
                "introspection_endpoint": format!("{}/oauth2/introspection", &config.issuer),
                "response_types_supported": ["code"],
                "response_modes_supported": ["query"],
                "grant_types_supported": ["authorization_code", "refresh_token"],
                "code_challenge_methods_supported": ["S256"],
                "token_endpoint_auth_methods_supported": [
                    "none",
                    "client_secret_post",
                    "client_secret_basic"
                ],
                "scopes_supported": ["email", "offline_access", "openid", "profile"]
            });

            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(metadata.to_string())
                .build())
        }
        _ => Ok(Response::builder()
            .status(404)
            .body("Not found".to_string())
            .build()),
    }
}