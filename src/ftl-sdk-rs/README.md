# ftl-sdk (Rust)

Rust SDK for building Model Context Protocol (MCP) tools that compile to WebAssembly.

## Installation

```toml
[dependencies]
ftl-sdk = "0.2"
schemars = "0.8"  # For automatic schema generation
serde = { version = "1.0", features = ["derive"] }
```

## Overview

This SDK provides:
- MCP protocol type definitions
- `#[tool]` procedural macro for minimal boilerplate
- Automatic JSON schema generation using schemars
- Convenience methods for creating responses

## Quick Start

### Using the `#[tool]` Macro

The simplest way to create a tool:

```rust
use ftl_sdk::{tool, ToolResponse};
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct AddInput {
    /// First number to add
    a: i32,
    /// Second number to add
    b: i32,
}

/// Adds two numbers together
#[tool]
fn add(input: AddInput) -> ToolResponse {
    let result = input.a + input.b;
    ToolResponse::text(format!("{} + {} = {}", input.a, input.b, result))
}
```

The `#[tool]` macro automatically:
- Generates the HTTP handler
- Creates metadata from the function name and doc comments
- Derives JSON schema from your input type using schemars
- Handles GET/POST requests appropriately

### Manual Implementation

For more control, implement the protocol manually:

```rust
use ftl_sdk::{ToolMetadata, ToolResponse};
use serde_json::json;
use spin_sdk::http::{IntoResponse, Method, Request, Response};
use spin_sdk::http_component;

#[http_component]
fn handle_tool(req: Request) -> anyhow::Result<impl IntoResponse> {
    match *req.method() {
        Method::Get => {
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
            
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(serde_json::to_vec(&metadata)?)
                .build())
        }
        Method::Post => {
            // Parse input and execute tool logic
            let response = ToolResponse::text("Tool executed successfully!");
            
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(serde_json::to_vec(&response)?)
                .build())
        }
        _ => Ok(Response::builder()
            .status(405)
            .header("Allow", "GET, POST")
            .body("Method not allowed")
            .build())
    }
}
```

## Building to WebAssembly

Tools must be compiled to WebAssembly for the Spin platform:

```toml
# Cargo.toml
[dependencies]
ftl-sdk = "0.2"
schemars = "0.8"
serde = { version = "1.0", features = ["derive"] }
spin-sdk = "3.0"

[lib]
crate-type = ["cdylib"]
```

Build command:
```bash
cargo build --target wasm32-wasip1 --release
```

## Response Helpers

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

// Multiple content items
let response = ToolResponse {
    content: vec![
        ToolContent::text("Processing complete", None),
        ToolContent::image(base64_data, "image/png", None),
    ],
    structured_content: None,
    is_error: None,
};
```

## Advanced Features

### Async Tools

The `#[tool]` macro supports async functions:

```rust
#[tool]
async fn fetch_weather(input: WeatherInput) -> ToolResponse {
    let weather = fetch_from_api(&input.location).await?;
    ToolResponse::text(format!("Weather: {}", weather))
}
```

### Custom Metadata

Override automatic metadata generation:

```rust
#[tool(
    name = "custom_name",
    title = "Custom Title",
    description = "Custom description"
)]
fn my_tool(input: MyInput) -> ToolResponse {
    // Implementation
}
```

### Tool Annotations

Add hints about tool behavior:

```rust
#[tool(
    read_only_hint = true,
    idempotent_hint = true
)]
fn query_data(input: QueryInput) -> ToolResponse {
    // Read-only, idempotent operation
}
```