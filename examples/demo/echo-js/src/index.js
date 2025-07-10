// For AutoRouter documentation refer to https://itty.dev/itty-router/routers/autorouter
import { AutoRouter } from 'itty-router'
import { ToolResponse } from 'ftl-sdk'

let router = AutoRouter()

// Route ordering matters, the first route that matches will be used
// Any route that does not return will be treated as a middleware
// Any unmatched route will return a 404
router
    .get('/', async () => {
        const metadata = {
            name: 'echo-js',
            title: 'JavaScript Echo Tool',
            description: 'Echoes back the input message (JavaScript implementation)',
            inputSchema: {
                type: 'object',
                properties: {
                    message: {
                        type: 'string',
                        description: 'The message to echo back'
                    }
                },
                required: ['message']
            },
            annotations: {
                readOnlyHint: true,
                idempotentHint: true
            }
        }
        
        return new Response(JSON.stringify(metadata), {
            headers: {
                'Content-Type': 'application/json',
            },
        })
    })
    .post('/', async (request) => {
        try {
            const body = await request.json()
            
            // Validate input
            if (!body.message) {
                const errorResponse = ToolResponse.error('Missing required field: message')
                return new Response(JSON.stringify(errorResponse), {
                    status: 400,
                    headers: {
                        'Content-Type': 'application/json',
                    },
                })
            }
            
            // Create MCP-compliant response
            const response = ToolResponse.text(`Echo: ${body.message}`)
            
            return new Response(JSON.stringify(response), {
                headers: {
                    'Content-Type': 'application/json',
                },
            })
        } catch (e) {
            const errorResponse = ToolResponse.error(`Invalid request: ${e.message}`)
            return new Response(JSON.stringify(errorResponse), {
                status: 400,
                headers: {
                    'Content-Type': 'application/json',
                },
            })
        }
    })

addEventListener('fetch', (event) => {
    event.respondWith(router.fetch(event.request))
})