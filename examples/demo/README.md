# FTL MCP Demo Examples

This directory contains various example implementations of MCP tools using the FTL SDK.

## Examples Overview

### JavaScript Examples

- **echo-js** - Uses itty-router for routing, shows the traditional approach
- **echo-simplified-js** - Pure JavaScript using the `createTool` helper (no TypeScript)

### TypeScript Examples

- **echo-ts** - Basic TypeScript example without any helpers (raw implementation)
- **echo-simplified-ts** - TypeScript with `createTool` helper and Zod schema validation
- **calculator-ts** - Advanced TypeScript example showcasing Zod's validation capabilities

### Rust Examples

- **echo** - Basic Rust implementation without macros
- **echo-simplified** - Rust using the `#[tool]` macro for minimal boilerplate
- **add** - Basic addition tool
- **add-advanced** - Addition tool using `#[tool_component]` with full metadata control
- **image-demo** - Demonstrates handling binary data (images)

## Which Example Should I Use?

### For JavaScript Developers
- Start with **echo-simplified-js** if you want the simplest approach
- Use **echo-js** if you prefer using a router library

### For TypeScript Developers
- Start with **echo-simplified-ts** for the best developer experience with Zod
- Use **echo-ts** if you want to understand the raw implementation
- Check **calculator-ts** for advanced validation patterns

### For Rust Developers
- Start with **echo-simplified** for the cleanest code
- Use **echo** to understand the underlying HTTP handling
- Check **add-advanced** for full control over metadata

## Key Differences

1. **Language Choice**: JavaScript vs TypeScript vs Rust
2. **Abstraction Level**: Raw implementation vs SDK helpers/macros
3. **Validation**: Manual validation vs Zod schemas (TypeScript only)
4. **Routing**: Some examples use routers (itty-router) while others use direct request handling

## Running the Examples

All examples are configured in `spin.toml`. To run the entire demo:

```bash
spin build --up
```

Then test individual tools:
```bash
# Test endpoints
curl http://localhost:3000/test/echo-ts
curl http://localhost:3000/test/calculator-ts

# Or use the MCP gateway
curl http://localhost:3000/mcp -X POST -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```