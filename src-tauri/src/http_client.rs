//! Shared HTTP client configuration with secure TLS settings.
//!
//! This module provides a pre-configured HTTP client with:
//! - Native TLS (macOS Security.framework / Windows SChannel)
//! - HTTPS-only enforcement
//! - Reasonable timeouts
//! - Connection pooling and reuse via global cached clients

use reqwest::Client;
use std::sync::OnceLock;
use std::time::Duration;

/// Default timeout for API requests (30 seconds)
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Extended timeout for audio transcription (120 seconds)
const TRANSCRIPTION_TIMEOUT_SECS: u64 = 120;

/// Global cached client for standard API calls (30s timeout)
static CACHED_CLIENT: OnceLock<Client> = OnceLock::new();

/// Global cached client for transcription (120s timeout)
static CACHED_TRANSCRIPTION_CLIENT: OnceLock<Client> = OnceLock::new();

/// Get the shared secure HTTP client (30s timeout).
/// This reuses connections across requests for better performance.
pub fn get_client() -> Result<&'static Client, String> {
    Ok(CACHED_CLIENT.get_or_init(|| {
        Client::builder()
            .use_native_tls()
            .https_only(true)
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .expect("Failed to create HTTP client - this should never happen")
    }))
}

/// Get the shared transcription client (120s timeout).
/// This reuses connections for transcription requests.
pub fn get_transcription_client() -> Result<&'static Client, String> {
    Ok(CACHED_TRANSCRIPTION_CLIENT.get_or_init(|| {
        Client::builder()
            .use_native_tls()
            .https_only(true)
            .timeout(Duration::from_secs(TRANSCRIPTION_TIMEOUT_SECS))
            .build()
            .expect("Failed to create HTTP client - this should never happen")
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_client() {
        let client = get_client();
        assert!(client.is_ok(), "Failed to get client: {:?}", client);
    }

    #[test]
    fn test_get_transcription_client() {
        let client = get_transcription_client();
        assert!(
            client.is_ok(),
            "Failed to get transcription client: {:?}",
            client
        );
    }

    #[tokio::test]
    async fn test_https_only_enforcement() {
        let client = get_client().unwrap();

        // HTTP request should fail (HTTPS-only)
        let result = client.get("http://example.com").send().await;
        assert!(
            result.is_err(),
            "HTTP request should fail with HTTPS-only client"
        );
    }
}
