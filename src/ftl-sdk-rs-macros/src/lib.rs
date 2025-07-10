use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, ReturnType};

/// A procedural macro that transforms a simple function into a Spin HTTP component
/// that serves as an MCP tool.
///
/// # Example
///
/// ```rust
/// use ftl_sdk::{ToolMetadata, ToolResponse};
/// use ftl_sdk_macros::tool_component;
/// use serde::Deserialize;
/// use serde_json::json;
///
/// #[derive(Deserialize)]
/// struct EchoRequest {
///     message: String,
/// }
///
/// #[tool_component(
///     metadata = ToolMetadata {
///         name: "echo".to_string(),
///         title: Some("Echo Tool".to_string()),
///         description: Some("Echoes back the input message".to_string()),
///         input_schema: json!({
///             "type": "object",
///             "properties": {
///                 "message": { "type": "string" }
///             },
///             "required": ["message"]
///         }),
///         output_schema: None,
///         annotations: None,
///         meta: None,
///     }
/// )]
/// fn echo(req: EchoRequest) -> ToolResponse {
///     ToolResponse::text(format!("Echo: {}", req.message))
/// }
/// ```
///
/// This will generate the necessary boilerplate to handle HTTP requests:
/// - GET requests return the tool metadata
/// - POST requests deserialize the body and call your function
#[proc_macro_attribute]
pub fn tool_component(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    
    // Parse the metadata argument
    let args = proc_macro2::TokenStream::from(args);
    
    // Extract function details
    let fn_name = &input_fn.sig.ident;
    let fn_visibility = &input_fn.vis;
    
    // Get the input type from the function argument
    let input_type = match input_fn.sig.inputs.first() {
        Some(FnArg::Typed(pat_type)) => &pat_type.ty,
        _ => {
            return syn::Error::new_spanned(
                &input_fn.sig,
                "Function must have exactly one argument"
            )
            .to_compile_error()
            .into();
        }
    };
    
    // Ensure the function returns ToolResponse
    match &input_fn.sig.output {
        ReturnType::Type(_, ty) => {
            let ty_str = quote!(#ty).to_string();
            if !ty_str.contains("ToolResponse") {
                return syn::Error::new_spanned(
                    &input_fn.sig.output,
                    "Function must return ToolResponse"
                )
                .to_compile_error()
                .into();
            }
        }
        ReturnType::Default => {
            return syn::Error::new_spanned(
                &input_fn.sig,
                "Function must return ToolResponse"
            )
            .to_compile_error()
            .into();
        }
    }
    
    // Generate the HTTP handler
    let output = quote! {
        #input_fn
        
        #[::spin_sdk::http_component]
        #fn_visibility fn handle_tool_component(req: ::spin_sdk::http::Request) -> ::spin_sdk::http::Response {
            use ::spin_sdk::http::{Method, Response};
            
            // Define metadata
            let metadata = {
                #args
            };
            
            match req.method() {
                &Method::Get => {
                    // Return tool metadata
                    match ::serde_json::to_vec(&metadata) {
                        Ok(body) => Response::builder()
                            .status(200)
                            .header("Content-Type", "application/json")
                            .body(body)
                            .build(),
                        Err(e) => Response::builder()
                            .status(500)
                            .body(format!("Failed to serialize metadata: {}", e))
                            .build()
                    }
                }
                &Method::Post => {
                    // Parse request body and execute tool
                    let body = req.body();
                    match ::serde_json::from_slice::<#input_type>(body) {
                        Ok(input) => {
                            let response = #fn_name(input);
                            match ::serde_json::to_vec(&response) {
                                Ok(body) => Response::builder()
                                    .status(200)
                                    .header("Content-Type", "application/json")
                                    .body(body)
                                    .build(),
                                Err(e) => {
                                    let error_response = ::ftl_sdk::ToolResponse::error(
                                        format!("Failed to serialize response: {}", e)
                                    );
                                    Response::builder()
                                        .status(500)
                                        .header("Content-Type", "application/json")
                                        .body(::serde_json::to_vec(&error_response).unwrap_or_default())
                                        .build()
                                }
                            }
                        }
                        Err(e) => {
                            let error_response = ::ftl_sdk::ToolResponse::error(
                                format!("Invalid request body: {}", e)
                            );
                            Response::builder()
                                .status(400)
                                .header("Content-Type", "application/json")
                                .body(::serde_json::to_vec(&error_response).unwrap_or_default())
                                .build()
                        }
                    }
                }
                _ => Response::builder()
                    .status(405)
                    .header("Allow", "GET, POST")
                    .body("Method not allowed")
                    .build()
            }
        }
    };
    
    output.into()
}

/// A simpler version of tool_component that builds metadata from attributes
///
/// # Example
///
/// ```rust
/// use ftl_sdk::ToolResponse;
/// use ftl_sdk_macros::tool;
/// use serde::Deserialize;
/// use serde_json::json;
///
/// #[derive(Deserialize)]
/// struct AddRequest {
///     a: i32,
///     b: i32,
/// }
///
/// /// Adds two numbers together
/// #[tool(
///     input_schema = json!({
///         "type": "object",
///         "properties": {
///             "a": { "type": "integer" },
///             "b": { "type": "integer" }
///         },
///         "required": ["a", "b"]
///     })
/// )]
/// fn add(req: AddRequest) -> ToolResponse {
///     let result = req.a + req.b;
///     ToolResponse::text(format!("{} + {} = {}", req.a, req.b, result))
/// }
/// ```
///
/// The macro will:
/// - Use the function name as the tool name (unless overridden)
/// - Extract the first line of the doc comment as the description (unless overridden)
/// - Generate the title from the function name (unless overridden)
#[proc_macro_attribute]
pub fn tool(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    
    // Parse the arguments to extract name, title, description
    let args_parsed = match syn::parse::<ToolArgs>(args) {
        Ok(args) => args,
        Err(e) => return e.to_compile_error().into(),
    };
    
    // Get the input type to derive the schema
    let input_type = match input_fn.sig.inputs.first() {
        Some(FnArg::Typed(pat_type)) => &pat_type.ty,
        _ => {
            return syn::Error::new_spanned(
                &input_fn.sig,
                "Function must have exactly one argument"
            )
            .to_compile_error()
            .into();
        }
    };
    
    let fn_name = &input_fn.sig.ident;
    let fn_visibility = &input_fn.vis;
    
    // Extract doc comments from the function
    let doc_comment = extract_doc_comment(&input_fn.attrs);
    
    // Use provided values or fall back to defaults
    let name = args_parsed.name.unwrap_or_else(|| fn_name.to_string());
    let title = args_parsed.title
        .map(|s| quote!(Some(#s.to_string())))
        .unwrap_or_else(|| {
            let title = generate_title(&fn_name.to_string());
            quote!(Some(#title.to_string()))
        });
    let description = args_parsed.description
        .map(|s| quote!(Some(#s.to_string())))
        .unwrap_or_else(|| {
            if let Some(doc) = doc_comment {
                quote!(Some(#doc.to_string()))
            } else {
                quote!(None)
            }
        });
    let input_schema = args_parsed.input_schema;
    
    // For now, we'll use a generic object schema
    // In a more advanced version, we could derive this from the input type
    let output = quote! {
        #input_fn
        
        #[::spin_sdk::http_component]
        #fn_visibility fn handle_tool_component(req: ::spin_sdk::http::Request) -> ::spin_sdk::http::Response {
            use ::spin_sdk::http::{Method, Response};
            
            // Build metadata
            let metadata = ::ftl_sdk::ToolMetadata {
                name: #name.to_string(),
                title: #title,
                description: #description,
                input_schema: #input_schema,
                output_schema: None,
                annotations: None,
                meta: None,
            };
            
            match req.method() {
                &Method::Get => {
                    // Return tool metadata
                    match ::serde_json::to_vec(&metadata) {
                        Ok(body) => Response::builder()
                            .status(200)
                            .header("Content-Type", "application/json")
                            .body(body)
                            .build(),
                        Err(e) => Response::builder()
                            .status(500)
                            .body(format!("Failed to serialize metadata: {}", e))
                            .build()
                    }
                }
                &Method::Post => {
                    // Parse request body and execute tool
                    let body = req.body();
                    match ::serde_json::from_slice::<#input_type>(body) {
                        Ok(input) => {
                            let response = #fn_name(input);
                            match ::serde_json::to_vec(&response) {
                                Ok(body) => Response::builder()
                                    .status(200)
                                    .header("Content-Type", "application/json")
                                    .body(body)
                                    .build(),
                                Err(e) => {
                                    let error_response = ::ftl_sdk::ToolResponse::error(
                                        format!("Failed to serialize response: {}", e)
                                    );
                                    Response::builder()
                                        .status(500)
                                        .header("Content-Type", "application/json")
                                        .body(::serde_json::to_vec(&error_response).unwrap_or_default())
                                        .build()
                                }
                            }
                        }
                        Err(e) => {
                            let error_response = ::ftl_sdk::ToolResponse::error(
                                format!("Invalid request body: {}", e)
                            );
                            Response::builder()
                                .status(400)
                                .header("Content-Type", "application/json")
                                .body(::serde_json::to_vec(&error_response).unwrap_or_default())
                                .build()
                        }
                    }
                }
                _ => Response::builder()
                    .status(405)
                    .header("Allow", "GET, POST")
                    .body("Method not allowed")
                    .build()
            }
        }
    };
    
    output.into()
}

// Helper struct to parse tool macro arguments
struct ToolArgs {
    name: Option<String>,
    title: Option<String>,
    description: Option<String>,
    input_schema: proc_macro2::TokenStream,
}

// Extract the first line of doc comments from attributes
fn extract_doc_comment(attrs: &[syn::Attribute]) -> Option<String> {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                match &attr.meta {
                    syn::Meta::NameValue(nv) => {
                        if let syn::Expr::Lit(lit) = &nv.value {
                            if let syn::Lit::Str(s) = &lit.lit {
                                let doc = s.value();
                                // Trim leading space that rustdoc adds
                                return Some(doc.trim_start_matches(' ').to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
            None
        })
        .next()
}

// Generate a title from a function name (e.g., "calculate_sum" -> "Calculate Sum")
fn generate_title(name: &str) -> String {
    name.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

impl syn::parse::Parse for ToolArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut title = None;
        let mut description = None;
        let mut input_schema = None;
        
        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            input.parse::<syn::Token![=]>()?;
            
            match ident.to_string().as_str() {
                "name" => {
                    let lit: syn::LitStr = input.parse()?;
                    name = Some(lit.value());
                }
                "title" => {
                    let lit: syn::LitStr = input.parse()?;
                    title = Some(lit.value());
                }
                "description" => {
                    let lit: syn::LitStr = input.parse()?;
                    description = Some(lit.value());
                }
                "input_schema" => {
                    let expr: syn::Expr = input.parse()?;
                    input_schema = Some(quote!(#expr));
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        ident,
                        "Unknown attribute. Expected: name, title, description, or input_schema"
                    ));
                }
            }
            
            if !input.is_empty() {
                input.parse::<syn::Token![,]>()?;
            }
        }
        
        let input_schema = input_schema.ok_or_else(|| {
            syn::Error::new(input.span(), "Missing required attribute: input_schema")
        })?;
        
        Ok(ToolArgs {
            name,
            title,
            description,
            input_schema,
        })
    }
}