use anyhow::Result;
use spin_sdk::http::{IntoResponse, Request};

mod auth;
mod config;
mod handlers;
mod jwks;
mod logging;
mod metadata;
mod providers;
mod proxy;

use config::GatewayConfig;
use handlers::{handle_authenticated_request, handle_cors_preflight, handle_metadata_endpoints};
use logging::{get_trace_id, Logger};

/// Main entry point for the authentication gateway
#[spin_sdk::http_component]
async fn handle_request(req: Request) -> Result<impl IntoResponse> {
    // Load gateway configuration
    let config = GatewayConfig::from_spin_vars()?;
    let registry = config.build_registry();

    // Extract trace ID for structured logging
    let trace_id = get_trace_id(&req, &config.trace_id_header);
    let logger = Logger::new(&trace_id);


    let path = req.path();
    let method = req.method();

    // Extract host header for metadata endpoints
    // Check multiple headers that might contain the host
    let host = req
        .headers()
        .find(|(name, _)| name.eq_ignore_ascii_case("host"))
        .and_then(|(_, value)| value.as_str())
        .map(String::from)
        .or_else(|| {
            req.headers()
                .find(|(name, _)| name.eq_ignore_ascii_case("x-forwarded-host"))
                .and_then(|(_, value)| value.as_str())
                .map(String::from)
        })
        .or_else(|| {
            req.headers()
                .find(|(name, _)| name.eq_ignore_ascii_case("x-original-host"))
                .and_then(|(_, value)| value.as_str())
                .map(String::from)
        });

    // Handle metadata endpoints
    if let Some(response) = handle_metadata_endpoints(path, &registry, host.as_deref(), &req, &logger) {
        return Ok(response);
    }

    // Handle CORS preflight
    if let Some(response) = handle_cors_preflight(method) {
        return Ok(response);
    }

    // All other requests require authentication
    Ok(handle_authenticated_request(req, &config, &registry, host.as_deref(), &trace_id, &logger).await)
}
