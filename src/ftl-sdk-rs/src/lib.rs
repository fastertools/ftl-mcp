//! Thin SDK providing MCP protocol types for FTL tool development.
//!
//! This crate provides only the type definitions needed to implement
//! MCP-compliant tools. It does not include any HTTP server logic,
//! allowing you to use any web framework of your choice.

// Re-export macros when the feature is enabled
#[cfg(feature = "macros")]
pub use ftl_sdk_macros::tool;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Tool metadata returned by GET requests to tool endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// The name of the tool (must be unique within the gateway)
    pub name: String,

    /// Optional human-readable title for the tool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Optional description of what the tool does
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// JSON Schema describing the expected input parameters
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,

    /// Optional JSON Schema describing the output format
    #[serde(rename = "outputSchema", skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<Value>,

    /// Optional annotations providing hints about tool behavior
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ToolAnnotations>,

    /// Optional metadata for tool-specific extensions
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
}

/// Annotations providing hints about tool behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolAnnotations {
    /// Optional title annotation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Hint that the tool is read-only (doesn't modify state)
    #[serde(rename = "readOnlyHint", skip_serializing_if = "Option::is_none")]
    pub read_only_hint: Option<bool>,

    /// Hint that the tool may perform destructive operations
    #[serde(rename = "destructiveHint", skip_serializing_if = "Option::is_none")]
    pub destructive_hint: Option<bool>,

    /// Hint that the tool is idempotent (same input → same output)
    #[serde(rename = "idempotentHint", skip_serializing_if = "Option::is_none")]
    pub idempotent_hint: Option<bool>,

    /// Hint that the tool accepts open-world inputs
    #[serde(rename = "openWorldHint", skip_serializing_if = "Option::is_none")]
    pub open_world_hint: Option<bool>,
}

/// Response format for tool execution (POST requests)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse {
    /// Array of content items returned by the tool
    pub content: Vec<ToolContent>,

    /// Optional structured content matching the outputSchema
    #[serde(rename = "structuredContent", skip_serializing_if = "Option::is_none")]
    pub structured_content: Option<Value>,

    /// Indicates if this response represents an error
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// Content types that can be returned by tools
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    /// Text content
    #[serde(rename = "text")]
    Text {
        /// The text content
        text: String,
        /// Optional annotations for this content
        #[serde(skip_serializing_if = "Option::is_none")]
        annotations: Option<ContentAnnotations>,
    },

    /// Image content
    #[serde(rename = "image")]
    Image {
        /// Base64-encoded image data
        data: String,
        /// MIME type of the image (e.g., "image/png")
        #[serde(rename = "mimeType")]
        mime_type: String,
        /// Optional annotations for this content
        #[serde(skip_serializing_if = "Option::is_none")]
        annotations: Option<ContentAnnotations>,
    },

    /// Audio content
    #[serde(rename = "audio")]
    Audio {
        /// Base64-encoded audio data
        data: String,
        /// MIME type of the audio (e.g., "audio/wav")
        #[serde(rename = "mimeType")]
        mime_type: String,
        /// Optional annotations for this content
        #[serde(skip_serializing_if = "Option::is_none")]
        annotations: Option<ContentAnnotations>,
    },

    /// Resource reference
    #[serde(rename = "resource")]
    Resource {
        /// The resource contents
        resource: ResourceContents,
        /// Optional annotations for this content
        #[serde(skip_serializing_if = "Option::is_none")]
        annotations: Option<ContentAnnotations>,
    },
}

/// Annotations for content items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAnnotations {
    /// Target audience for this content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<Vec<String>>,

    /// Priority of this content (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<f32>,
}

/// Resource contents for resource-type content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContents {
    /// URI of the resource
    pub uri: String,

    /// MIME type of the resource
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,

    /// Text content of the resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    /// Base64-encoded binary content of the resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
}

// Convenience constructors
impl ToolResponse {
    /// Create a simple text response
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::Text {
                text: text.into(),
                annotations: None,
            }],
            structured_content: None,
            is_error: None,
        }
    }

    /// Create an error response
    pub fn error(error: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::Text {
                text: error.into(),
                annotations: None,
            }],
            structured_content: None,
            is_error: Some(true),
        }
    }

    /// Create a response with structured content
    pub fn with_structured(text: impl Into<String>, structured: Value) -> Self {
        Self {
            content: vec![ToolContent::Text {
                text: text.into(),
                annotations: None,
            }],
            structured_content: Some(structured),
            is_error: None,
        }
    }
}

impl ToolContent {
    /// Create a text content item
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text {
            text: text.into(),
            annotations: None,
        }
    }

    /// Create an image content item
    pub fn image(data: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self::Image {
            data: data.into(),
            mime_type: mime_type.into(),
            annotations: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_tool_response_text() {
        let response = ToolResponse::text("Hello, world!");
        assert_eq!(response.content.len(), 1);
        assert!(response.is_error.is_none());
    }

    #[test]
    fn test_tool_response_error() {
        let response = ToolResponse::error("Something went wrong");
        assert_eq!(response.is_error, Some(true));
    }

    #[test]
    fn test_serialization() {
        let metadata = ToolMetadata {
            name: "test-tool".to_string(),
            title: Some("Test Tool".to_string()),
            description: None,
            input_schema: json!({
                "type": "object",
                "properties": {
                    "input": { "type": "string" }
                }
            }),
            output_schema: None,
            annotations: None,
            meta: None,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"name\":\"test-tool\""));
        assert!(json.contains("\"title\":\"Test Tool\""));
        assert!(!json.contains("\"description\""));
    }
}
