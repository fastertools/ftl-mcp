use ftl_sdk::{tool, ToolResponse};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct EchoInput {
    message: String,
}

/// A minimal echo tool with the ftl-sdk macro
#[tool(
    input_schema = json!({
        "type": "object",
        "properties": {
            "message": {
                "type": "string",
                "description": "The message to echo back to the caller"
            }
        },
        "required": ["message"]
    })
)]
fn echo_rs(input: EchoInput) -> ToolResponse {
    ToolResponse::text(format!("Echo: {}", input.message))
}
