# ftl-sdk (TypeScript)

TypeScript SDK for building Model Context Protocol (MCP) tools that compile to WebAssembly.

## Installation

```bash
npm install ftl-sdk
```

## Overview

This SDK provides:
- TypeScript type definitions for the MCP protocol
- Zero-dependency `createTool` helper for building tools
- Seamless integration with Zod v4 for schema validation
- Full compatibility with Spin WebAssembly components

## Quick Start

### Using the `createTool` Helper (Recommended)

The SDK includes a `createTool` helper that handles the MCP protocol for you:

```typescript
import { createTool, ToolResponse } from 'ftl-sdk'
import { z } from 'zod'

// Define your input schema using Zod
const InputSchema = z.object({
  message: z.string().describe('The message to process')
})

// Create the tool handler
const handle = createTool<z.infer<typeof InputSchema>>({
  metadata: {
    name: 'my-tool',
    title: 'My Tool',
    description: 'A simple tool example',
    inputSchema: z.toJSONSchema(InputSchema) as Record<string, unknown>
  },
  handler: async (input) => {
    // Input is already validated by the gateway
    return ToolResponse.text(`Processed: ${input.message}`)
  }
})

// For Spin components
addEventListener('fetch', (event: FetchEvent) => {
  event.respondWith(handle(event.request))
})
```

### Manual Implementation

You can also implement the protocol manually with any web framework:

```typescript
import { ToolMetadata, ToolResponse } from 'ftl-sdk';
import { AutoRouter } from 'itty-router';

const router = AutoRouter();

router
  .get('/', async () => {
    // Return tool metadata
    const metadata: ToolMetadata = {
      name: 'my-tool',
      title: 'My Tool',
      description: 'Does something useful',
      inputSchema: {
        type: 'object',
        properties: {
          input: { type: 'string' }
        },
        required: ['input']
      }
    };
    
    return new Response(JSON.stringify(metadata), {
      headers: { 'Content-Type': 'application/json' }
    });
  })
  .post('/', async (request) => {
    // Parse input, do work...
    const response = ToolResponse.text('Tool executed successfully!');
    
    return new Response(JSON.stringify(response), {
      headers: { 'Content-Type': 'application/json' }
    });
  });

export default router;
```

## Building to WebAssembly

Tools must be compiled to WebAssembly to run on the Spin platform:

```json
{
  "scripts": {
    "build": "esbuild src/index.ts --bundle --outfile=build/bundle.js --format=esm --platform=browser --external:node:* && j2w -i build/bundle.js -o dist/my-tool.wasm"
  },
  "devDependencies": {
    "@spinframework/build-tools": "^1.0.1"
  }
}
```

The build process:
1. Bundle TypeScript to ESM format using esbuild
2. Convert JavaScript to WebAssembly using `j2w` (js-to-wasm)

## Using with Zod

The SDK integrates with Zod v4's native JSON Schema conversion:

```typescript
import { z } from 'zod'

// Define schema with validation rules
const CalculatorSchema = z.object({
  operation: z.enum(['add', 'subtract', 'multiply', 'divide']),
  a: z.number(),
  b: z.number()
}).refine(
  (data) => data.operation !== 'divide' || data.b !== 0,
  { message: "Cannot divide by zero" }
)

// Convert to JSON Schema - validation rules are preserved
const jsonSchema = z.toJSONSchema(CalculatorSchema)

// Use with createTool
const handle = createTool<z.infer<typeof CalculatorSchema>>({
  metadata: {
    name: 'calculator',
    inputSchema: jsonSchema as Record<string, unknown>
  },
  handler: async (input) => {
    // input is fully typed and validated by the gateway
    switch (input.operation) {
      case 'add': return ToolResponse.text(`Result: ${input.a + input.b}`)
      case 'subtract': return ToolResponse.text(`Result: ${input.a - input.b}`)
      case 'multiply': return ToolResponse.text(`Result: ${input.a * input.b}`)
      case 'divide': return ToolResponse.text(`Result: ${input.a / input.b}`)
    }
  }
})
```

## Important: Input Validation

**Tools should NOT validate inputs themselves.** The FTL gateway handles all input validation against your tool's JSON Schema before invoking your handler. This means:

- Your handler can assume all inputs are valid
- Type safety is guaranteed at runtime
- Complex validation rules (like Zod refinements) are enforced by the gateway
- You can focus on business logic, not validation

## API Reference

### `createTool<T>(options)`

Creates a request handler for MCP tool requests.

```typescript
interface CreateToolOptions<T> {
  metadata: ToolMetadata
  handler: (input: T) => ToolResponse | Promise<ToolResponse>
}
```

The returned handler:
- Returns tool metadata on GET requests
- Executes your handler on POST requests with validated input
- Handles errors gracefully

### `ToolResponse` Helper Methods

```typescript
// Simple text response
ToolResponse.text('Hello, world!')

// Error response
ToolResponse.error('Something went wrong')

// Response with structured content
ToolResponse.withStructured('Operation complete', { result: 42 })
```

### `ToolContent` Helper Methods

```typescript
// Text content
ToolContent.text('Some text', { priority: 0.8 })

// Image content
ToolContent.image(base64Data, 'image/png')

// Audio content
ToolContent.audio(base64Data, 'audio/wav')

// Resource reference
ToolContent.resource({ uri: 'file:///example.txt' })
```

### Type Guards

```typescript
import { isTextContent, isImageContent, isAudioContent, isResourceContent } from 'ftl-sdk'

// Check content types
if (isTextContent(content)) {
  console.log(content.text)
}
```

## Best Practices

1. **Use Zod for Schema Definition**: Leverage Zod's powerful schema capabilities and convert to JSON Schema using `z.toJSONSchema()`.

2. **Trust Input Validation**: Don't validate inputs in your handler - the gateway ensures inputs match your schema.

3. **Keep It Simple**: The SDK is intentionally minimal. Use it for types and basic helpers, not complex abstractions.

4. **Type Safety**: Always provide the input type to `createTool<T>()` for full type safety:
   ```typescript
   type MyInput = z.infer<typeof MySchema>
   const handle = createTool<MyInput>({ ... })
   ```

5. **Error Handling**: Return `ToolResponse.error()` for business logic errors. The SDK handles exceptions automatically.

## Examples

See the [examples directory](https://github.com/fastertools/ftl-mcp/tree/main/examples/demo) for complete examples:

- `echo-simplified-ts`: Minimal echo tool using Zod
- `calculator-ts`: Complex validation with Zod refinements
- `weather-ts`: External API integration

## License

Apache-2.0