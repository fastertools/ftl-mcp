use ftl_sdk::{tool, ToolResponse};
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct EchoInput {
    /// The message to echo back to the caller, verbatim
    message: String
}

/// Echo the message back to the caller
#[tool]
fn echo_rs(input: EchoInput) -> ToolResponse {
    ToolResponse::text(format!("Echo: {}", input.message))
}
