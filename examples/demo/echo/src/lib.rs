use ftl_sdk::{ToolMetadata, ToolResponse, ToolContent, ToolAnnotations};
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
                title: None,
                read_only_hint: Some(true),
                destructive_hint: None,
                idempotent_hint: Some(true),
                open_world_hint: None,
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
    let request: EchoRequest = match serde_json::from_slice(req.body()) {
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

    // Create the echo response
    let _echo_response = EchoResponse {
        echo: request.message.clone(),
        received: true,
    };

    // Create the MCP tool response
    let tool_response = ToolResponse::text(format!("Echo: {}", request.message));

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(serde_json::to_vec(&tool_response)?)
        .build())
}