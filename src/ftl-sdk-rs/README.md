# ftl-sdk (Rust)

Thin SDK providing MCP protocol types for FTL tool development.

## Installation

```toml
[dependencies]
ftl-sdk = "0.1"
```

## Usage

This crate provides only type definitions - no HTTP server logic. Use with any web framework:

```rust
use ftl_sdk::{ToolMetadata, ToolResponse};
use serde_json::json;
use spin_sdk::http::{IntoResponse, Method, Request, Response};
use spin_sdk::http_component;

#[http_component]
fn handle_tool(req: Request) -> anyhow::Result<impl IntoResponse> {
    // Handle GET requests for tool metadata
    if *req.method() == Method::Get {
        let metadata = ToolMetadata {
            name: "my-tool".to_string(),
            title: Some("My Tool".to_string()),
            description: Some("Does something useful".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "input": { "type": "string" }
                },
                "required": ["input"]
            }),
            output_schema: None,
            annotations: None,
            meta: None,
        };
        
        return Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(serde_json::to_vec(&metadata)?)
            .build());
    }
    
    // Handle POST requests for tool execution
    if *req.method() == Method::Post {
        // Parse input, do work...
        let response = ToolResponse::text("Tool executed successfully!");
        
        return Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(serde_json::to_vec(&response)?)
            .build());
    }
    
    // Handle other methods...
}
```

## Convenience Methods

```rust
// Simple text response
let response = ToolResponse::text("Hello, world!");

// Error response
let response = ToolResponse::error("Something went wrong");

// Response with structured content
let response = ToolResponse::with_structured(
    "Calculation complete",
    json!({ "result": 42 })
);
```