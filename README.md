<div align="center">

# *`ftl-mcp`*

</div>

A WebAssembly-based implementation of the Model Context Protocol (MCP) for building secure, fast, deployable tools that plug in to any AI agent and run efficiently on almost any type of compute.

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
    /// The message to echo back to the caller, verbatim
    message: String
}

/// Echo the message back to the caller
#[tool]
fn echo_rs(input: EchoInput) -> ToolResponse {
    ToolResponse::text(format!("Echo: {}", input.message))
}

```
</details>

<details>
<summary><strong>🟦 TypeScript Example</strong></summary>

```typescript
import { createTool, ToolResponse } from 'ftl-sdk'
import { z } from 'zod'

// Define the schema using Zod
const EchoInputSchema = z.object({
  message: z.string().describe('The message to echo back to the caller, verbatim')
})

// Derive TypeScript type from the schema
type EchoInput = z.infer<typeof EchoInputSchema>

const echo = createTool<EchoInput>({
  metadata: {
    name: 'echo_ts',
    title: 'Echo TS',
    description: 'Echo the message back to the caller',
    // Using Zod v4's native JSON Schema conversion
    inputSchema: z.toJSONSchema(EchoInputSchema)
  },
  handler: async (input) => {
    // Input is pre-validated
    return ToolResponse.text(`Echo: ${input.message}`)
  }
})

//@ts-ignore
addEventListener('fetch', (event: FetchEvent) => {
  event.respondWith(echo(event.request))
})
```
</details>

## Why?

`ftl-mcp` enables you to build MCP-compatible toolkits that run as WebAssembly components on the wasmtime-based Spin framework. FTL toolkits can be natively deployed to extremely fast, globally distributed Akamai edge workers via Fermyon. They can also run on any WebAssembly-compatible host. Each tool executes within its own isolated sandbox with capability-based permissions. You can mix and match tools written in different source languages within a toolkit.

## Architecture

```
MCP Client → Auth Gateway → MCP Gateway → [Tool Components]
```

### Components

- **Auth Gateway**: MCP-compatible authentication layer using WorkOS AuthKit
- **MCP Gateway**: Implements the MCP protocol, routes requests to the appropriate tool component, and validates tool call arguments
- **Tool Components**: Individual WebAssembly components implementing specific functionality as MCP tools
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
- **Standards**: Full MCP protocol compliance. Built on the Wasm component model.

## SDKs

### TypeScript SDK (`ftl-sdk`)
- Zero-dependency core with optional Zod integration
- Type-safe tool creation with `createTool` helper
- Automatic schema generation from Zod schemas

### Rust SDK (`ftl-sdk`)
- Procedural macros for minimal boilerplate
- Automatic schema generation using schemars
- Native async/await support

## Getting Started with Templates

Install the Spin templates:
```bash
spin templates install --dir .
```

Create a new MCP server:
```bash
spin new -t ftl-mcp-server my-server
cd my-server
spin add -t ftl-mcp-ts hello-tool
spin add -t ftl-mcp-rust greet-tool
# Update tool_components in spin.toml
spin build --up
```

See the `templates/` directory for available templates and documentation.

## Examples

See the `examples/demo` directory for complete examples:
- **echo-ts/echo-rs**: Basic echo tools
- **calculator-ts**: Advanced validation with Zod
- **weather-ts/weather-rs**: External API integration

## License

Apache-2.0