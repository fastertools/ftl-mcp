# ftl-mcp-demo

FTL MCP server for hosting MCP tools

## Getting Started

This FTL MCP server is ready to host MCP tools. Add tools using:

```bash
# Add a TypeScript tool
spin add -t ftl-mcp-ts

# Add a Rust tool
spin add -t ftl-mcp-rust
```

## Running the Server

### Without Authentication (Default)

```bash
spin build --up
```

The MCP endpoint will be available at `http://localhost:3000/mcp`

### With Authentication

Authentication is now controlled via the `auth_config` variable. By default, authentication is disabled. To enable it, override the configuration:

```bash
# Example with AuthKit
export SPIN_VARIABLE_AUTH_CONFIG='{
  "mcp_gateway_url": "http://ftl-mcp-gateway.spin.internal/mcp-internal",
  "trace_id_header": "X-Trace-Id",
  "enabled": true,
  "providers": [{
    "type": "authkit",
    "issuer": "https://your-tenant.authkit.app"
  }]
}'

spin build --up
```

For more authentication examples and provider configurations, see `.env.example`.

The authenticated MCP endpoint will be available at `http://localhost:3000/mcp` and requires a valid JWT token from the configured provider.

## Testing

### Without Authentication

List available tools:
```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```

### With Authentication

First, check the authentication requirements:
```bash
# This will return 401 with authentication details
curl -i http://localhost:3000/mcp

# Discover OAuth configuration
curl http://localhost:3000/.well-known/oauth-protected-resource
```

Then make authenticated requests with a JWT token:
```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```
