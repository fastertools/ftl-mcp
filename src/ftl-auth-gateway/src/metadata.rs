use spin_sdk::http::{Request, Response};

use crate::auth::AuthKitConfig;

/// Handle OAuth metadata endpoints
pub fn handle_metadata_request(
    path: &str,
    config: &AuthKitConfig,
    host: Option<&str>,
    req: &Request,
) -> Response {
    match path {
        "/.well-known/oauth-protected-resource" => {
            // The resource should be this server's MCP endpoint URL
            // Use the exact host header sent by the client
            let resource_url = host.map_or_else(
                || {
                    eprintln!("No host header found, using default");
                    "http://127.0.0.1:3000/mcp".to_string() // Default fallback
                },
                |h| {
                    // Check X-Forwarded-Proto header for protocol
                    let forwarded_proto = req
                        .headers()
                        .find(|(name, _)| name.eq_ignore_ascii_case("x-forwarded-proto"))
                        .and_then(|(_, value)| value.as_str());

                    // Determine protocol
                    let protocol = forwarded_proto.unwrap_or_else(|| {
                        if h.contains(":443")
                            || h.contains(".fermyon.tech")
                            || h.contains(".fermyon.cloud")
                        {
                            "https"
                        } else if h.contains(":80")
                            || h.starts_with("localhost")
                            || h.starts_with("127.0.0.1")
                        {
                            "http"
                        } else {
                            // Default to https for production domains
                            "https"
                        }
                    });

                    let url = format!("{protocol}://{h}/mcp");
                    eprintln!("Returning resource URL: {url}");
                    url
                },
            );

            let metadata = serde_json::json!({
                "resource": resource_url,
                "authorization_servers": [&config.issuer],
                "bearer_methods_supported": ["header"]
            });

            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(metadata.to_string())
                .build()
        }
        "/.well-known/oauth-authorization-server" => {
            // For legacy clients, proxy to AuthKit's metadata
            // In a real implementation, we would fetch this from AuthKit
            // For now, return a standard response
            let issuer = &config.issuer;
            let metadata = serde_json::json!({
                "issuer": issuer,
                "authorization_endpoint": format!("{issuer}/oauth2/authorize"),
                "token_endpoint": format!("{issuer}/oauth2/token"),
                "jwks_uri": &config.jwks_uri,
                "registration_endpoint": format!("{issuer}/oauth2/register"),
                "introspection_endpoint": format!("{issuer}/oauth2/introspection"),
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

            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(metadata.to_string())
                .build()
        }
        _ => Response::builder()
            .status(404)
            .body("Not found".to_string())
            .build(),
    }
}

