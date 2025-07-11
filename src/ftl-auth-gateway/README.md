# FTL Auth Gateway

An authentication gateway for FTL MCP servers that integrates with WorkOS AuthKit to provide OAuth 2.0 authentication.

## Overview

The FTL Auth Gateway acts as a security layer that sits in front of the FTL MCP Gateway, ensuring that all MCP requests are properly authenticated before being forwarded to the actual MCP server.

```
MCP Client → FTL Auth Gateway → FTL MCP Gateway → Tool Components
              (AuthKit JWT)       (Internal)        (WASM)
```

## Features

- **JWT Verification**: Validates JWT tokens issued by AuthKit
- **OAuth 2.0 Metadata**: Implements discovery endpoints for zero-config client integration
- **Dynamic Client Registration**: Supports MCP clients that can self-register with AuthKit
- **User Context Injection**: Adds authenticated user information to MCP requests
- **Transparent Proxying**: Forwards authenticated requests to the MCP gateway

## Configuration

The gateway is configured using Spin variables:

- `authkit_issuer`: The AuthKit issuer URL (e.g., `https://your-domain.authkit.app`)
- `authkit_audience`: (Optional) Expected audience for JWT validation
- `authkit_jwks_uri`: (Optional) JWKS endpoint URL (defaults to `{issuer}/oauth2/jwks`)
- `mcp_gateway_url`: Internal URL of the FTL MCP Gateway (defaults to `http://ftl-mcp-gateway.spin.internal/mcp-internal`)

## Endpoints

### OAuth Metadata Endpoints

- `GET /.well-known/oauth-protected-resource` - Returns resource metadata pointing to AuthKit
- `GET /.well-known/oauth-authorization-server` - Returns AuthKit's authorization server metadata

### MCP Endpoint

- `POST /mcp` - Protected MCP endpoint requiring Bearer token authentication

## Authentication Flow

1. MCP client attempts to access `/mcp` endpoint
2. If no token provided, returns 401 with `WWW-Authenticate` header
3. Client discovers AuthKit via metadata endpoints
4. Client authenticates with AuthKit and receives JWT
5. Client includes JWT in `Authorization: Bearer <token>` header
6. Gateway validates JWT and forwards request to MCP gateway
7. User context is injected into appropriate MCP messages

## Usage Example

### With Spin

```toml
# spin.toml
[[trigger.http]]
route = "/mcp"
component = "ftl-auth-gateway"

[component.ftl-auth-gateway]
source = "path/to/ftl_auth_gateway.wasm"
allowed_outbound_hosts = ["http://*.spin.internal", "https://*.authkit.app"]
[component.ftl-auth-gateway.variables]
authkit_issuer = "https://your-domain.authkit.app"
mcp_gateway_url = "http://ftl-mcp-gateway.spin.internal/mcp-internal"
```

### Testing

```bash
# Get metadata
curl http://localhost:3000/.well-known/oauth-protected-resource

# Make authenticated request
curl -H "Authorization: Bearer YOUR_JWT_TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"tools/list","id":1}' \
     http://localhost:3000/mcp
```

## Development

```bash
# Build the gateway
cd src/ftl-auth-gateway
cargo build --target wasm32-wasip1 --release

# Run with demo
cd examples/demo
spin build --up -f spin-auth.toml
```

## Security Notes

- The gateway performs basic JWT validation including signature, issuer, audience, and expiration checks
- For production use, implement proper JWKS fetching and caching
- Consider adding rate limiting and additional security headers
- Ensure AuthKit Dynamic Client Registration is enabled for MCP client compatibility