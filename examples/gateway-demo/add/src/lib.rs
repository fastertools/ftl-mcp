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

#[derive(Debug, Serialize)]
struct ToolResponse {
    content: Vec<ToolContent>,
    #[serde(rename = "structuredContent", skip_serializing_if = "Option::is_none")]
    structured_content: Option<serde_json::Value>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    is_error: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },
}

#[derive(Debug, Serialize)]
struct ToolMetadata {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(rename = "inputSchema")]
    input_schema: serde_json::Value,
    #[serde(rename = "outputSchema", skip_serializing_if = "Option::is_none")]
    output_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    annotations: Option<ToolAnnotations>,
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    meta: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct ToolAnnotations {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(rename = "readOnlyHint", skip_serializing_if = "Option::is_none")]
    read_only_hint: Option<bool>,
    #[serde(rename = "idempotentHint", skip_serializing_if = "Option::is_none")]
    idempotent_hint: Option<bool>,
    #[serde(rename = "openWorldHint", skip_serializing_if = "Option::is_none")]
    open_world_hint: Option<bool>,
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
            .header("Content-Type", "application/json")
            .body(serde_json::to_vec(&ToolResponse {
                content: vec![ToolContent::Text {
                    text: "Method not allowed. Only GET and POST are supported.".to_string(),
                }],
                structured_content: None,
                is_error: Some(true),
            })?)
            .build());
    }

    // Parse the request body
    let request: AddRequest = match serde_json::from_slice(req.body()) {
        Ok(r) => r,
        Err(e) => {
            return Ok(Response::builder()
                .status(400)
                .header("Content-Type", "application/json")
                .body(serde_json::to_vec(&ToolResponse {
                    content: vec![ToolContent::Text {
                        text: format!("Invalid request body: {}", e),
                    }],
                    structured_content: None,
                    is_error: Some(true),
                })?)
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

    // Create the MCP tool response
    let tool_response = ToolResponse {
        content: vec![ToolContent::Text {
            text: add_response.operation.clone(),
        }],
        structured_content: Some(serde_json::to_value(&add_response)?),
        is_error: None,
    };

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(serde_json::to_vec(&tool_response)?)
        .build())
}