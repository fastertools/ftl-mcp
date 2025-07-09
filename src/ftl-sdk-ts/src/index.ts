/**
 * Thin SDK providing MCP protocol types for FTL tool development.
 *
 * This package provides only the type definitions needed to implement
 * MCP-compliant tools. It does not include any HTTP server logic,
 * allowing you to use any web framework of your choice.
 */

/**
 * Tool metadata returned by GET requests to tool endpoints
 */
export interface ToolMetadata {
  /** The name of the tool (must be unique within the gateway) */
  name: string

  /** Optional human-readable title for the tool */
  title?: string

  /** Optional description of what the tool does */
  description?: string

  /** JSON Schema describing the expected input parameters */
  inputSchema: Record<string, unknown>

  /** Optional JSON Schema describing the output format */
  outputSchema?: Record<string, unknown>

  /** Optional annotations providing hints about tool behavior */
  annotations?: ToolAnnotations

  /** Optional metadata for tool-specific extensions */
  _meta?: Record<string, unknown>
}

/**
 * Annotations providing hints about tool behavior
 */
export interface ToolAnnotations {
  /** Optional title annotation */
  title?: string

  /** Hint that the tool is read-only (doesn't modify state) */
  readOnlyHint?: boolean

  /** Hint that the tool may perform destructive operations */
  destructiveHint?: boolean

  /** Hint that the tool is idempotent (same input → same output) */
  idempotentHint?: boolean

  /** Hint that the tool accepts open-world inputs */
  openWorldHint?: boolean
}

/**
 * Response format for tool execution (POST requests)
 */
export interface ToolResponse {
  /** Array of content items returned by the tool */
  content: ToolContent[]

  /** Optional structured content matching the outputSchema */
  structuredContent?: unknown

  /** Indicates if this response represents an error */
  isError?: boolean
}

/**
 * Base type for all content items
 */
export interface BaseContent {
  /** Content type discriminator */
  type: string

  /** Optional annotations for this content */
  annotations?: ContentAnnotations
}

/**
 * Text content
 */
export interface TextContent extends BaseContent {
  type: 'text'

  /** The text content */
  text: string
}

/**
 * Image content
 */
export interface ImageContent extends BaseContent {
  type: 'image'

  /** Base64-encoded image data */
  data: string

  /** MIME type of the image (e.g., "image/png") */
  mimeType: string
}

/**
 * Audio content
 */
export interface AudioContent extends BaseContent {
  type: 'audio'

  /** Base64-encoded audio data */
  data: string

  /** MIME type of the audio (e.g., "audio/wav") */
  mimeType: string
}

/**
 * Resource reference
 */
export interface ResourceContent extends BaseContent {
  type: 'resource'

  /** The resource contents */
  resource: ResourceContents
}

/**
 * Content types that can be returned by tools
 */
export type ToolContent = TextContent | ImageContent | AudioContent | ResourceContent

/**
 * Annotations for content items
 */
export interface ContentAnnotations {
  /** Target audience for this content */
  audience?: string[]

  /** Priority of this content (0.0 to 1.0) */
  priority?: number
}

/**
 * Resource contents for resource-type content
 */
export interface ResourceContents {
  /** URI of the resource */
  uri: string

  /** MIME type of the resource */
  mimeType?: string

  /** Text content of the resource */
  text?: string

  /** Base64-encoded binary content of the resource */
  blob?: string
}

/**
 * Convenience functions for creating responses
 */
export const ToolResponse = {
  /**
   * Create a simple text response
   */
  text(text: string): ToolResponse {
    return {
      content: [
        {
          type: 'text',
          text,
        },
      ],
    }
  },

  /**
   * Create an error response
   */
  error(error: string): ToolResponse {
    return {
      content: [
        {
          type: 'text',
          text: error,
        },
      ],
      isError: true,
    }
  },

  /**
   * Create a response with structured content
   */
  withStructured(text: string, structured: unknown): ToolResponse {
    return {
      content: [
        {
          type: 'text',
          text,
        },
      ],
      structuredContent: structured,
    }
  },
}

/**
 * Convenience functions for creating content items
 */
export const ToolContent = {
  /**
   * Create a text content item
   */
  text(text: string, annotations?: ContentAnnotations): TextContent {
    return {
      type: 'text',
      text,
      annotations,
    }
  },

  /**
   * Create an image content item
   */
  image(data: string, mimeType: string, annotations?: ContentAnnotations): ImageContent {
    return {
      type: 'image',
      data,
      mimeType,
      annotations,
    }
  },

  /**
   * Create an audio content item
   */
  audio(data: string, mimeType: string, annotations?: ContentAnnotations): AudioContent {
    return {
      type: 'audio',
      data,
      mimeType,
      annotations,
    }
  },

  /**
   * Create a resource content item
   */
  resource(resource: ResourceContents, annotations?: ContentAnnotations): ResourceContent {
    return {
      type: 'resource',
      resource,
      annotations,
    }
  },
}

// Type guards for content types
export function isTextContent(content: ToolContent): content is TextContent {
  return content.type === 'text'
}

export function isImageContent(content: ToolContent): content is ImageContent {
  return content.type === 'image'
}

export function isAudioContent(content: ToolContent): content is AudioContent {
  return content.type === 'audio'
}

export function isResourceContent(content: ToolContent): content is ResourceContent {
  return content.type === 'resource'
}
