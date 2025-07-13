use ftl_sdk::{tool, ToolResponse};
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct {{project-name | pascal_case}}Input {
    /// The input message to process
    message: String,
}

/// {{tool-description}}
#[tool]
fn {{project-name | snake_case}}(input: {{project-name | pascal_case}}Input) -> ToolResponse {
    // TODO: Implement your tool logic here
    ToolResponse::text(format!("Processed: {}", input.message))
}