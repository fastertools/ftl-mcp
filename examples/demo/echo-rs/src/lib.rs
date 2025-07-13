use ftl_sdk::{tool, ToolResponse};
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct EchoRsInput {
    /// The input message to process
    message: String,
}

/// An MCP tool written in Rust
#[tool]
fn echo_rs(input: EchoRsInput) -> ToolResponse {
    // TODO: Implement your tool logic here
    ToolResponse::text(format!("Processed: {}", input.message))
}