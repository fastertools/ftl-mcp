# FTL Auth Gateway Setup

The auth gateway has been added to your project. To complete the setup:

## 1. Manual Configuration Required

Update your `spin.toml` file:

1. Find the existing `ftl-mcp-gateway` trigger configuration:
   ```toml
   [[trigger.http]]
   route = "/mcp"
   component = "ftl-mcp-gateway"
   ```

2. Change the route to `/mcp-internal`:
   ```toml
   [[trigger.http]]
   route = "/mcp-internal"
   component = "ftl-mcp-gateway"
   ```

## 2. Configure AuthKit

1. Set up your WorkOS AuthKit application at https://dashboard.workos.com
2. Update the auth gateway variables in `spin.toml` with your actual values:
   - `authkit_issuer`: Your AuthKit issuer URL (e.g., `https://your-tenant.authkit.app`)
   - `authkit_audience`: (Optional) Your audience value
   - `authkit_jwks_uri`: (Optional) Custom JWKS URI

## 3. Testing Authentication

After configuration, your MCP endpoint at `/mcp` will require JWT authentication.

Test with a valid JWT token:
```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```
