use ftl_sdk::{ToolMetadata, ToolResponse, ToolAnnotations};
use serde::{Deserialize, Serialize};
use serde_json::json;
use spin_sdk::http::{IntoResponse, Method, Request, Response};
use spin_sdk::http_component;

#[derive(Debug, Deserialize)]
struct AddRequest {
    a: f64,
    b: f64,
}

#[derive(Debug, Serialize)]
struct AddResponse {
    result: f64,
    operation: String,
}

#[http_component]
fn handle_add(req: Request) -> anyhow::Result<impl IntoResponse> {
    // Handle GET requests for tool metadata
    if *req.method() == Method::Get {
        let metadata = ToolMetadata {
            name: "add".to_string(),
            title: Some("Addition Calculator".to_string()),
            description: Some("Adds two numbers together".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "a": {
                        "type": "number",
                        "description": "First number"
                    },
                    "b": {
                        "type": "number",
                        "description": "Second number"
                    }
                },
                "required": ["a", "b"]
            }),
            output_schema: Some(json!({
                "type": "object",
                "properties": {
                    "result": {
                        "type": "number",
                        "description": "The sum of a and b"
                    },
                    "operation": {
                        "type": "string",
                        "description": "Description of the operation performed"
                    }
                },
                "required": ["result", "operation"]
            })),
            annotations: Some(ToolAnnotations {
                title: None,
                read_only_hint: Some(true),
                destructive_hint: None,
                idempotent_hint: Some(true),
                open_world_hint: Some(false),
            }),
            meta: None,
        };

        return Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(serde_json::to_vec(&metadata)?)
            .build());
    }

    // Only accept POST requests for tool execution
    if *req.method() != Method::Post {
        return Ok(Response::builder()
            .status(405)
            .header("Allow", "GET, POST")
            .body("Method not allowed. Only GET and POST are supported.")
            .build());
    }

    // Parse the request body
    let request: AddRequest = match serde_json::from_slice(req.body()) {
        Ok(r) => r,
        Err(e) => {
            let error_response = ToolResponse::error(format!("Invalid request body: {}", e));
            return Ok(Response::builder()
                .status(400)
                .header("Content-Type", "application/json")
                .body(serde_json::to_vec(&error_response)?)
                .build());
        }
    };

    // Perform the addition
    let result = request.a + request.b;

    // Create the structured response
    let add_response = AddResponse {
        result,
        operation: format!("{} + {} = {}", request.a, request.b, result),
    };

    // Create the MCP tool response with structured content
    let tool_response = ToolResponse::with_structured(
        add_response.operation.clone(),
        serde_json::to_value(&add_response)?
    );

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(serde_json::to_vec(&tool_response)?)
        .build())
}