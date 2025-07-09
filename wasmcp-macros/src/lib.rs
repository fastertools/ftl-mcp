//! Procedural macros for the WASMCP SDK
//!
//! This crate provides macros that generate HTTP handler and tool registration
//! code automatically, allowing plugin authors to focus on business logic.

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, ItemImpl, Ident, Type, Meta, Lit};

/// Generate HTTP handler boilerplate for MCP plugins
/// 
/// This macro generates the complete HTTP handler function that:
/// - Validates HTTP method and path
/// - Parses JSON-RPC requests
/// - Delegates to the McpHandler trait implementation
/// - Converts responses back to HTTP format
/// 
/// Usage:
/// ```rust
/// #[mcp_plugin]
/// impl MyHandler {
///     // Your tool implementations here
/// }
/// ```
#[proc_macro_attribute]
pub fn mcp_plugin(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    
    // Extract the handler type name
    let handler_type = &input.self_ty;
    let handler_name = extract_type_name(handler_type);
    
    // Generate the HTTP handler function
    let handler_fn = generate_http_handler(&handler_name);
    
    // Generate the expanded implementation
    let expanded = quote! {
        #input
        
        #handler_fn
    };
    
    TokenStream::from(expanded)
}

/// Generate tool registration for MCP methods
/// 
/// This macro examines method signatures and generates the appropriate
/// tool registration code for the McpHandler trait.
/// 
/// Usage:
/// ```rust
/// #[mcp_tool("tool_name", "Tool description")]
/// async fn my_tool(&self, arg: String) -> Result<String> {
///     // Implementation
/// }
/// ```
#[proc_macro_attribute]
pub fn mcp_tool(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _attr_meta = parse_macro_input!(attr as Meta);
    let input = parse_macro_input!(item as syn::ItemFn);
    
    // For now, just return the original function
    // We'll implement tool registration generation in the next step
    let expanded = quote! {
        #input
    };
    
    TokenStream::from(expanded)
}

fn extract_type_name(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => {
            type_path.path.segments.last()
                .map(|seg| seg.ident.to_string())
                .unwrap_or_else(|| "Handler".to_string())
        }
        _ => "Handler".to_string(),
    }
}

fn generate_http_handler(handler_name: &str) -> proc_macro2::TokenStream {
    let handler_ident = Ident::new(handler_name, Span::call_site());
    
    quote! {
        #[spin_sdk::http_component]
        async fn handle_request(req: spin_sdk::http::Request) -> anyhow::Result<impl spin_sdk::http::IntoResponse> {
            use spin_sdk::http::{Method, Response};
            use wasmcp::{parse_jsonrpc_request, handle_jsonrpc_request, jsonrpc_to_http_response};
            
            println!("MCP_PLUGIN: Component started - handling request");
            println!("MCP_PLUGIN: Received request: method={}, path={}", req.method(), req.path());
            
            // Handle POST requests to our MCP endpoint
            if req.method() != &Method::Post || !req.path().ends_with("/mcp") {
                println!("MCP_PLUGIN: Request rejected: method={}, path={}", req.method(), req.path());
                return Ok(Response::builder()
                    .status(404)
                    .body("Not found")
                    .build());
            }

            // Parse JSON-RPC request
            let body = req.body();
            let json_req = match parse_jsonrpc_request(body) {
                Ok(req) => req,
                Err(e) => {
                    let error_response = wasmcp::build_jsonrpc_error(None, e);
                    return Ok(jsonrpc_to_http_response(error_response));
                }
            };

            println!("MCP_PLUGIN: Processing JSON-RPC request: method={}, id={:?}", json_req.method, json_req.id);

            // Handle the request using the SDK helper
            let handler = #handler_ident;
            let response = handle_jsonrpc_request(&handler, json_req).await;
            
            println!("MCP_PLUGIN: Generated response: {:?}", response);
            
            Ok(jsonrpc_to_http_response(response))
        }
    }
}

fn _extract_tool_info(_meta: &Meta) -> (String, String) {
    // For now, just return empty strings - we'll implement this later
    (String::new(), String::new())
}