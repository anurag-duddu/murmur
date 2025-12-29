/**
 * Murmur Proxy - Cloudflare Worker
 *
 * Proxies requests to Groq API, adding the API key server-side.
 * This keeps the API key secure and allows the app to work without
 * users needing to manage their own keys.
 *
 * Security measures:
 * - HMAC-based request signing (timestamp + nonce + body hash)
 * - Rate limiting via Cloudflare dashboard rules (Dashboard > Security > WAF)
 * - Request size limits (10MB max)
 * - Privacy-compliant logging (IPs are hashed, not stored raw)
 * - Input validation for request bodies
 */

interface Env {
  GROQ_API_KEY: string;
  MURMUR_APP_SECRET: string; // HMAC signing secret shared between app and proxy
  IP_HASH_SALT?: string; // Optional salt for IP hashing (rotatable)
}

// Groq API endpoints we proxy
const GROQ_ENDPOINTS = {
  whisper: 'https://api.groq.com/openai/v1/audio/transcriptions',
  chat: 'https://api.groq.com/openai/v1/chat/completions',
};

// Max request size (10MB for audio files)
const MAX_REQUEST_SIZE = 10 * 1024 * 1024;

// Rate limiting is configured via Cloudflare dashboard rules (Workers are stateless,
// so in-memory rate limiting doesn't work across isolates). Configure rate limiting
// at: Dashboard > Security > WAF > Rate limiting rules

/**
 * Hash an IP address for privacy-compliant logging.
 * Uses a non-reversible hash so we can detect patterns without storing raw IPs.
 * Salt is configurable via environment variable for rotation.
 */
async function hashIP(ip: string, env: Env): Promise<string> {
  const salt = env.IP_HASH_SALT || 'murmur-default-salt';
  const encoder = new TextEncoder();
  const data = encoder.encode(salt + ip);
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  // Return first 16 chars of hex hash (better uniqueness, not reversible)
  return hashArray.slice(0, 8).map(b => b.toString(16).padStart(2, '0')).join('');
}

/**
 * Verify HMAC signature for request authentication.
 *
 * For JSON endpoints (chat): Full verification with body hash
 * Signature format: HMAC-SHA256(timestamp + ":" + nonce + ":" + bodyHash, secret)
 *
 * For multipart endpoints (whisper): Verify signature with audio file hash
 * The client signs the raw audio bytes, not the multipart-encoded body.
 * We extract the audio file from the multipart and verify against that.
 *
 * Headers: X-Murmur-Timestamp, X-Murmur-Nonce, X-Murmur-Signature
 */
async function verifyHmacSignature(
  request: Request,
  bodyBytes: ArrayBuffer,
  env: Env,
  isMultipart: boolean
): Promise<{ valid: boolean; error?: string }> {
  const timestamp = request.headers.get('X-Murmur-Timestamp');
  const nonce = request.headers.get('X-Murmur-Nonce');
  const signature = request.headers.get('X-Murmur-Signature');

  if (!timestamp || !nonce || !signature) {
    return { valid: false, error: 'Missing authentication headers' };
  }

  // Check timestamp is within 5 minutes (prevent replay attacks)
  const requestTime = parseInt(timestamp, 10);
  const now = Math.floor(Date.now() / 1000);
  const MAX_AGE_SECONDS = 300; // 5 minutes

  if (isNaN(requestTime) || Math.abs(now - requestTime) > MAX_AGE_SECONDS) {
    return { valid: false, error: 'Request timestamp expired or invalid' };
  }

  let bodyHash: string;

  if (isMultipart) {
    // For multipart/form-data, extract the audio file and hash it
    // The client signed the raw audio bytes
    try {
      const formData = await new Request(request.url, {
        method: 'POST',
        headers: request.headers,
        body: bodyBytes,
      }).formData();

      const audioFile = formData.get('file') as File | null;
      if (!audioFile || typeof audioFile === 'string') {
        return { valid: false, error: 'Missing audio file in request' };
      }

      const audioBytes = await audioFile.arrayBuffer();
      const audioHashBuffer = await crypto.subtle.digest('SHA-256', audioBytes);
      bodyHash = Array.from(new Uint8Array(audioHashBuffer))
        .map(b => b.toString(16).padStart(2, '0'))
        .join('');
    } catch (e) {
      return { valid: false, error: 'Failed to parse multipart request' };
    }
  } else {
    // For JSON, hash the entire body
    const bodyHashBuffer = await crypto.subtle.digest('SHA-256', bodyBytes);
    bodyHash = Array.from(new Uint8Array(bodyHashBuffer))
      .map(b => b.toString(16).padStart(2, '0'))
      .join('');
  }

  // Build the message to sign
  const message = `${timestamp}:${nonce}:${bodyHash}`;

  // Compute expected HMAC
  const encoder = new TextEncoder();
  const keyData = encoder.encode(env.MURMUR_APP_SECRET);
  const key = await crypto.subtle.importKey(
    'raw',
    keyData,
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign']
  );

  const expectedSigBuffer = await crypto.subtle.sign('HMAC', key, encoder.encode(message));
  const expectedSignature = Array.from(new Uint8Array(expectedSigBuffer))
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');

  // Constant-time comparison (prevent timing attacks)
  if (signature.length !== expectedSignature.length) {
    return { valid: false, error: 'Invalid signature' };
  }

  let mismatch = 0;
  for (let i = 0; i < signature.length; i++) {
    mismatch |= signature.charCodeAt(i) ^ expectedSignature.charCodeAt(i);
  }

  if (mismatch !== 0) {
    return { valid: false, error: 'Invalid signature' };
  }

  return { valid: true };
}

/**
 * Validate request content type and basic structure
 */
function validateContentType(contentType: string | null, path: string): { valid: boolean; error?: string } {
  if (!contentType) {
    return { valid: false, error: 'Missing Content-Type header' };
  }

  if (path === '/v1/audio/transcriptions' || path === '/whisper') {
    // Audio endpoint accepts multipart/form-data
    if (!contentType.includes('multipart/form-data')) {
      return { valid: false, error: 'Invalid Content-Type for audio endpoint' };
    }
  } else if (path === '/v1/chat/completions' || path === '/chat') {
    // Chat endpoint accepts application/json
    if (!contentType.includes('application/json')) {
      return { valid: false, error: 'Invalid Content-Type for chat endpoint' };
    }
  }

  return { valid: true };
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

    // Get client IP for logging
    const clientIP = request.headers.get('CF-Connecting-IP') || 'unknown';
    const hashedIP = await hashIP(clientIP, env);

    // ===== SECURITY: Check request size before reading body =====
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

    // ===== SECURITY: Validate content type =====
    const contentType = request.headers.get('Content-Type');
    const contentValidation = validateContentType(contentType, path);
    if (!contentValidation.valid) {
      console.error(`Invalid content type from ${hashedIP}: ${contentValidation.error}`);
      return jsonError(contentValidation.error || 'Invalid request', 400);
    }

    // ===== SECURITY: Clone request and read body for HMAC verification =====
    const bodyBytes = await request.arrayBuffer();
    const isMultipart = contentType?.includes('multipart/form-data') ?? false;

    // ===== SECURITY: Verify HMAC signature =====
    const signatureResult = await verifyHmacSignature(request, bodyBytes, env, isMultipart);
    if (!signatureResult.valid) {
      console.error(`Auth failed from ${hashedIP}: ${signatureResult.error}`);
      return jsonError('Unauthorized', 401);
    }

    try {
      // Forward the request to Groq with our API key
      const groqResponse = await fetch(targetUrl, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${env.GROQ_API_KEY}`,
          'Content-Type': contentType || 'application/json',
        },
        body: bodyBytes,
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
      console.error(`Proxy error for ${hashedIP}:`, error);
      return jsonError('Proxy request failed', 502);
    }
  },
};

function corsHeaders(): HeadersInit {
  // Note: Using '*' for origin since this is a desktop app proxy.
  // The HMAC signature provides the actual authentication.
  // Browser-based attacks are mitigated by requiring valid HMAC signatures.
  return {
    'Access-Control-Allow-Origin': '*',
    'Access-Control-Allow-Methods': 'POST, OPTIONS',
    'Access-Control-Allow-Headers': 'Content-Type, X-Murmur-Timestamp, X-Murmur-Nonce, X-Murmur-Signature',
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
