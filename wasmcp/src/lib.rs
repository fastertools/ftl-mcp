//! WASMCP SDK - Model Context Protocol SDK for Spin WebAssembly
//! 
//! This crate provides types and traits for building MCP plugins on Spin.

pub mod types;
pub mod traits;
pub mod helpers;
pub mod errors;

// Re-export commonly used types
pub use types::*;
pub use traits::*;
pub use helpers::*;
pub use errors::*;

// Re-export procedural macros
pub use wasmcp_macros::*;

// Re-export dependencies that plugins will need
pub use serde_json;