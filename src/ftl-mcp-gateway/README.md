# FTL MCP Gateway

The core routing component that implements the Model Context Protocol (MCP) server and forwards requests to individual tool components.

## Overview

The MCP Gateway acts as a central router that:
- Implements the MCP JSON-RPC protocol
- Discovers and lists available tools from configured components
- Validates tool arguments against JSON schemas
- Routes tool calls to appropriate WebAssembly components
- Handles errors and protocol compliance

## Architecture

```
JSON-RPC Request → MCP Gateway → Tool Component (via Spin internal networking)
                        ↓
                  Tool Discovery
                  Validation
                  Routing
```

## Configuration

The gateway is configured using Spin variables:

```toml
[component.ftl-mcp-gateway.variables]
tool_components = "echo,calculator,weather"  # Comma-separated list of tools
validate_arguments = "true"                  # Enable JSON schema validation
```

## Protocol Implementation

### Supported Methods

- `initialize` - Establishes protocol version and capabilities
- `initialized` - Notification (no response)
- `tools/list` - Returns metadata for all configured tools
- `tools/call` - Executes a specific tool with arguments
- `ping` - Health check

### Request Flow

1. **Tool Discovery**: On startup, the gateway fetches metadata from each configured tool component
2. **Validation**: When `validate_arguments` is enabled, incoming arguments are validated against the tool's JSON schema
3. **Routing**: Tool names are converted from snake_case to kebab-case for component resolution
4. **Execution**: Requests are forwarded to `http://{tool-name}.spin.internal/`

## Tool Component Requirements

Each tool component must:

1. Respond to GET requests with metadata:
```json
{
  "name": "tool_name",
  "title": "Human Readable Name",
  "description": "What this tool does",
  "inputSchema": { /* JSON Schema */ }
}
```

2. Respond to POST requests with tool execution:
```json
{
  "content": [
    {
      "type": "text",
      "text": "Tool output"
    }
  ]
}
```

## Error Handling

The gateway returns JSON-RPC error responses for:
- Invalid protocol versions
- Unknown methods
- Missing tools
- Validation failures
- Tool execution errors

Standard error codes:
- `-32700`: Parse error
- `-32600`: Invalid request
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error

## Performance Features

- Parallel metadata fetching for all tools
- Minimal overhead routing via Spin's internal networking
- Optional argument validation can be disabled for performance

## Usage Example

```bash
# Initialize the connection
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "initialize",
    "params": {
      "protocolVersion": "2025-06-18",
      "capabilities": {},
      "clientInfo": {
        "name": "example-client",
        "version": "1.0.0"
      }
    },
    "id": 1
  }'

# List available tools
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":2}'

# Call a tool
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "echo",
      "arguments": {"message": "Hello"}
    },
    "id": 3
  }'
```

## Development

Built with:
- Rust and the Spin SDK
- JSON Schema validation via jsonschema crate
- Async/await for concurrent operations

To modify the gateway:
1. Update the source in `src/`
2. Build: `cargo build --target wasm32-wasip1 --release`
3. Test with the demo application