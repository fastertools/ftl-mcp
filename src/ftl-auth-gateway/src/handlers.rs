use spin_sdk::http::{Method, Request, Response};

use crate::{
    auth::{self, verify_request},
    config::GatewayConfig,
    logging::Logger,
    metadata::handle_metadata_request,
    proxy::forward_to_mcp_gateway,
};

/// Handle metadata endpoints (no auth required)
pub fn handle_metadata_endpoints(
    path: &str,
    provider: Option<&dyn crate::providers::AuthProvider>,
    host: Option<&str>,
    req: &Request,
    logger: &Logger<'_>,
) -> Option<Response> {
    if !matches!(
        path,
        "/.well-known/oauth-protected-resource" | "/.well-known/oauth-authorization-server"
    ) {
        return None;
    }

    logger
        .info("Metadata request")
        .field("path", path)
        .field("host", host.unwrap_or("unknown"))
        .emit();

    provider.map_or_else(
        || {
            logger.warn("No auth provider configured").emit();
            Some(
                Response::builder()
                    .status(500)
                    .body("No authentication provider configured")
                    .build(),
            )
        },
        |p| Some(handle_metadata_request(path, p, host, req)),
    )
}

/// Handle OPTIONS requests (CORS preflight)
pub fn handle_cors_preflight(method: &Method) -> Option<Response> {
    if *method != Method::Options {
        return None;
    }

    Some(
        Response::builder()
            .status(204)
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
            .header(
                "Access-Control-Allow-Headers",
                "Content-Type, Authorization",
            )
            .header("Access-Control-Max-Age", "86400")
            .build(),
    )
}

/// Handle authenticated requests
pub async fn handle_authenticated_request(
    req: Request,
    config: &GatewayConfig,
    provider: Option<&dyn crate::providers::AuthProvider>,
    host: Option<&str>,
    trace_id: &str,
    logger: &Logger<'_>,
) -> Response {
    let Some(p) = provider else {
        logger.warn("No authentication provider configured").emit();
        return auth::auth_error_response(
            "No authentication provider configured",
            host,
            Some(trace_id),
        );
    };

    match verify_request(&req, p, host, Some(trace_id)).await {
        Ok((claims, user_context)) => {
            logger
                .info("Authentication successful")
                .field("provider", p.name())
                .field("user_id", &user_context.id)
                .emit();

            // Forward authenticated request to MCP gateway
            let auth_config = crate::auth::AuthConfig {
                mcp_gateway_url: config.mcp_gateway_url.clone(),
            };

            match forward_to_mcp_gateway(req, &auth_config, Some((claims, user_context)), trace_id)
                .await
            {
                Ok(response) => response,
                Err(e) => {
                    logger
                        .error("Failed to forward request to MCP gateway")
                        .field("error", &e)
                        .emit();
                    Response::builder()
                        .status(502)
                        .body(format!("Gateway error: {e}"))
                        .build()
                }
            }
        }
        Err(auth_error) => {
            logger
                .warn("Authentication failed")
                .field("provider", p.name())
                .emit();
            auth_error
        }
    }
}
