mod gateway;
mod mcp_types;

use spin_sdk::http::{IntoResponse, Request};
use spin_sdk::http_component;

#[http_component]
async fn handle_mcp_gateway(req: Request) -> anyhow::Result<impl IntoResponse> {
    Ok(gateway::handle_mcp_request(req).await)
}
