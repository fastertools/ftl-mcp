# FTL MCP Example Tools

This directory contains a single toolkit composed of MCP tools written in Rust and TypeScript, demonstrating various features and implementation patterns using the FTL SDK.

## Quick Start

### Run All Examples

```bash
# Start the MCP gateway with all tools
spin up --build

# Or with authentication enabled
spin up --build -f spin-auth.toml
```

## Tools Overview

### Echo Tools
Simple tools that echo back input messages.

- **echo-ts**: TypeScript implementation using Zod schemas and the `createTool` helper
- **echo-rs**: Rust implementation using the `#[tool]` procedural macro

### Calculator Tool
- **calculator-ts**

### Weather Tools
Fetch weather data from external APIs.
- **weather-ts**
- **weather-rs**

### Test Individual Tools

Each tool has a test endpoint at `/test/{tool-name}`:

```bash
# Test echo tool metadata
curl http://localhost:3000/test/echo-ts

# Test echo tool execution
curl -X POST http://localhost:3000/test/echo-ts \
  -H "Content-Type: application/json" \
  -d '{"message": "Hello, World!"}'
```

### Use the MCP Gateway

List all available tools:
```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```

Call a specific tool:
```bash
# Echo example
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"tools/call",
    "params": {
      "name": "echo_ts",
      "arguments": {"message": "Hello from MCP!"}
    },
    "id": 2
  }'

# Calculator example
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"tools/call",
    "params": {
      "name": "calculator_ts",
      "arguments": {
        "operation": "multiply",
        "operands": [7, 6]
      }
    },
    "id": 3
  }'

# Weather example
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"tools/call",
    "params": {
      "name": "weather_ts",
      "arguments": {"location": "San Francisco"}
    },
    "id": 4
  }'
```

## Implementation Patterns

### TypeScript Pattern (echo-ts)

```typescript
import { createTool, ToolResponse } from 'ftl-sdk'
import { z } from 'zod'

// Define schema with Zod
const InputSchema = z.object({
  message: z.string().describe('The message to echo')
})

// Create tool with type-safe handler
const handle = createTool<z.infer<typeof InputSchema>>({
  metadata: {
    name: 'echo_ts',
    title: 'Echo Tool',
    description: 'Echoes back the input message',
    inputSchema: z.toJSONSchema(InputSchema)
  },
  handler: async (input) => {
    return ToolResponse.text(`Echo: ${input.message}`)
  }
})

// Register event listener for Spin
addEventListener('fetch', (event: FetchEvent) => {
  event.respondWith(handle(event.request))
})
```

### Rust Pattern (echo-rs)

```rust
use ftl_sdk::{tool, ToolResponse};
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct EchoInput {
    /// The message to echo back
    message: String
}

/// Echo the message back to the caller
#[tool]
fn echo_rs(input: EchoInput) -> ToolResponse {
    ToolResponse::text(format!("Echo: {}", input.message))
}
```

## Build Process

### TypeScript Tools

1. **Dependencies**: Install with `npm install`
2. **Type Checking**: `tsc --noEmit`
3. **Bundling**: esbuild creates ESM bundle
4. **WASM Conversion**: j2w converts JavaScript to WebAssembly

Build command:
```bash
npm run build
```

### Rust Tools

1. **Target**: wasm32-wasip1 (WASI Preview 1)
2. **Build**: Standard cargo build

Build command:
```bash
cargo build --target wasm32-wasip1 --release
```

## Configuration

### spin.toml
- Configures the MCP gateway and all tool components
- Sets up routing for both MCP endpoint and test endpoints
- Defines build commands and watch patterns for development
- Specifies allowed outbound hosts for tools that need network access

### spin-auth.toml
- Adds authentication layer using ftl-auth-gateway
- Configures AuthKit integration
- Routes public `/mcp` through auth before reaching the gateway

## Key Features Demonstrated

1. **Schema Validation**: Zod schemas automatically convert to JSON Schema for gateway validation
2. **Type Safety**: Full TypeScript/Rust type safety from schema to handler
3. **External APIs**: Weather tools show proper network request handling
4. **Error Handling**: Graceful error responses for validation and runtime failures
5. **Structured Responses**: Calculator returns both text and structured data
6. **Tool Annotations**: Metadata hints for tool behavior
7. **Async Operations**: Both TypeScript and Rust support async handlers
8. **Development Experience**: Hot reloading with file watching

## Choosing an Example

- **New to FTL?** Start with `echo-ts` or `echo-rs`
- **Need validation?** Check out `calculator-ts` for advanced Zod patterns
- **External APIs?** See `weather-ts` or `weather-rs`
- **TypeScript developer?** All `-ts` examples use modern TypeScript patterns
- **Rust developer?** The `-rs` examples showcase the `#[tool]` macro

## Troubleshooting

### Build Failures
- Ensure you have the correct toolchain:
  - TypeScript: Node.js 18+ and npm
  - Rust: wasm32-wasip1 target (`rustup target add wasm32-wasip1`)
- Check that j2w is available (installed with @spinframework/build-tools)

### Runtime Errors
- Check Spin logs: `spin logs`
- Verify allowed_outbound_hosts for tools making HTTP requests
- Ensure tool names match between configuration and gateway

### Network Issues
- Weather tools require internet access to Open-Meteo APIs
- Check firewall/proxy settings if requests fail