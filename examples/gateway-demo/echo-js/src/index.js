
// For AutoRouter documentation refer to https://itty.dev/itty-router/routers/autorouter
import { AutoRouter } from 'itty-router';

let router = AutoRouter();

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
            }
        };
        
        return new Response(JSON.stringify(metadata), {
            headers: {
                'Content-Type': 'application/json',
            },
        });
    })
    .post('/', async (request) => {
        const body = await request.json();
        return new Response(JSON.stringify({ echo: body.message }), {
            headers: {
                'Content-Type': 'application/json',
            },
        });
    })

addEventListener('fetch', (event) => {
    event.respondWith(router.fetch(event.request));
});

