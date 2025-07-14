use spin_test_sdk::{
    bindings::{fermyon::spin_test_virt, wasi::http},
    spin_test,
};

/// Set up default test configuration
fn setup_test_config() {
    spin_test_virt::variables::set(
        "auth_config",
        r#"{
        "mcp_gateway_url": "http://test-gateway.internal/mcp-internal",
        "trace_id_header": "X-Trace-Id",
        "providers": [{
            "type": "authkit",
            "issuer": "https://test.authkit.app",
            "jwks_uri": "https://test.authkit.app/.well-known/jwks.json"
        }]
    }"#,
    );
}

#[spin_test]
fn unauthenticated_request() {
    // Set up test configuration
    setup_test_config();

    // Make request without auth header
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request.set_path_with_query(Some("/mcp")).unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should return 401 Unauthorized
    assert_eq!(response.status(), 401);
    
    // Check for WWW-Authenticate header
    let headers = response.headers();
    let www_auth_exists = headers.entries().iter().any(|(name, _)| name == "www-authenticate");
    assert!(www_auth_exists);
}

#[spin_test]
fn options_cors_request() {
    // Set up test configuration
    setup_test_config();

    // Make OPTIONS request (CORS preflight)
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request.set_method(&http::types::Method::Options).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should return 204 No Content
    assert_eq!(response.status(), 204);
    
    // Check for CORS headers
    let headers = response.headers();
    let has_cors = headers.entries().iter().any(|(name, _)| name == "access-control-allow-origin");
    assert!(has_cors);
}

#[spin_test]
fn metadata_endpoint() {
    // Set up test configuration
    setup_test_config();

    // Test /.well-known/oauth-protected-resource endpoint
    let headers = http::types::Headers::new();
    headers.append("host", b"example.com").unwrap();

    let request = http::types::OutgoingRequest::new(headers);
    request
        .set_path_with_query(Some("/.well-known/oauth-protected-resource"))
        .unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should return 500 when no providers configured
    assert_eq!(response.status(), 500);
}

#[spin_test]
fn authorization_server_metadata() {
    // Set up test configuration
    setup_test_config();

    // Test /.well-known/oauth-authorization-server endpoint
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request
        .set_path_with_query(Some("/.well-known/oauth-authorization-server"))
        .unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should return 500 when no providers configured
    assert_eq!(response.status(), 500);
}

#[spin_test]
fn empty_provider_config() {
    // Set empty provider list
    spin_test_virt::variables::set(
        "auth_config",
        r#"{
        "mcp_gateway_url": "http://test-gateway.internal/mcp-internal",
        "trace_id_header": "X-Trace-Id",
        "providers": []
    }"#,
    );

    // Make request to metadata endpoint
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request
        .set_path_with_query(Some("/.well-known/oauth-authorization-server"))
        .unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should return 500 when no providers configured
    assert_eq!(response.status(), 500);
}

#[spin_test]
fn trace_id_header() {
    // Set up test configuration
    setup_test_config();

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
    let has_trace = response_headers.entries().iter().any(|(name, _)| name == "x-trace-id");
    assert!(has_trace);
}