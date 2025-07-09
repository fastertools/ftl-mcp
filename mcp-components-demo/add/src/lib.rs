use serde::{Deserialize, Serialize};
use spin_sdk::http::{IntoResponse, Request, Response};
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
struct ErrorResponse {
    error: String,
}

#[http_component]
fn handle_add(req: Request) -> anyhow::Result<impl IntoResponse> {
    // Parse the request body
    let request: AddRequest = match serde_json::from_slice(req.body()) {
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

    // Perform the addition
    let result = request.a + request.b;

    // Create the response
    let response = AddResponse {
        result,
        operation: format!("{} + {} = {}", request.a, request.b, result),
    };

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}