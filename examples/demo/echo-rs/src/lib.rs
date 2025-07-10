use ftl_sdk::{tool, ToolResponse};
use serde::Deserialize;
use schemars::{schema_for, JsonSchema};

#[derive(Deserialize, JsonSchema)]
struct EchoInput {
    /// The message to echo back to the caller, verbatim
    message: String,
}

/// Echo the message back to the caller
#[tool(
    input_schema = serde_json::to_value(schema_for!(EchoInput)).unwrap()
)]
fn echo_rs(input: EchoInput) -> ToolResponse {
    ToolResponse::text(format!("Echo: {}", input.message))
}
