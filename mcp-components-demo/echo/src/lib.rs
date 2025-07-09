use serde::{Deserialize, Serialize};
use spin_sdk::http::{IntoResponse, Request, Response};
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

#[http_component]
fn handle_echo(req: Request) -> anyhow::Result<impl IntoResponse> {
    // Only accept POST requests
    if *req.method() != spin_sdk::http::Method::Post {
        return Ok(Response::builder()
            .status(405)
            .header("Allow", "POST")
            .header("Content-Type", "application/json")
            .body(serde_json::to_vec(&ErrorResponse {
                error: "Method not allowed. Only POST is supported.".to_string(),
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

    // Create the response
    let response = EchoResponse {
        echo: request.message,
        received: true,
    };

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}