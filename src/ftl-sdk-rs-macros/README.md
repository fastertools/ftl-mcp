# FTL SDK Rust Macros

Procedural macros for reducing boilerplate in FTL tool components written in Rust.

## Overview

This crate provides two main macros:
- `#[tool]` - Attribute macro for tool handler functions
- `#[tool_component]` - Attribute macro for complete tool components

## Usage

### Basic Tool Handler

The `#[tool]` macro simplifies creating tool handlers:

```rust
use ftl_sdk_macros::tool;
use ftl_sdk::{ToolResponse, ToolContent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct EchoRequest {
    message: String,
}

#[tool]
fn echo(req: EchoRequest) -> ToolResponse {
    ToolResponse {
        content: vec![ToolContent::Text {
            text: format!("Echo: {}", req.message),
            annotations: None,
        }],
        structured_content: None,
        is_error: None,
    }
}
```

### Complete Tool Component

The `#[tool_component]` macro generates a complete HTTP handler with metadata:

```rust
use ftl_sdk_macros::tool_component;
use ftl_sdk::{ToolMetadata, ToolResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize)]
struct CalculatorRequest {
    operation: String,
    a: f64,
    b: f64,
}

#[tool_component(
    metadata = ToolMetadata {
        name: "calculator".to_string(),
        title: Some("Calculator Tool".to_string()),
        description: Some("Performs basic arithmetic operations".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": { "type": "string", "enum": ["add", "subtract", "multiply", "divide"] },
                "a": { "type": "number" },
                "b": { "type": "number" }
            },
            "required": ["operation", "a", "b"]
        }),
        output_schema: None,
        annotations: None,
        _meta: None,
    }
)]
fn calculator(req: CalculatorRequest) -> ToolResponse {
    let result = match req.operation.as_str() {
        "add" => req.a + req.b,
        "subtract" => req.a - req.b,
        "multiply" => req.a * req.b,
        "divide" => {
            if req.b == 0.0 {
                return ToolResponse::error("Cannot divide by zero");
            }
            req.a / req.b
        }
        _ => return ToolResponse::error("Invalid operation"),
    };
    
    ToolResponse::text(&format!("{} {} {} = {}", req.a, req.operation, req.b, result))
}

// The macro generates the HTTP handler automatically!
```

## Generated Code

The `#[tool_component]` macro generates:
- A `handle_request` function that returns metadata on GET and executes the handler on POST
- Automatic JSON deserialization of request bodies
- Error handling with proper HTTP status codes
- Correct Content-Type headers

## Important: Input Validation

Just like with the TypeScript SDK, **tools should NOT validate inputs themselves**. The FTL gateway handles all input validation against your tool's JSON Schema before invoking your handler. This means:

- Your handler can assume all inputs match the schema
- Focus on business logic, not validation
- The gateway enforces all JSON Schema constraints

## Best Practices

1. **Use serde for Input Types**: Define input structs with `#[derive(Deserialize)]`

2. **Leverage ToolResponse Helpers**: Use convenience methods like `ToolResponse::text()` and `ToolResponse::error()`

3. **Keep Metadata in Sync**: Ensure your input schema matches your Rust struct definition

4. **Error Handling**: Return `ToolResponse::error()` for business logic errors - the macro handles panics

## Example with Spin

```rust
use ftl_sdk_macros::tool_component;
use ftl_sdk::{ToolMetadata, ToolResponse};
use serde::Deserialize;
use spin_sdk::http::{Request, Response};

#[derive(Deserialize)]
struct MyInput {
    value: String,
}

#[tool_component(
    metadata = ToolMetadata {
        name: "my-tool".to_string(),
        // ... metadata fields
    }
)]
fn my_tool(input: MyInput) -> ToolResponse {
    ToolResponse::text(&format!("Processed: {}", input.value))
}

#[spin_sdk::http_component]
fn handle_request(req: Request) -> Response {
    // The macro generates handle_request for you!
    handle_request(req)
}
```

## License

Apache-2.0