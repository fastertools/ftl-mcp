# {{project-name}}

{{project-description}}

## Getting Started

This FTL MCP server is ready to host MCP tools. Add tools using:

```bash
# Add a TypeScript tool
spin add -t ftl-mcp-ts

# Add a Rust tool
spin add -t ftl-mcp-rust
```

## Running the Server

```bash
spin build --up
```

The MCP endpoint will be available at `http://localhost:3000/mcp`

## Testing

List available tools:
```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```

## Authentication

This MCP server includes authentication support via FTL Auth Gateway, which is **disabled by default**. 

To enable authentication:

1. Configure your authentication providers in the `auth_config` variable
2. Set `"enabled": true` in the auth configuration

Example configuration with WorkOS AuthKit:
```toml
[variables]
auth_config = '''
{
  "mcp_gateway_url": "http://ftl-mcp-gateway.spin.internal/mcp-internal",
  "trace_id_header": "X-Trace-Id",
  "enabled": true,
  "providers": [
    {
      "name": "workos",
      "type": "workos",
      "config": {
        "client_id": "YOUR_CLIENT_ID",
        "api_key": "YOUR_API_KEY",
        "redirect_uri": "YOUR_REDIRECT_URI"
      }
    }
  ]
}
'''
```

When authentication is enabled, the auth gateway will:
- Handle OAuth 2.0 flows at `/.well-known/oauth-protected-resource` and `/.well-known/oauth-authorization-server`
- Validate tokens on requests to `/mcp`
- Forward authenticated requests to the internal MCP gateway