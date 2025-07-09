import { describe, test, expect } from 'vitest'
import type {
  ToolMetadata,
  TextContent,
  ToolResponse as ToolResponseType,
  ResourceContent,
} from '../src/index'
import {
  ToolResponse,
  ToolContent,
  isTextContent,
  isImageContent,
  isAudioContent,
  isResourceContent,
} from '../src/index'

describe('ToolResponse convenience methods', () => {
  test('text() creates a simple text response', () => {
    const response = ToolResponse.text('Hello, world!')

    expect(response.content).toHaveLength(1)
    expect(response.content[0].type).toBe('text')
    expect((response.content[0] as TextContent).text).toBe('Hello, world!')
    expect(response.isError).toBeUndefined()
    expect(response.structuredContent).toBeUndefined()
  })

  test('error() creates an error response', () => {
    const response = ToolResponse.error('Something went wrong')

    expect(response.content).toHaveLength(1)
    expect(response.content[0].type).toBe('text')
    expect((response.content[0] as TextContent).text).toBe('Something went wrong')
    expect(response.isError).toBe(true)
  })

  test('withStructured() creates a response with structured content', () => {
    const structured = { result: 42, status: 'success' }
    const response = ToolResponse.withStructured('Operation complete', structured)

    expect(response.content).toHaveLength(1)
    expect((response.content[0] as TextContent).text).toBe('Operation complete')
    expect(response.structuredContent).toEqual(structured)
    expect(response.isError).toBeUndefined()
  })
})

describe('ToolContent convenience methods', () => {
  test('text() creates text content', () => {
    const content = ToolContent.text('Sample text')

    expect(content.type).toBe('text')
    expect(content.text).toBe('Sample text')
    expect(content.annotations).toBeUndefined()
  })

  test('text() with annotations', () => {
    const annotations = { audience: ['developers'], priority: 0.8 }
    const content = ToolContent.text('Sample text', annotations)

    expect(content.annotations).toEqual(annotations)
  })

  test('image() creates image content', () => {
    const content = ToolContent.image('base64data', 'image/png')

    expect(content.type).toBe('image')
    expect(content.data).toBe('base64data')
    expect(content.mimeType).toBe('image/png')
  })

  test('audio() creates audio content', () => {
    const content = ToolContent.audio('audiodata', 'audio/mp3')

    expect(content.type).toBe('audio')
    expect(content.data).toBe('audiodata')
    expect(content.mimeType).toBe('audio/mp3')
  })

  test('resource() creates resource content', () => {
    const resource = {
      uri: 'file:///example.txt',
      mimeType: 'text/plain',
      text: 'File contents',
    }
    const content = ToolContent.resource(resource)

    expect(content.type).toBe('resource')
    expect(content.resource).toEqual(resource)
  })
})

describe('Type guards', () => {
  test('isTextContent identifies text content', () => {
    const text = ToolContent.text('hello')
    const image = ToolContent.image('data', 'image/png')

    expect(isTextContent(text)).toBe(true)
    expect(isTextContent(image)).toBe(false)
  })

  test('isImageContent identifies image content', () => {
    const text = ToolContent.text('hello')
    const image = ToolContent.image('data', 'image/png')

    expect(isImageContent(text)).toBe(false)
    expect(isImageContent(image)).toBe(true)
  })

  test('isAudioContent identifies audio content', () => {
    const audio = ToolContent.audio('data', 'audio/mp3')
    const text = ToolContent.text('hello')

    expect(isAudioContent(audio)).toBe(true)
    expect(isAudioContent(text)).toBe(false)
  })

  test('isResourceContent identifies resource content', () => {
    const resource = ToolContent.resource({ uri: 'file://test' })
    const text = ToolContent.text('hello')

    expect(isResourceContent(resource)).toBe(true)
    expect(isResourceContent(text)).toBe(false)
  })
})

describe('ToolMetadata structure', () => {
  test('minimal metadata has required fields', () => {
    const metadata: ToolMetadata = {
      name: 'test-tool',
      inputSchema: {
        type: 'object',
        properties: {},
      },
    }

    expect(metadata.name).toBe('test-tool')
    expect(metadata.inputSchema).toBeDefined()
    expect(metadata.title).toBeUndefined()
    expect(metadata.description).toBeUndefined()
  })

  test('full metadata with all optional fields', () => {
    const metadata: ToolMetadata = {
      name: 'test-tool',
      title: 'Test Tool',
      description: 'A tool for testing',
      inputSchema: {
        type: 'object',
        properties: {
          input: { type: 'string' },
        },
        required: ['input'],
      },
      outputSchema: {
        type: 'object',
        properties: {
          result: { type: 'string' },
        },
      },
      annotations: {
        readOnlyHint: true,
        idempotentHint: true,
        destructiveHint: false,
      },
      _meta: {
        version: '1.0.0',
      },
    }

    expect(metadata.title).toBe('Test Tool')
    expect(metadata.annotations?.readOnlyHint).toBe(true)
    expect(metadata._meta?.version).toBe('1.0.0')
  })
})

describe('JSON serialization', () => {
  test('ToolResponse serializes correctly', () => {
    const response: ToolResponseType = {
      content: [
        {
          type: 'text',
          text: 'Hello',
        },
      ],
      structuredContent: { foo: 'bar' },
      isError: false,
    }

    const json = JSON.stringify(response)
    const parsed = JSON.parse(json) as ToolResponseType

    expect(parsed.content).toHaveLength(1)
    expect(parsed.content[0].type).toBe('text')
    expect(parsed.structuredContent).toEqual({ foo: 'bar' })
    expect(parsed.isError).toBe(false)
  })

  test('Complex nested content serializes correctly', () => {
    const response: ToolResponseType = {
      content: [
        {
          type: 'text',
          text: 'First',
          annotations: {
            audience: ['users'],
            priority: 1.0,
          },
        },
        {
          type: 'image',
          data: 'imagedata',
          mimeType: 'image/jpeg',
        },
        {
          type: 'resource',
          resource: {
            uri: 'https://example.com/data',
            mimeType: 'application/json',
            text: '{"key": "value"}',
          },
        },
      ],
    }

    const json = JSON.stringify(response)
    const parsed = JSON.parse(json) as ToolResponseType

    expect(parsed.content).toHaveLength(3)
    expect(parsed.content[0].type).toBe('text')
    expect(parsed.content[1].type).toBe('image')
    expect(parsed.content[2].type).toBe('resource')

    const textContent = parsed.content[0] as TextContent
    expect(textContent.annotations?.audience).toEqual(['users'])

    const resourceContent = parsed.content[2] as ResourceContent
    expect(resourceContent.resource.uri).toBe('https://example.com/data')
  })
})
