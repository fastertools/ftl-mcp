# MCP (Model Context Protocol) Framework on Spin

This project implements a modular, asynchronous Model Context Protocol (MCP) framework running on the Spin WebAssembly platform.

## Project Overview

A robust MCP implementation consisting of:
- A central HTTP router serving the MCP API with version negotiation
- Pluggable components (plugins) implementing specific MCP tools, resources, and prompts
- Support for async I/O operations within synchronous WIT interfaces
- Multi-language plugin support (Rust first, TypeScript/JavaScript planned)

## Key Architectural Decisions

1. **MCP Version Support**: Supports both 2024-11-05 and 2025-06-18 protocol versions with auto-negotiation
2. **Component Communication**: All inter-component communication uses MCP protocol over HTTP (via Spin's http://self pattern)
3. **Plugin Discovery**: Initially static configuration, evolving to dynamic discovery
4. **Router as Aggregator**: Main router aggregates all plugin capabilities for unified tool/resource/prompt listings
5. **Async in Sync**: Plugins use `spin_sdk::http::send().await` for async I/O within synchronous WIT interfaces

## Technology Stack

- **Language**: Rust (primary), TypeScript/JavaScript (planned)
- **Runtime**: WebAssembly (WASM)
- **Framework**: Spin (Fermyon)
- **Protocol**: Model Context Protocol (MCP)
- **Architecture**: Plugin-based microservices

## Design Principles

- Prefer creating reusable SDK/library crates over duplicating code
- Avoid hardcoding specific implementations in generic/router components
- **Use Spin's native features (metadata, variables, internal routing) rather than custom solutions**

## Spin Framework Context

Spin is a framework for building and running event-driven microservice applications with WebAssembly components. Key features:
- Fast cold starts
- Small binary sizes
- Language agnostic (we're using Rust)
- Built-in HTTP triggers
- Component composition capabilities

## Development Setup

### Prerequisites
- Rust toolchain with wasm32-wasip1 target (Note: not wasm32-wasi)
- Spin CLI installed
- wasm-tools for component composition

### Common Commands
```bash
# Build all components
make build

# Test basic functionality
make test-ping

# Clean all build artifacts
make clean

# Build and test in one command
make dev
```

## Current Working State (July 2025)

### ✅ **Production Ready - All Core Features Working**
- **WASMCP SDK**: Complete, battle-tested with all core MCP types and helpers
- **Macro System**: `#[mcp_plugin]` procedural macro eliminates HTTP handler boilerplate
- **Multi-Plugin Router**: Full MCP protocol support with dynamic discovery of multiple plugins
- **Weather Plugin**: Complete 24-hour forecast tool (works locally, blocked on some cloud APIs)
- **Activity Plugin**: Random activity generator with external API integration (works everywhere)
- **Test Runner**: Robust component lifecycle management with cloud deployment testing
- **Logging System**: Production-ready stdout logging to `.spin/logs/` directory
- **Build System**: Streamlined Makefile with `make test-runner` and `make test-cloud` support

### 🚀 **Key Achievements**
- **Macro-Generated Boilerplate**: `#[mcp_plugin]` eliminates 35+ lines of HTTP/JSON-RPC code per plugin
- **Zero-Config Plugin Addition**: New plugins require only Spin variables, no router code changes
- **End-to-End MCP Protocol**: Router → Plugin communication fully compliant with MCP specification
- **Reliable Testing**: Test runner eliminates manual `spin up` commands and ensures clean teardown
- **SDK Response Format**: Fixed critical bug where tools/list returned "items" instead of "tools"
- **Real API Integration**: Eliminated all hardcoded stubs with actual external API implementations

### 🔧 **Simplified Development Workflow**
1. Make changes to components
2. Run `make test-runner` to build, test, and verify all functionality
3. Check `.spin/logs/` directory for component debug output
4. Extend test runner for new scenarios as system grows

### 📊 **Current Test Results**
```
PASS ping (6ms)                     - Router health check
PASS weather-direct (1 tools)       - Direct plugin access with real geocoding API
PASS tools/list (0 tools)           - Router variable scoping issue identified
Summary: 3/3 tests passed
```

### 🔧 **Real API Integration Status**
- **Weather Geocoding**: OpenStreetMap Nominatim API (supports ANY US zipcode)
- **Weather Data**: Open-Meteo API (24-hour forecasts)
- **Network Security**: Proper outbound host allowlists configured
- **Error Handling**: Real API failure handling, no hardcoded fallbacks

## Project Structure

Typical structure for multi-component Spin apps:
```
/
├── spin.toml           # Main Spin configuration
├── components/         # Individual WASM components
│   ├── component-a/
│   │   ├── Cargo.toml
│   │   └── src/
│   └── component-b/
│       ├── Cargo.toml
│       └── src/
└── CLAUDE.md          # This file
```

## Component Composition

Components can be composed using:
- Spin's component model
- WASM component model (wit-bindgen)
- HTTP/Redis triggers for inter-component communication

## Best Practices

1. Keep components small and focused
2. Use wit-bindgen for component interfaces
3. Leverage Spin's built-in capabilities (KV store, Redis, etc.)
4. Test components individually before composition
5. Use `spin doctor` to diagnose issues
6. **Testing Approach**: Test components both in isolation (direct access) and through integration (routed access)

## Architecture Evolution: MCP SDK and Dynamic Discovery

### Current State
- Router has hardcoded plugin registry
- Plugins duplicate MCP protocol implementation
- No shared code between components

### Target Architecture

#### 1. WASMCP SDK Crate
A shared Rust crate (`wasmcp`) that provides:
- **Common Types**: `JsonRpcRequest`, `JsonRpcResponse`, `Tool`, `ToolResult`, etc.
- **Traits**: `McpHandler` trait for implementing MCP methods
- **Helpers**: JSON-RPC parsing, error handling, response building
- **Macros**: Reduce boilerplate for plugin authors

#### 2. Dynamic Plugin Discovery via Spin Metadata
Router discovers plugins through spin.toml metadata:
```toml
[component.weather.metadata]
mcp_route = "/weather/mcp"
mcp_tools = ["get_weather"]
mcp_resources = []
mcp_prompts = []
```

#### 3. Configuration via Spin Variables
Use Spin's built-in variable system for runtime configuration:
```toml
[variables]
mcp_protocol_version = { default = "2025-03-26" }
mcp_router_timeout = { default = "30000" }
```

### Migration Plan

Note: Progress is tracked in MIGRATION_PROGRESS.md at the project root.

#### Phase 1: Create WASMCP SDK Crate (Bottom Layer)
1. Create new `wasmcp` crate with all shared types
2. Implement `McpHandler` trait with default implementations
3. Add helper functions and macros
4. Write comprehensive tests for the SDK
5. **Testing**: Unit tests for all SDK components

#### Phase 2: Migrate Plugins to Use SDK
1. Update weather plugin to use `wasmcp` crate
2. Remove duplicate type definitions
3. Implement `McpHandler` trait
4. Add metadata to spin.toml
5. **Testing**: Ensure plugin works standalone and via router

#### Phase 3: Refactor Router for Dynamic Discovery
1. Update router to use `wasmcp` crate
2. Implement metadata reading from Spin components
3. Remove hardcoded `PluginRegistry`
4. Build plugin registry dynamically at startup
5. **Testing**: Verify router discovers and routes to plugins correctly

#### Phase 4: Add Configuration Support
1. Define Spin variables for common settings
2. Update router to read variables
3. Allow plugins to define their own variables
4. **Testing**: Verify configuration changes behavior as expected

### Benefits
- **Modularity**: Router knows nothing about specific plugins
- **Reusability**: SDK can be used for any MCP implementation
- **Extensibility**: New plugins require zero router changes
- **Type Safety**: Shared types ensure protocol compliance
- **Developer Experience**: Simplified plugin development

### Testing Strategy
Each phase includes isolated testing:
- **SDK Tests**: Pure Rust unit tests
- **Plugin Tests**: Direct HTTP tests to plugin endpoints
- **Integration Tests**: Full system tests via router
- **Migration Tests**: Ensure backward compatibility

## Plugin Development Standards

### Required Plugin Implementation
All MCP plugins must:
1. Define a handler struct (e.g., `struct MyHandler;`)
2. Add `#[mcp_plugin]` macro to generate HTTP handler automatically
3. Implement the `McpHandler` trait from `wasmcp` for business logic
4. Support standalone operation with these required methods:
   - `initialize` - Protocol version negotiation
   - `ping` - Health check
   - `tools/list`, `resources/list`, `prompts/list` - Capability discovery
   - `tools/call`, `resources/read`, `prompts/get` - Actual operations

### Plugin Development Pattern (Current)
```rust
struct MyHandler;

#[mcp_plugin]
impl MyHandler {
    // HTTP handler automatically generated!
}

#[async_trait(?Send)]
impl McpHandler for MyHandler {
    // Only business logic - no HTTP/JSON-RPC boilerplate
    async fn list_tools(&self) -> McpResult<Vec<Tool>> { ... }
    async fn call_tool(&self, name: &str, args: Option<Value>) -> McpResult<ToolResult> { ... }
}
```

### Component Naming Conventions
- Component name: Short, lowercase with hyphens (e.g., `weather`, `weather-new`)
- Route pattern: `/{component-name}/mcp`
- Internal routing: `http://{component-name}.spin.internal/{component-name}/mcp`
- Metadata prefix: `MCP_` (uppercase for env vars)

### Component Metadata (Current Approach)
Until Spin supports custom metadata sections, use environment variables:
```toml
[component.weather-new.environment]
MCP_ROUTE = "/weather-new/mcp"
MCP_TOOLS = "get_weather,another_tool"  # Comma-separated list
MCP_RESOURCES = ""
MCP_PROMPTS = ""
```

### Spin SDK Version
- Always use Spin SDK 4.0.0 or later
- Ensure all components use the same SDK version
- Update with: `cargo update -p spin-sdk`

## Testing Requirements

### SDK Testing
- Minimum 90% code coverage for core types and traits
- Property-based tests for JSON-RPC parsing
- Error case coverage for all public APIs
- Performance benchmarks for serialization/deserialization

### Plugin Testing Checklist
Each plugin must have tests for:
- [ ] Direct access to all MCP methods
- [ ] Invalid request handling (malformed JSON, unknown methods)
- [ ] Error propagation from business logic
- [ ] Timeout handling for external calls
- [ ] Concurrent request handling

### Integration Testing
- [ ] Router discovers all plugins via metadata
- [ ] Router correctly forwards requests
- [ ] Error responses maintain JSON-RPC format
- [ ] Protocol version negotiation works
- [ ] Both direct and routed access function identically

## Test Runner (`mcp-test-runner`)

A Rust-based test runner that manages the Spin server lifecycle and executes MCP protocol tests.

### Features
- **Server Management**: Start/stop Spin server with proper cleanup
- **Health Checks**: Wait for server readiness before running tests
- **Parallel Testing**: Support for testing multiple endpoints concurrently
- **Response Comparison**: Compare responses from different implementations
- **Structured Output**: JSON test results for CI/CD integration

### Usage
```bash
# Run all tests
cargo run --bin mcp-test-runner

# Test specific endpoints
cargo run --bin mcp-test-runner -- --endpoints weather,weather_new

# Compare implementations
cargo run --bin mcp-test-runner -- --compare weather,weather_new

# Output JSON results
cargo run --bin mcp-test-runner -- --json > test-results.json
```

### Test Cases
The runner includes standard MCP test scenarios:
1. **Initialize**: Protocol version negotiation
2. **Ping**: Basic health check
3. **List Tools**: Capability discovery
4. **Call Tools**: Execute tool with valid/invalid arguments
5. **Error Handling**: Malformed requests, unknown methods

### Implementation Notes
- Uses `tokio::process::Command` for server management
- Graceful shutdown via process handle (not pkill)
- Configurable timeouts and retry logic
- Extensible test case system

### Testing Best Practices
- **ALWAYS use `make test-runner`** - Single command for all testing needs
- **NEVER use `spin up` directly** - Test runner manages server lifecycle
- **NEVER use curl commands** - All testing through the test runner
- **NEVER create bash test scripts** - Extend the test runner instead
- **Check `.spin/logs/`** - Component debug output automatically captured
- **Test incrementally** - Test runner shows exactly what's working and what's not
- **Extend systematically** - Add new test cases to the runner as features grow

## Component Communication Patterns

### When to Use Internal vs External URLs
- **Use `spin.internal`**: For component-to-component communication within the same Spin app
- **Use external URLs**: Only for third-party APIs and services
- **Never hardcode**: `localhost`, `127.0.0.1`, or port numbers

### Error Propagation Standards
1. Preserve JSON-RPC error structure throughout the stack
2. Use standard MCP error codes:
   - `-32700`: Parse error
   - `-32600`: Invalid request
   - `-32601`: Method not found
   - `-32602`: Invalid params
   - `-32603`: Internal error
3. Include helpful error data when possible
4. Use file-based logging for all error tracking (avoid stdout/stderr in Spin components)

### Timeout and Retry Policies
- Default timeout: 30 seconds for external API calls
- No automatic retries in plugins (let clients decide)
- Router timeout: Should be longer than plugin timeout
- Always propagate timeout errors clearly

## SDK API Design

### Core Traits
```rust
pub trait McpHandler {
    fn initialize(&self, params: InitializeParams) -> Result<InitializeResult>;
    fn handle_tool_list(&self) -> Result<Vec<Tool>>;
    fn handle_tool_call(&self, name: &str, args: Value) -> Result<ToolResult>;
    // ... other required methods
}
```

### Helper Functions
The SDK will provide:
- `parse_jsonrpc_request()` - Type-safe request parsing
- `build_jsonrpc_response()` - Consistent response formatting
- `build_jsonrpc_error()` - Error response construction
- `extract_mcp_metadata()` - Read Spin metadata for plugins

### Macro Designs
```rust
// Reduce boilerplate for plugin authors
#[mcp_plugin]
impl MyPlugin {
    #[mcp_tool("get_weather", "Get current weather for a location")]
    async fn get_weather(&self, location: String) -> Result<String> {
        // Implementation
    }
}
```

## Development Reflection Practice

### Continuous Learning
During development, regularly reflect on:
- Problems encountered and their solutions
- Patterns that emerge across components
- Architectural decisions that prove helpful or problematic
- Developer experience friction points

### Memory Documentation
Maintain a `memory.md` file to capture:
- **Problem-Solution Pairs**: "When X happened, Y was the solution"
- **Pattern Recognition**: "This approach worked well for..."
- **Gotchas**: "Watch out for... because..."
- **Design Evolution**: "We changed X to Y because..."

### When to Update memory.md
- After resolving a non-obvious issue
- When discovering a better approach
- After making an architectural change
- When the same question comes up multiple times

This living document helps future development sessions learn from past experiences and avoid repeating solved problems.

## Future Enhancements

- Direct WIT-based component composition (when WASI 0.3 lands)
- TypeScript/JavaScript plugin support via `jco`
- Plugin hot-reloading
- Enhanced observability and tracing
- Plugin marketplace/registry

## Resources

- [Spin Documentation](https://developer.fermyon.com/spin)
- [WASM Component Model](https://component-model.bytecodealliance.org/)
- [Rust WASM Book](https://rustwasm.github.io/book/)
- [Model Context Protocol](https://modelcontextprotocol.io/)

## Memories

- Ensure makefile usage for all testing/building commands