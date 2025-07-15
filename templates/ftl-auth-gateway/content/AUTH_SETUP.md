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
   - `/mcp` - Main MCP endpoint (requires authentication)
   - `/.well-known/oauth-protected-resource` - OAuth discovery
   - `/.well-known/oauth-authorization-server` - OAuth discovery

## 2. Configure AuthKit

1. Set up your WorkOS AuthKit application at https://dashboard.workos.com
2. Set the `authkit_issuer` variable in your deployment or `.env` file:
   ```bash
   # Example:
   authkit_issuer="https://your-tenant.authkit.app"
   ```

3. The auth gateway is configured to use the new JSON-based `auth_config` format which includes:
   - MCP gateway URL (already configured)
   - Trace ID header for request tracking
   - AuthKit provider configuration

## 3. Testing Authentication

After configuration, your MCP endpoint at `/mcp` will require JWT authentication.

Test with a valid JWT token:
```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```
