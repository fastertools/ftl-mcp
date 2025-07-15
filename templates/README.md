# FTL MCP Spin Templates

Spin templates for building MCP servers and tools with FTL.

## Templates

- **ftl-mcp-server** - MCP server with FTL gateway and auth support (disabled by default)
- **ftl-mcp-rust** - Rust tool component
- **ftl-mcp-ts** - TypeScript tool component
- **ftl-auth-gateway** - WorkOS AuthKit authentication (for adding to existing projects)

## Installation

```bash
spin templates install --dir .
```

## Quick Start

### 1. Create Server

```bash
spin new -t ftl-mcp-server my-server
cd my-server
```

### 2. Add Tools

```bash
spin add -t ftl-mcp-ts greeting
spin add -t ftl-mcp-rust calculator
```

### 3. Register Tools

Update `tool_components` in `spin.toml`:

```toml
[variables]
tool_components = { default = "greeting,calculator" }
```

### 4. Build and Run

```bash
spin build
spin up
```

## Testing

```bash
# List tools
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'

# Call a tool
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"greeting","arguments":{"message":"Hello"}},"id":2}'
```

## Authentication

The `ftl-mcp-server` template includes authentication support that is **disabled by default**. To enable authentication, update the `auth_config` variable in your `spin.toml` file with `"enabled": true` and configure your authentication providers.

For existing projects without authentication support, you can add it using:

```bash
spin add -t ftl-auth-gateway
```

See the Authentication section in your project's README.md for configuration details.