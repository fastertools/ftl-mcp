import { createTool, ToolResponse } from 'ftl-sdk'
import { z } from 'zod'

// Define the operation enum with Zod
const Operation = z.enum(['add', 'subtract', 'multiply', 'divide', 'power', 'sqrt'])

// Define the calculator input schema
const CalculatorSchema = z.object({
  operation: Operation,
  a: z.number().describe('First operand'),
  b: z.number().optional().describe('Second operand (not needed for sqrt)')
}).refine(
  (data) => data.operation !== 'sqrt' || data.b === undefined,
  { message: "sqrt operation only takes one operand" }
).refine(
  (data) => data.operation === 'sqrt' || data.b !== undefined,
  { message: "This operation requires two operands" }
).refine(
  (data) => data.operation !== 'divide' || data.b !== 0,
  { message: "Cannot divide by zero" }
)

// Derive TypeScript type from the schema
type CalculatorRequest = z.infer<typeof CalculatorSchema>

const handle = createTool<CalculatorRequest>({
  metadata: {
    name: 'calculator_ts',
    title: 'Advanced Calculator',
    description: 'Performs mathematical operations with comprehensive validation',
    // Use Zod v4's native JSON Schema conversion - all validation rules are preserved!
    inputSchema: z.toJSONSchema(CalculatorSchema),
    annotations: {
      readOnlyHint: true,
      idempotentHint: true
    }
  },
  handler: async (input) => {
    // Input is already validated by the gateway against the JSON Schema
    // All our Zod refinements (divide by zero, operand requirements) are enforced!
    let result: number
    
    switch (input.operation) {
      case 'add':
        result = input.a + input.b!
        break
      case 'subtract':
        result = input.a - input.b!
        break
      case 'multiply':
        result = input.a * input.b!
        break
      case 'divide':
        result = input.a / input.b!
        break
      case 'power':
        result = Math.pow(input.a, input.b!)
        break
      case 'sqrt':
        if (input.a < 0) {
          return ToolResponse.error('Cannot take square root of negative number')
        }
        result = Math.sqrt(input.a)
        break
      default:
        // This should never happen due to type checking
        return ToolResponse.error('Invalid operation')
    }
    
    // Format the response based on operation
    const operationText = input.operation === 'sqrt' 
      ? `√${input.a} = ${result}`
      : `${input.a} ${getOperatorSymbol(input.operation)} ${input.b} = ${result}`
    
    return ToolResponse.withStructured(
      operationText,
      {
        operation: input.operation,
        operands: input.operation === 'sqrt' ? [input.a] : [input.a, input.b],
        result
      }
    )
  }
})

function getOperatorSymbol(op: z.infer<typeof Operation>): string {
  switch (op) {
    case 'add': return '+'
    case 'subtract': return '-'
    case 'multiply': return '×'
    case 'divide': return '÷'
    case 'power': return '^'
    default: return '?'
  }
}

//@ts-ignore
addEventListener('fetch', (event: FetchEvent) => {
  event.respondWith(handle(event.request))
})