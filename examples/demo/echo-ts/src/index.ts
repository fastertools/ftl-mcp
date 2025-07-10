import { createTool, ToolResponse } from 'ftl-sdk'
import { z } from 'zod'

// Define the schema using Zod
const EchoInputSchema = z.object({
  message: z.string().describe('The message to echo back')
})

// Derive TypeScript type from the schema
type EchoInput = z.infer<typeof EchoInputSchema>

const echo = createTool<EchoInput>({
  metadata: {
    name: 'echo_ts',
    title: 'Simplified Echo Tool (TypeScript)',
    description: 'A minimal echo tool using Zod for schema definition',
    // Use Zod v4's native JSON Schema conversion
    inputSchema: z.toJSONSchema(EchoInputSchema)
  },
  handler: async (input) => {
    // Input is already validated by the gateway
    // TypeScript knows that input.message is a string!
    return ToolResponse.text(`Echo: ${input.message}`)
  }
})

//@ts-ignore
addEventListener('fetch', (event: FetchEvent) => {
  event.respondWith(echo(event.request))
})