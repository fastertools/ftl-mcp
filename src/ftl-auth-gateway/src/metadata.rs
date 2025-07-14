use spin_sdk::http::{Request, Response};

use crate::providers::AuthProvider;

/// Handle OAuth metadata endpoints
pub fn handle_metadata_request(
    path: &str,
    provider: &dyn AuthProvider,
    host: Option<&str>,
    req: &Request,
) -> Response {
    // Determine resource URL first
    let resource_url = determine_resource_url(host, req);

    match path {
        "/.well-known/oauth-protected-resource" => {
            let metadata = serde_json::json!({
                "resource": resource_url,
                "authorization_servers": [provider.issuer()],
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
            // Return provider-specific metadata
            let discovery = provider.discovery_metadata(&resource_url);
            let metadata = serde_json::json!({
                "issuer": discovery.issuer,
                "authorization_endpoint": discovery.authorization_endpoint,
                "token_endpoint": discovery.token_endpoint,
                "jwks_uri": discovery.jwks_uri,
                "userinfo_endpoint": discovery.userinfo_endpoint,
                "revocation_endpoint": discovery.revocation_endpoint,
                "introspection_endpoint": discovery.introspection_endpoint,
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

/// Determine the resource URL based on request headers
fn determine_resource_url(host: Option<&str>, req: &Request) -> String {
    host.map_or_else(
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
                if h.contains(":443") || h.contains(".fermyon.tech") || h.contains(".fermyon.cloud")
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
    )
}
