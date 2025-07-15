# FTL Auth Gateway Setup

The auth gateway has been added to your project. To complete the setup:

## 1. Manual Configuration Required

Update your `spin.toml` file:

1. Find the existing `mcp` component trigger configuration:
   ```toml
   [[trigger.http]]
   route = "/mcp"
   component = "mcp"
   ```

2. Update it to use a private route and rename the component:
   ```toml
   [[trigger.http]]
   route = { private = true }
   component = "ftl-mcp-gateway"
   ```

3. The auth gateway component has already been added with the correct routes:
   - `/mcp` - Main MCP endpoint (with optional authentication)
   - `/.well-known/oauth-protected-resource` - OAuth discovery
   - `/.well-known/oauth-authorization-server` - OAuth discovery

## 2. Authentication Configuration

By default, authentication is **disabled**. The auth gateway will forward requests directly to the MCP gateway without authentication.

To enable authentication, override the `auth_config` variable when running your application:

### Option 1: Using environment variables

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

spin up
```

### Option 2: Using a configuration file

Create a `.env` file or shell script with your auth configuration. See the project's `.env.example` for more provider examples.

## 3. Available Authentication Providers

### AuthKit (WorkOS)
```json
{
  "type": "authkit",
  "issuer": "https://your-tenant.authkit.app",
  "audience": "mcp-api"  // optional
}
```

### Generic OIDC (Auth0, Keycloak, etc)
```json
{
  "type": "oidc",
  "name": "auth0",
  "issuer": "https://your-domain.auth0.com",
  "jwks_uri": "https://your-domain.auth0.com/.well-known/jwks.json",
  "authorization_endpoint": "https://your-domain.auth0.com/authorize",
  "token_endpoint": "https://your-domain.auth0.com/oauth/token",
  "userinfo_endpoint": "https://your-domain.auth0.com/userinfo",  // optional
  "audience": "your-api-identifier",  // optional
  "allowed_domains": ["*.auth0.com"]  // optional
}
```

## 4. Testing

### Without Authentication (default)
```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```

### With Authentication (when enabled)
```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```