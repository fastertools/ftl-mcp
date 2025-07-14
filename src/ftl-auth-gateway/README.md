# FTL Auth Gateway

A provider-agnostic JWT authentication gateway for MCP servers supporting any OIDC-compliant identity provider.

## Overview

The FTL Auth Gateway provides OAuth 2.0 authentication for MCP endpoints by validating JWT tokens and injecting user context into requests. It supports multiple authentication providers simultaneously, including WorkOS AuthKit, Auth0, Keycloak, Okta, Azure AD, and any OIDC-compliant provider.

```
MCP Client → FTL Auth Gateway → FTL MCP Gateway → Tool Components
              (JWT Auth)         (Internal)        (WASM)
```

## Features

- **Multi-Provider Support**: Configure multiple OIDC providers and authenticate with any of them
- **Provider Agnostic**: Works with AuthKit, Auth0, Keycloak, Okta, Azure AD, Google Identity, or any OIDC provider
- **Secure JWT Verification**: Full RS256/ES256 signature verification with automatic JWKS key rotation
- **OAuth 2.0 Metadata Discovery**: Complete implementation of discovery endpoints for zero-config client integration
- **User Context Injection**: Automatically injects authenticated user information into MCP `initialize` requests
- **Structured Logging**: All logs include trace IDs for request correlation and debugging
- **JWKS Caching**: Intelligent 5-minute cache to minimize network calls while supporting key rotation
- **Production Ready**: Full support for X-Forwarded headers, CORS, and cloud deployments
- **Transparent Proxying**: Seamlessly forwards authenticated requests to the internal MCP gateway

## Configuration

The gateway is configured using a JSON configuration via the `auth_config` Spin variable:

```json
{
  "mcp_gateway_url": "http://ftl-mcp-gateway.spin.internal/mcp-internal",
  "trace_id_header": "X-Trace-Id",
  "providers": [
    {
      "type": "authkit",
      "issuer": "https://your-domain.authkit.app",
      "jwks_uri": "https://your-domain.authkit.app/oauth2/jwks",
      "audience": "your-api-audience"
    },
    {
      "type": "oidc",
      "name": "auth0",
      "issuer": "https://your-domain.auth0.com",
      "jwks_uri": "https://your-domain.auth0.com/.well-known/jwks.json",
      "authorization_endpoint": "https://your-domain.auth0.com/authorize",
      "token_endpoint": "https://your-domain.auth0.com/oauth/token",
      "allowed_domains": ["*.auth0.com"]
    }
  ]
}
```

### Configuration Fields

- `mcp_gateway_url`: Internal URL of the FTL MCP Gateway
- `trace_id_header`: Header name for trace ID propagation (default: "X-Trace-Id")
- `providers`: Array of authentication provider configurations
  - `type`: Either "authkit" or "oidc"
  - `issuer`: The OIDC issuer URL
  - `jwks_uri`: JWKS endpoint URL (optional for AuthKit, computed from issuer)
  - `audience`: Expected audience for JWT validation (optional)
  - For OIDC providers:
    - `name`: Unique name for the provider
    - `authorization_endpoint`: OAuth 2.0 authorization endpoint
    - `token_endpoint`: OAuth 2.0 token endpoint
    - `userinfo_endpoint`: OIDC userinfo endpoint (optional)
    - `allowed_domains`: List of allowed domains for JWKS fetching

## Complete AuthKit Example

Here's a complete example of setting up the auth gateway with WorkOS AuthKit:

### 1. Create `spin.toml`

```toml
spin_manifest_version = 2

[application]
name = "mcp-with-auth"
version = "0.1.0"

[variables]
tool_components = { default = "my-tool" }

# Auth Gateway - handles authentication
[[trigger.http]]
route = "/mcp"
component = "ftl-auth-gateway"

[component.ftl-auth-gateway]
source = { registry = "ghcr.io", package = "fastertools:ftl-auth-gateway", version = "0.1.0" }
allowed_outbound_hosts = ["http://*.spin.internal", "https://*.authkit.app"]
[component.ftl-auth-gateway.variables]
auth_config = '''
{
  "mcp_gateway_url": "http://ftl-mcp-gateway.spin.internal/mcp-internal",
  "trace_id_header": "X-Request-Id",
  "providers": [{
    "type": "authkit",
    "issuer": "https://your-tenant.authkit.app",
    "audience": "mcp-api"
  }]
}
'''

# MCP Gateway - internal endpoint (protected by auth gateway)
[[trigger.http]]
route = "/mcp-internal"
component = "ftl-mcp-gateway"

[component.ftl-mcp-gateway]
source = { registry = "ghcr.io", package = "fastertools:ftl-mcp-gateway", version = "0.0.3" }
allowed_outbound_hosts = ["http://*.spin.internal"]
[component.ftl-mcp-gateway.variables]
tool_components = "{{ tool_components }}"

# Your MCP Tool
[[trigger.http]]
route = "/my-tool"
component = "my-tool"

[component.my-tool]
source = "target/wasm32-wasip1/release/my_tool.wasm"
[component.my-tool.build]
command = "cargo build --target wasm32-wasip1 --release"
```

### 2. Set up AuthKit

1. Sign up for [WorkOS](https://workos.com) and create an organization
2. In the WorkOS Dashboard, go to AuthKit
3. Create a new AuthKit instance
4. Note your AuthKit domain (e.g., `your-tenant.authkit.app`)
5. Configure redirect URIs for your MCP client

### 3. Test Authentication Flow

```bash
# 1. Start the server
spin up

# 2. Attempt unauthenticated access (returns 401)
curl -i http://localhost:3000/mcp

# Response includes WWW-Authenticate header:
# WWW-Authenticate: Bearer error="unauthorized", 
#   error_description="Missing authorization header",
#   resource_metadata="http://localhost:3000/.well-known/oauth-protected-resource"

# 3. Discover OAuth configuration
curl http://localhost:3000/.well-known/oauth-protected-resource

# 4. Get JWT token from AuthKit (use AuthKit SDK or OAuth flow)
# Example using AuthKit Node.js SDK:
# const token = await authkit.getAccessToken(userId)

# 5. Make authenticated MCP request
curl -H "Authorization: Bearer YOUR_JWT_TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"tools/list","id":1}' \
     http://localhost:3000/mcp
```

### 4. Client Integration Example

For MCP clients, use the OAuth 2.0 discovery:

```typescript
// Discover OAuth configuration
const resourceMeta = await fetch('https://your-app.com/.well-known/oauth-protected-resource')
  .then(r => r.json());

const authServer = resourceMeta.authorization_servers[0];

// Use AuthKit SDK or standard OAuth flow
import { AuthKit } from '@workos-inc/authkit';

const authkit = new AuthKit({
  domain: 'your-tenant.authkit.app',
  clientId: 'your_client_id'
});

// Get access token
const { accessToken } = await authkit.signIn({
  email: 'user@example.com',
  password: 'password'
});

// Use token with MCP client
const response = await fetch('https://your-app.com/mcp', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${accessToken}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    jsonrpc: '2.0',
    method: 'tools/list',
    id: 1
  })
});
```

## Multi-Provider Example

Configure multiple providers to allow authentication from different identity providers:

```json
{
  "mcp_gateway_url": "http://ftl-mcp-gateway.spin.internal/mcp-internal",
  "trace_id_header": "X-Trace-Id",
  "providers": [
    {
      "type": "authkit",
      "issuer": "https://acme.authkit.app",
      "audience": "mcp-api"
    },
    {
      "type": "oidc",
      "name": "google",
      "issuer": "https://accounts.google.com",
      "jwks_uri": "https://www.googleapis.com/oauth2/v3/certs",
      "authorization_endpoint": "https://accounts.google.com/o/oauth2/v2/auth",
      "token_endpoint": "https://oauth2.googleapis.com/token",
      "allowed_domains": ["*.google.com", "*.googleapis.com"]
    },
    {
      "type": "oidc", 
      "name": "internal",
      "issuer": "https://auth.internal.company.com",
      "jwks_uri": "https://auth.internal.company.com/.well-known/jwks.json",
      "authorization_endpoint": "https://auth.internal.company.com/oauth/authorize",
      "token_endpoint": "https://auth.internal.company.com/oauth/token",
      "audience": "internal-api",
      "allowed_domains": ["*.internal.company.com"]
    }
  ]
}
```

## Authentication Flow

1. MCP client attempts to access `/mcp` endpoint
2. If no token provided, returns 401 with `WWW-Authenticate` header
3. Client discovers OAuth configuration via `/.well-known/oauth-protected-resource`
4. Client authenticates with any configured provider and receives JWT token
5. Client includes JWT in `Authorization: Bearer <token>` header
6. Gateway:
   - Extracts key ID (kid) from JWT header
   - Tries each configured provider's JWKS endpoint
   - Verifies JWT signature using the appropriate public key
   - Validates issuer, expiration, and optional audience claims
7. On successful validation:
   - Extracts user information from JWT claims
   - Injects user context into MCP `initialize` requests with provider info
   - Forwards all requests to internal MCP gateway
8. Response is proxied back to client with trace ID and CORS headers

## User Context Injection

The gateway automatically injects authenticated user information into MCP `initialize` requests:

```json
{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": {
    "protocolVersion": "0.1.0",
    "_authContext": {
      "authenticated_user": "user123",
      "email": "user@example.com",
      "provider": "authkit"
    }
  }
}
```

## Endpoints

### OAuth Metadata Endpoints

- `GET /.well-known/oauth-protected-resource` - Returns resource metadata for OAuth discovery
- `GET /.well-known/oauth-authorization-server` - Returns authorization server metadata

### MCP Endpoint

- `POST /mcp` - Protected MCP endpoint requiring Bearer token authentication
- `OPTIONS /mcp` - CORS preflight endpoint

## Development

### Prerequisites
- Rust toolchain with `wasm32-wasip1` target
- Spin CLI v2.0+

### Building

```bash
# Build the gateway
cargo build --target wasm32-wasip1 --release

# Run tests
cargo test
spin test
```

### Testing

The gateway includes comprehensive tests using the Spin test framework:

```bash
# Run unit tests
cargo test

# Run integration tests
spin test
```

## Deployment

### Fermyon Cloud
The gateway automatically detects Fermyon Cloud deployments and handles:
- X-Forwarded-Host headers for proper URL construction
- HTTPS protocol detection for `.fermyon.tech` and `.fermyon.cloud` domains
- Proper CORS headers for cross-origin requests

### Performance
- JWKS caching reduces identity provider API calls by ~99%
- JWT verification adds ~1-2ms latency per request
- Structured logging with trace IDs enables request tracing
- Internal Spin networking ensures minimal overhead

## Troubleshooting

### Common Issues

1. **"Failed to get decoding key" errors**
   - Verify the JWKS URI is accessible from your deployment
   - Check that the JWT's `kid` exists in the JWKS response
   - Ensure allowed_outbound_hosts includes your identity provider's domain

2. **"Invalid audience" errors**
   - Configure the expected audience in the provider configuration
   - Or omit audience to skip validation

3. **"No authentication providers configured"**
   - Ensure auth_config is properly set with at least one provider
   - Check JSON syntax in the configuration

### Debug Logging

View structured logs with trace IDs:
```bash
# Local development
spin up

# Fermyon Cloud
spin cloud logs

# Example log output:
[INFO] trace_id=gen-19806d27e75 Metadata request path=/.well-known/oauth-protected-resource host=example.com
[INFO] trace_id=req-123-456 Authentication successful provider=authkit user_id=user_123
[WARN] trace_id=req-789-012 Authentication failed with all providers
```

## Security Considerations

- All JWT verification uses public keys - no secrets are stored
- Supports RS256 and ES256 algorithms
- Automatic JWKS key rotation with caching
- Provider domains are restricted via allowed_domains
- Internal gateway communication uses Spin's secure internal networking
- Each request is tagged with a trace ID for audit trails

## License

Apache-2.0