import { createTool, ToolResponse } from 'ftl-sdk'
import { z } from 'zod'

// Define the schema using Zod
const ToolSchema = z.object({
  message: z.string().describe('The input message to process')
})

// Derive TypeScript type from the schema
type ToolInput = z.infer<typeof ToolSchema>

const tool = createTool<ToolInput>({
  metadata: {
    name: 'echo_ts',
    title: 'echo-ts',
    description: 'An MCP tool written in TypeScript',
    inputSchema: z.toJSONSchema(ToolSchema)
  },
  handler: async (input) => {
    // TODO: Implement your tool logic here
    return ToolResponse.text(`Processed: ${input.message}`)
  }
})

//@ts-ignore
addEventListener('fetch', (event: FetchEvent) => {
  event.respondWith(tool(event.request))
})