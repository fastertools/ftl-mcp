use spin_test_sdk::{bindings::wasi::http, spin_test};

// Note: Test configuration is provided by spin-test.toml
// Auth is enabled with an AuthKit provider configured

#[spin_test]
fn unauthenticated_request() {
    // Make request without auth header
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request.set_path_with_query(Some("/mcp")).unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should return 401 Unauthorized
    assert_eq!(response.status(), 401);

    // Check for WWW-Authenticate header
    let headers = response.headers();
    let www_auth_exists = headers
        .entries()
        .iter()
        .any(|(name, _)| name == "www-authenticate");
    assert!(www_auth_exists);
}

#[spin_test]
fn options_cors_request() {
    // Make OPTIONS request (CORS preflight)
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request.set_method(&http::types::Method::Options).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should return 204 No Content
    assert_eq!(response.status(), 204);

    // Check for CORS headers
    let headers = response.headers();
    let has_cors = headers
        .entries()
        .iter()
        .any(|(name, _)| name == "access-control-allow-origin");
    assert!(has_cors);
}

#[spin_test]
fn metadata_endpoint() {
    // With the test configuration, we have a provider configured
    // Test /.well-known/oauth-protected-resource endpoint
    let headers = http::types::Headers::new();
    headers.append("host", b"example.com").unwrap();

    let request = http::types::OutgoingRequest::new(headers);
    request
        .set_path_with_query(Some("/.well-known/oauth-protected-resource"))
        .unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should return 200 when provider is configured
    assert_eq!(response.status(), 200);

    // Check for proper content type
    let headers = response.headers();
    let has_json_content = headers.entries().iter().any(|(name, value)| {
        name == "content-type" && String::from_utf8_lossy(value).contains("application/json")
    });
    assert!(has_json_content);
}

#[spin_test]
fn authorization_server_metadata() {
    // With the test configuration, we have a provider configured
    // Test /.well-known/oauth-authorization-server endpoint
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request
        .set_path_with_query(Some("/.well-known/oauth-authorization-server"))
        .unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should return 200 when provider is configured
    assert_eq!(response.status(), 200);

    // Check response contains OAuth metadata
    let headers = response.headers();
    let has_json_content = headers.entries().iter().any(|(name, value)| {
        name == "content-type" && String::from_utf8_lossy(value).contains("application/json")
    });
    assert!(has_json_content);
}

#[spin_test]
fn provider_config_works() {
    // Test that the provider configuration works correctly
    // Make request to metadata endpoint
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request
        .set_path_with_query(Some("/.well-known/oauth-authorization-server"))
        .unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should return 200 with configured provider
    assert_eq!(response.status(), 200);

    // Verify CORS headers are present
    let headers = response.headers();
    let has_cors = headers
        .entries()
        .iter()
        .any(|(name, _)| name == "access-control-allow-origin");
    assert!(has_cors);
}

#[spin_test]
fn trace_id_header() {
    // Test that trace ID is propagated through requests
    let headers = http::types::Headers::new();
    headers.append("x-trace-id", b"test-trace-123").unwrap();

    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should return 401
    assert_eq!(response.status(), 401);

    // Check for trace ID in response
    let response_headers = response.headers();
    let has_trace = response_headers
        .entries()
        .iter()
        .any(|(name, _)| name == "x-trace-id");
    assert!(has_trace);
}

#[spin_test]
fn auth_enabled_requires_token() {
    // With auth enabled in test config, requests without auth should fail
    // Make request without auth header
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request.set_path_with_query(Some("/mcp")).unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should return 401 because auth is required
    assert_eq!(response.status(), 401);

    // Check for WWW-Authenticate header
    let headers = response.headers();
    let www_auth_exists = headers
        .entries()
        .iter()
        .any(|(name, _)| name == "www-authenticate");
    assert!(www_auth_exists);
}

#[spin_test]
fn metadata_endpoint_with_provider() {
    // Test /.well-known/oauth-protected-resource endpoint
    let headers = http::types::Headers::new();
    headers.append("host", b"example.com").unwrap();

    let request = http::types::OutgoingRequest::new(headers);
    request
        .set_path_with_query(Some("/.well-known/oauth-protected-resource"))
        .unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should return 200 when provider is configured
    assert_eq!(response.status(), 200);

    // Check for content type
    let headers = response.headers();
    let has_content_type = headers
        .entries()
        .iter()
        .any(|(name, _)| name == "content-type");
    assert!(has_content_type);
}
