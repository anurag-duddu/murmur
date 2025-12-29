//! HMAC-based request signing for proxy authentication.
//!
//! This module provides secure request signing to authenticate with the proxy.
//! The signature prevents replay attacks and ensures request integrity.

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// The shared secret for HMAC signing.
/// This must match the MURMUR_APP_SECRET in the proxy's environment.
/// In production, this should be a strong, randomly generated secret.
const HMAC_SECRET: &str = "d8a30062682b1fb12471dfd838779a7b6047e04a54e8ce0d440db87c50eb2411";

/// Proxy URLs
pub const PROXY_URL_WHISPER: &str = "https://murmur-proxy.anurag-ebc.workers.dev/whisper";
pub const PROXY_URL_CHAT: &str = "https://murmur-proxy.anurag-ebc.workers.dev/chat";

/// Direct API URLs (for development with local API key)
pub const DIRECT_API_URL_WHISPER: &str = "https://api.groq.com/openai/v1/audio/transcriptions";
pub const DIRECT_API_URL_CHAT: &str = "https://api.groq.com/openai/v1/chat/completions";

/// Get API configuration based on build type.
///
/// - **Debug builds** (`npm run tauri dev`): Use direct API with GROQ_API_KEY from environment
/// - **Release builds** (`npm run tauri build`): Always use proxy with HMAC auth
///
/// Returns (whisper_url, chat_url, Option<api_key>)
pub fn get_api_config() -> (&'static str, &'static str, Option<String>) {
    #[cfg(debug_assertions)]
    {
        // Dev mode: use direct API if GROQ_API_KEY is available
        if let Ok(key) = std::env::var("GROQ_API_KEY") {
            if !key.is_empty() {
                println!("[DEV] Using direct Groq API");
                return (DIRECT_API_URL_WHISPER, DIRECT_API_URL_CHAT, Some(key));
            }
        }
        println!("[DEV] No GROQ_API_KEY, falling back to proxy");
        (PROXY_URL_WHISPER, PROXY_URL_CHAT, None)
    }

    #[cfg(not(debug_assertions))]
    {
        // Release mode: always use proxy
        println!("[PROD] Using proxy");
        (PROXY_URL_WHISPER, PROXY_URL_CHAT, None)
    }
}

/// Generate a cryptographically secure nonce using a CSPRNG.
/// The nonce is 32 random bytes (256 bits) hex-encoded.
pub fn generate_nonce() -> String {
    use rand::rngs::OsRng;
    use rand::RngCore;

    // Use OS-provided CSPRNG for cryptographically secure randomness
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);

    // Hex-encode the random bytes
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Compute SHA-256 hash of data and return as hex string
pub fn sha256_hex(data: &[u8]) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Sign a request for proxy authentication.
///
/// Returns (timestamp, nonce, signature) to be added as headers:
/// - X-Murmur-Timestamp: Unix timestamp
/// - X-Murmur-Nonce: Random nonce for replay protection
/// - X-Murmur-Signature: HMAC-SHA256 signature
pub fn sign_request(body: &[u8]) -> (String, String, String) {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Get current timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let timestamp_str = timestamp.to_string();

    // Generate nonce
    let nonce = generate_nonce();

    // Compute body hash
    let body_hash = sha256_hex(body);

    // Build message: timestamp:nonce:bodyHash
    let message = format!("{}:{}:{}", timestamp_str, nonce, body_hash);

    // Compute HMAC signature
    let mut mac = HmacSha256::new_from_slice(HMAC_SECRET.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(message.as_bytes());
    let result = mac.finalize();
    let signature: String = result.into_bytes().iter().map(|b| format!("{:02x}", b)).collect();

    (timestamp_str, nonce, signature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_request_produces_valid_output() {
        let body = b"test body";
        let (timestamp, nonce, signature) = sign_request(body);

        // Timestamp should be numeric
        assert!(timestamp.parse::<u64>().is_ok());

        // Nonce should be non-empty hex
        assert!(!nonce.is_empty());

        // Signature should be 64 hex chars (256 bits)
        assert_eq!(signature.len(), 64);
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sha256_hex() {
        // Known SHA-256 hash of "hello"
        let hash = sha256_hex(b"hello");
        assert_eq!(hash, "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
    }

    #[test]
    fn test_different_bodies_produce_different_signatures() {
        let (_, _, sig1) = sign_request(b"body1");
        let (_, _, sig2) = sign_request(b"body2");
        assert_ne!(sig1, sig2);
    }
}
