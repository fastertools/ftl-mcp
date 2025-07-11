# FTL SDK Rust Macros

Procedural macros for reducing boilerplate in FTL tool components written in Rust.

## Overview

This crate provides the `#[tool]` attribute macro for creating tool handler functions with minimal boilerplate. The macro:

- Automatically derives JSON schemas from your input types (requires `JsonSchema` derive)
- Supports both synchronous and asynchronous functions
- Generates the complete HTTP handler with metadata
- Handles all the boilerplate for you

## Usage

### Basic Tool Handler

The `#[tool]` macro simplifies creating tool handlers:

```rust
use ftl_sdk_macros::tool;
use ftl_sdk::{ToolResponse, ToolContent};
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct EchoRequest {
    message: String,
}

/// Echoes back the input message
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

### Complete Example

Here's a more complete example showing how the macro works with complex input types:

```rust
use ftl_sdk_macros::tool;
use ftl_sdk::ToolResponse;
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct CalculatorRequest {
    operation: String,
    a: f64,
    b: f64,
}

/// Performs basic arithmetic operations  
#[tool]
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

The `#[tool]` macro generates:
- A `handle_tool_component` async function that returns metadata on GET and executes the handler on POST
- Automatic JSON deserialization of request bodies
- Error handling with proper HTTP status codes
- Correct Content-Type headers
- Full Spin HTTP component integration

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

The `#[tool]` macro automatically generates the Spin HTTP component handler:

```rust
use ftl_sdk_macros::tool;
use ftl_sdk::ToolResponse;
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct MyInput {
    value: String,
}

/// Processes the input value
#[tool]
fn my_tool(input: MyInput) -> ToolResponse {
    ToolResponse::text(&format!("Processed: {}", input.value))
}

// That's it! The macro generates the HTTP handler for you.
// No need to write any additional code.
```

## Async Support

The `#[tool]` macro automatically detects whether your function is async and generates the appropriate code:

```rust
use ftl_sdk_macros::tool;
use ftl_sdk::ToolResponse;
use serde::Deserialize;
use schemars::JsonSchema;
use spin_sdk::http::{send, Method, Request, Response};

#[derive(Deserialize, JsonSchema)]
struct WeatherInput {
    location: String,
}

/// Get weather for a location (async example)
#[tool]
async fn get_weather(input: WeatherInput) -> ToolResponse {
    // Make async HTTP requests
    let req = Request::builder()
        .method(Method::Get)
        .uri(format!("https://api.example.com/weather?location={}", input.location))
        .build();
    
    match send(req).await {
        Ok(res) => {
            // Process response...
            ToolResponse::text("Weather data here")
        }
        Err(e) => ToolResponse::error(format!("Failed to fetch weather: {}", e))
    }
}
```

## Custom Schemas (Advanced)

While the macro automatically derives schemas from your types, you can still provide a custom schema if needed:

```rust
#[tool(
    input_schema = serde_json::json!({
        "type": "object",
        "properties": {
            "custom": { "type": "string" }
        }
    })
)]
fn advanced_tool(input: MyInput) -> ToolResponse {
    // Custom schema handling
    ToolResponse::text("Processed")
}
```

## License

Apache-2.0