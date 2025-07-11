<div align="center">

# *`ftl-mcp`*

</div>

A WebAssembly-based implementation of the Model Context Protocol (MCP) for building secure, fast, deployable AI tools in any language that run efficiently on almost any type of compute.

## Quick Start

### Writing a Tool

<details>
<summary><strong>🦀 Rust Example</strong></summary>

```rust
use ftl_sdk::{tool, ToolResponse};
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct EchoInput {
    message: String
}

#[tool]
fn echo(input: EchoInput) -> ToolResponse {
    ToolResponse::text(format!("Echo: {}", input.message))
}
```
</details>

<details>
<summary><strong>🟦 TypeScript Example</strong></summary>

```typescript
import { createTool, ToolResponse } from 'ftl-sdk'
import { z } from 'zod'

const InputSchema = z.object({
  message: z.string().describe('The message to echo')
})

const handle = createTool<z.infer<typeof InputSchema>>({
  metadata: {
    name: 'echo',
    title: 'Echo Tool',
    description: 'Echoes back the input message',
    inputSchema: z.toJSONSchema(InputSchema)
  },
  handler: async (input) => {
    return ToolResponse.text(`Echo: ${input.message}`)
  }
})

addEventListener('fetch', (event: FetchEvent) => {
  event.respondWith(handle(event.request))
})
```
</details>

## Why?

`ftl-mcp` enables you to build MCP servers that run as WebAssembly components on the wasmtime-based Spin framework. FTL toolkits can be deployed to extremely fast, globally distributed Akamai edge workers via Fermyon. Each tool is isolated in its own sandbox with capability-based permissions, providing a secure execution environment for each tool call.

## Architecture

```
MCP Client → Auth Gateway → MCP Gateway → [Tool Components]
```

### Components

- **Auth Gateway**: Optional JWT authentication layer using WorkOS AuthKit
- **MCP Gateway**: Routes JSON-RPC requests to appropriate tool components and validates tool call arguments
- **Tool Components**: Individual WebAssembly modules implementing specific functionality
- **SDKs**: TypeScript and Rust libraries for building tools

### Run and use MCP tools

1. Configure your tools in `spin.toml`:
```toml
[variables]
tool_components = { default = "echo,calculator,weather" }

[[trigger.http]]
route = "/mcp"
component = "ftl-mcp-gateway"

[component.ftl-mcp-gateway]
source = { registry = "ghcr.io", package = "fastertools:ftl-mcp-gateway", version = "0.0.3" }
allowed_outbound_hosts = ["http://*.spin.internal"]
[component.ftl-mcp-gateway.variables]
tool_components = "{{ tool_components }}"
```

2. Build and run:
```bash
spin up --build
```

3. Test your deployment:
```bash
# List available tools
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'

# Call a tool
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"tools/call",
    "params": {
      "name": "echo",
      "arguments": {"message": "Hello, World!"}
    },
    "id": 2
  }'
```

4. Plug in to your MCP client.
```json
{
  "mcpServers": {
    "ftl-tools": {
      "url": "http://127.0.0.1:3000/mcp",
      "transport": "http"
    }
  }
}
```

## Features

- **Language Support**: Build tools in TypeScript or Rust
- **Schema Validation**: Automatic input validation using JSON Schema
- **Security**: WebAssembly isolation with capability-based permissions
- **Authentication**: Optional JWT authentication with AuthKit integration
- **Performance**: Parallel tool execution and efficient routing
- **Standards**: Full MCP protocol compliance

## SDKs

### TypeScript SDK (`ftl-sdk`)
- Zero-dependency core with optional Zod integration
- Type-safe tool creation with `createTool` helper
- Automatic schema generation from Zod schemas

### Rust SDK (`ftl-sdk`)
- Procedural macros for minimal boilerplate
- Automatic schema generation using schemars
- Native async/await support

## Examples

See the `examples/demo` directory for complete examples:
- **echo-ts/echo-rs**: Basic echo tools
- **calculator-ts**: Advanced validation with Zod
- **weather-ts/weather-rs**: External API integration

## License

Apache-2.0