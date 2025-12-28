/**
 * Murmur Proxy - Cloudflare Worker
 *
 * Proxies requests to Groq API, adding the API key server-side.
 * This keeps the API key secure and allows the app to work without
 * users needing to manage their own keys.
 *
 * Security measures:
 * - App signature verification (X-Murmur-Signature header)
 * - Rate limiting per IP (100 requests/minute)
 * - Request size limits
 */

interface Env {
  GROQ_API_KEY: string;
  MURMUR_APP_SECRET: string; // Secret shared between app and proxy
}

// Groq API endpoints we proxy
const GROQ_ENDPOINTS = {
  whisper: 'https://api.groq.com/openai/v1/audio/transcriptions',
  chat: 'https://api.groq.com/openai/v1/chat/completions',
};

// Rate limiting: track requests per IP
const rateLimitMap = new Map<string, { count: number; resetTime: number }>();
const RATE_LIMIT = 100; // requests per minute
const RATE_WINDOW = 60 * 1000; // 1 minute in ms

// Max request size (10MB for audio files)
const MAX_REQUEST_SIZE = 10 * 1024 * 1024;

function checkRateLimit(ip: string): boolean {
  const now = Date.now();
  const record = rateLimitMap.get(ip);

  if (!record || now > record.resetTime) {
    rateLimitMap.set(ip, { count: 1, resetTime: now + RATE_WINDOW });
    return true;
  }

  if (record.count >= RATE_LIMIT) {
    return false;
  }

  record.count++;
  return true;
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    // Handle CORS preflight
    if (request.method === 'OPTIONS') {
      return new Response(null, {
        headers: corsHeaders(),
      });
    }

    // Only allow POST requests
    if (request.method !== 'POST') {
      return jsonError('Method not allowed', 405);
    }

    // Check API key is configured
    if (!env.GROQ_API_KEY) {
      console.error('GROQ_API_KEY not configured');
      return jsonError('Proxy not configured', 500);
    }

    // ===== SECURITY: Verify app signature =====
    const signature = request.headers.get('X-Murmur-Signature');
    if (!signature || signature !== env.MURMUR_APP_SECRET) {
      console.error('Invalid or missing app signature');
      return jsonError('Unauthorized', 401);
    }

    // ===== SECURITY: Rate limiting =====
    const clientIP = request.headers.get('CF-Connecting-IP') || 'unknown';
    if (!checkRateLimit(clientIP)) {
      console.error(`Rate limit exceeded for IP: ${clientIP}`);
      return jsonError('Rate limit exceeded', 429);
    }

    // ===== SECURITY: Check request size =====
    const contentLength = request.headers.get('Content-Length');
    if (contentLength && parseInt(contentLength) > MAX_REQUEST_SIZE) {
      return jsonError('Request too large', 413);
    }

    // Parse the URL path to determine which endpoint to use
    const url = new URL(request.url);
    const path = url.pathname;

    let targetUrl: string;
    if (path === '/v1/audio/transcriptions' || path === '/whisper') {
      targetUrl = GROQ_ENDPOINTS.whisper;
    } else if (path === '/v1/chat/completions' || path === '/chat') {
      targetUrl = GROQ_ENDPOINTS.chat;
    } else {
      return jsonError('Unknown endpoint', 404);
    }

    try {
      // Forward the request to Groq with our API key
      const groqResponse = await fetch(targetUrl, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${env.GROQ_API_KEY}`,
          // Preserve content-type from original request
          'Content-Type': request.headers.get('Content-Type') || 'application/json',
        },
        body: request.body,
      });

      // Return the response with CORS headers
      const responseBody = await groqResponse.text();

      return new Response(responseBody, {
        status: groqResponse.status,
        headers: {
          ...corsHeaders(),
          'Content-Type': groqResponse.headers.get('Content-Type') || 'application/json',
        },
      });
    } catch (error) {
      console.error('Proxy error:', error);
      return jsonError('Proxy request failed', 502);
    }
  },
};

function corsHeaders(): HeadersInit {
  return {
    'Access-Control-Allow-Origin': '*',
    'Access-Control-Allow-Methods': 'POST, OPTIONS',
    'Access-Control-Allow-Headers': 'Content-Type, Authorization, X-Murmur-Signature',
    'Access-Control-Max-Age': '86400',
  };
}

function jsonError(message: string, status: number): Response {
  return new Response(JSON.stringify({ error: message }), {
    status,
    headers: {
      ...corsHeaders(),
      'Content-Type': 'application/json',
    },
  });
}
