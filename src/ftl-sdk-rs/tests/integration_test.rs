use ftl_sdk::*;
use pretty_assertions::assert_eq;
use serde_json::json;

#[test]
fn test_tool_metadata_serialization() {
    let metadata = ToolMetadata {
        name: "test-tool".to_string(),
        title: Some("Test Tool".to_string()),
        description: Some("A tool for testing".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "input": { "type": "string" }
            },
            "required": ["input"]
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "result": { "type": "string" }
            }
        })),
        annotations: Some(ToolAnnotations {
            title: None,
            read_only_hint: Some(true),
            destructive_hint: Some(false),
            idempotent_hint: Some(true),
            open_world_hint: None,
        }),
        meta: None,
    };

    let serialized = serde_json::to_value(&metadata).unwrap();

    // Check field names are properly transformed
    assert!(serialized.get("name").is_some());
    assert!(serialized.get("inputSchema").is_some());
    assert!(serialized.get("outputSchema").is_some());

    // Check nested annotations
    let annotations = serialized.get("annotations").unwrap();
    assert!(annotations.get("readOnlyHint").is_some());
    assert_eq!(annotations.get("readOnlyHint").unwrap(), &json!(true));
}

#[test]
fn test_tool_response_convenience_methods() {
    // Test text response
    let text_response = ToolResponse::text("Hello, world!");
    assert_eq!(text_response.content.len(), 1);
    match &text_response.content[0] {
        ToolContent::Text { text, .. } => assert_eq!(text, "Hello, world!"),
        _ => panic!("Expected text content"),
    }
    assert!(text_response.is_error.is_none());

    // Test error response
    let error_response = ToolResponse::error("Something went wrong");
    assert_eq!(error_response.is_error, Some(true));
    match &error_response.content[0] {
        ToolContent::Text { text, .. } => assert_eq!(text, "Something went wrong"),
        _ => panic!("Expected text content"),
    }

    // Test structured response
    let structured_data = json!({ "result": 42, "status": "success" });
    let structured_response =
        ToolResponse::with_structured("Operation complete", structured_data.clone());
    assert_eq!(
        structured_response.structured_content,
        Some(structured_data)
    );
}

#[test]
fn test_tool_content_serialization() {
    let text_content = ToolContent::Text {
        text: "Sample text".to_string(),
        annotations: Some(ContentAnnotations {
            audience: Some(vec!["developers".to_string()]),
            priority: Some(0.8),
        }),
    };

    let serialized = serde_json::to_value(&text_content).unwrap();
    assert_eq!(serialized.get("type").unwrap(), "text");
    assert_eq!(serialized.get("text").unwrap(), "Sample text");

    let annotations = serialized.get("annotations").unwrap();
    assert_eq!(annotations.get("audience").unwrap(), &json!(["developers"]));
}

#[test]
fn test_image_content() {
    let image = ToolContent::image("base64data", "image/png");
    let serialized = serde_json::to_value(&image).unwrap();

    assert_eq!(serialized.get("type").unwrap(), "image");
    assert_eq!(serialized.get("data").unwrap(), "base64data");
    assert_eq!(serialized.get("mimeType").unwrap(), "image/png");
}

#[test]
fn test_resource_content() {
    let resource = ToolContent::Resource {
        resource: ResourceContents {
            uri: "file:///example.txt".to_string(),
            mime_type: Some("text/plain".to_string()),
            text: Some("File contents".to_string()),
            blob: None,
        },
        annotations: None,
    };

    let serialized = serde_json::to_value(&resource).unwrap();
    assert_eq!(serialized.get("type").unwrap(), "resource");

    let resource_data = serialized.get("resource").unwrap();
    assert_eq!(resource_data.get("uri").unwrap(), "file:///example.txt");
    assert_eq!(resource_data.get("mimeType").unwrap(), "text/plain");
}

#[test]
fn test_optional_fields_are_excluded() {
    let minimal_metadata = ToolMetadata {
        name: "minimal".to_string(),
        title: None,
        description: None,
        input_schema: json!({}),
        output_schema: None,
        annotations: None,
        meta: None,
    };

    let serialized = serde_json::to_value(&minimal_metadata).unwrap();

    // These fields should not be present when None
    assert!(serialized.get("title").is_none());
    assert!(serialized.get("description").is_none());
    assert!(serialized.get("outputSchema").is_none());
    assert!(serialized.get("annotations").is_none());
    assert!(serialized.get("_meta").is_none());
}

#[test]
fn test_round_trip_serialization() {
    let original = ToolResponse {
        content: vec![
            ToolContent::Text {
                text: "First item".to_string(),
                annotations: None,
            },
            ToolContent::Image {
                data: "imagedata".to_string(),
                mime_type: "image/jpeg".to_string(),
                annotations: Some(ContentAnnotations {
                    audience: None,
                    priority: Some(0.5),
                }),
            },
        ],
        structured_content: Some(json!({ "complex": { "nested": "data" } })),
        is_error: Some(false),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&original).unwrap();

    // Deserialize back
    let deserialized: ToolResponse = serde_json::from_str(&json).unwrap();

    // Compare
    assert_eq!(original.content.len(), deserialized.content.len());
    assert_eq!(original.structured_content, deserialized.structured_content);
    assert_eq!(original.is_error, deserialized.is_error);
}
