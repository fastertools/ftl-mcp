use serde::{Deserialize, Serialize};
use serde_json::json;
use spin_sdk::http::{IntoResponse, Method, Request, Response};
use spin_sdk::http_component;

#[derive(Debug, Deserialize)]
struct EchoRequest {
    message: String,
}

#[derive(Debug, Serialize)]
struct EchoResponse {
    echo: String,
    received: bool,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
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
}

#[http_component]
fn handle_echo(req: Request) -> anyhow::Result<impl IntoResponse> {
    // Handle GET requests for tool metadata
    if *req.method() == Method::Get {
        let metadata = ToolMetadata {
            name: "echo".to_string(),
            title: Some("Echo Tool".to_string()),
            description: Some("Echoes back the input message".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "The message to echo back"
                    }
                },
                "required": ["message"]
            }),
            output_schema: None,
            annotations: Some(ToolAnnotations {
                title: None, // title is now at the top level
                read_only_hint: Some(true),
                idempotent_hint: Some(true),
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
            .body(serde_json::to_vec(&ErrorResponse {
                error: "Method not allowed. Only GET and POST are supported.".to_string(),
            })?)
            .build());
    }

    // Parse the request body
    let request: EchoRequest = match serde_json::from_slice(req.body()) {
        Ok(r) => r,
        Err(e) => {
            return Ok(Response::builder()
                .status(400)
                .header("Content-Type", "application/json")
                .body(serde_json::to_vec(&ErrorResponse {
                    error: format!("Invalid request body: {}", e),
                })?)
                .build());
        }
    };

    // Create the echo response
    let _echo_response = EchoResponse {
        echo: request.message.clone(),
        received: true,
    };

    // Create the MCP tool response
    let tool_response = ToolResponse {
        content: vec![ToolContent::Text {
            text: format!("Echo: {}", request.message),
        }],
        structured_content: None, // No outputSchema defined
        is_error: None,
    };

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(serde_json::to_vec(&tool_response)?)
        .build())
}