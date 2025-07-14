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

### With Authentication (AuthKit)

```bash
# Set your AuthKit issuer URL via environment variable
SPIN_VARIABLE_AUTHKIT_ISSUER="https://your-tenant.authkit.app" spin build --up -f spin-auth.toml
```

The authenticated MCP endpoint will be available at `http://localhost:3000/mcp` and requires a valid JWT token from AuthKit.

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
