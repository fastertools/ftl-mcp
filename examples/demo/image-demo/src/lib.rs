use ftl_sdk::{ToolMetadata, ToolResponse, ToolContent, ToolAnnotations};
use serde::{Deserialize, Serialize};
use serde_json::json;
use spin_sdk::http::{IntoResponse, Method, Request, Response};
use spin_sdk::http_component;
use base64::{Engine as _, engine::general_purpose};

#[derive(Debug, Deserialize)]
struct ImageRequest {
    size: Option<u32>,
    color: Option<String>,
}

#[http_component]
fn handle_image_demo(req: Request) -> anyhow::Result<impl IntoResponse> {
    // Handle GET requests for tool metadata
    if *req.method() == Method::Get {
        let metadata = ToolMetadata {
            name: "image-demo".to_string(),
            title: Some("Image Generator Demo".to_string()),
            description: Some("Generates a simple SVG image with customizable size and color".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "size": {
                        "type": "integer",
                        "description": "Size of the image in pixels (default: 200)",
                        "minimum": 50,
                        "maximum": 500
                    },
                    "color": {
                        "type": "string",
                        "description": "Color of the shape (default: blue)",
                        "enum": ["red", "green", "blue", "yellow", "purple", "orange"]
                    }
                },
                "required": []
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
    let request: ImageRequest = match serde_json::from_slice(req.body()) {
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

    // Set defaults
    let size = request.size.unwrap_or(200);
    let color = request.color.unwrap_or_else(|| "blue".to_string());

    // Generate a simple SVG image
    let svg = format!(
        r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
            <rect width="100%" height="100%" fill="white"/>
            <circle cx="{}" cy="{}" r="{}" fill="{}"/>
            <text x="{}" y="{}" text-anchor="middle" font-size="20" fill="black">MCP Demo</text>
        </svg>"#,
        size, size,
        size / 2, size / 2, size / 3, color,
        size / 2, size / 2
    );

    // Convert SVG to base64
    let base64_svg = general_purpose::STANDARD.encode(svg.as_bytes());

    // Create the MCP tool response with image content
    let tool_response = ToolResponse {
        content: vec![
            ToolContent::text(format!("Generated a {} {} SVG image", color, size)),
            ToolContent::image(base64_svg, "image/svg+xml".to_string()),
        ],
        structured_content: None,
        is_error: None,
    };

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(serde_json::to_vec(&tool_response)?)
        .build())
}