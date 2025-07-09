# ftl-sdk (TypeScript)

Thin SDK providing MCP protocol types for FTL tool development.

## Installation

```bash
npm install ftl-sdk
# or
yarn add ftl-sdk
# or
pnpm add ftl-sdk
```

## Usage

This package provides only type definitions - no HTTP server logic. Use with any web framework:

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

## Convenience Methods

```typescript
import { ToolResponse, ToolContent } from 'ftl-sdk';

// Simple text response
const response = ToolResponse.text('Hello, world!');

// Error response
const response = ToolResponse.error('Something went wrong');

// Response with structured content
const response = ToolResponse.withStructured(
  'Calculation complete',
  { result: 42 }
);

// Creating content items
const textContent = ToolContent.text('Some text');
const imageContent = ToolContent.image(base64Data, 'image/png');
```

## Type Guards

```typescript
import { isTextContent, isImageContent } from 'ftl-sdk';

// Check content types
if (isTextContent(content)) {
  console.log(content.text);
}
```