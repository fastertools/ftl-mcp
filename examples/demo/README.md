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
