import { z } from 'zod'

const schema = z.object({
  message: z.string()
})

const jsonSchema = z.toJSONSchema(schema)
type JsonSchemaType = typeof jsonSchema

// This will show the type in the TypeScript error
const test: never = jsonSchema