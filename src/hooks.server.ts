import type { Handle } from '@sveltejs/kit';

/**
 * This hook intercepts requests to the SvelteKit server.
 * It's used here to proxy API requests to the backend server
 * during development. This is the recommended SvelteKit way to handle
 * API proxying, as it avoids conflicts with SvelteKit's own router.
 */
export const handle: Handle = async ({ event, resolve }) => {
	if (event.url.pathname.startsWith('/api')) {
		const targetUrl = new URL(event.url.pathname + event.url.search, 'http://localhost:7370');
		console.log(`[API Proxy] Intercepted request for: ${event.url.pathname}. Forwarding to: ${targetUrl}`);

		const headers = new Headers(event.request.headers);
		headers.delete('host');

		const isGetOrHead = event.request.method === 'GET' || event.request.method === 'HEAD';

		try {
			const response = await fetch(targetUrl.toString(), {
				method: event.request.method,
				headers: headers,
				body: isGetOrHead ? undefined : event.request.body,
				// The 'duplex' property is only required for streaming request bodies.
				...(isGetOrHead ? {} : { duplex: 'half' })
			});

			console.log(`[API Proxy] Backend responded to ${event.url.pathname} with status: ${response.status}`);

			return response;
		} catch (error) {
			console.error(`[API Proxy] Error fetching ${targetUrl.toString()}:`, error);
			return new Response('API proxy error', { status: 502 });
		}
	}

	return resolve(event);
};