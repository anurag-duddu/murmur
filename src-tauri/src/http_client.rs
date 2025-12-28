//! Shared HTTP client configuration with secure TLS settings.
//!
//! This module provides a pre-configured HTTP client with:
//! - Explicit TLS validation using rustls
//! - HTTPS-only enforcement
//! - Reasonable timeouts
//! - Connection pooling

use reqwest::Client;
use std::time::Duration;

/// Default timeout for API requests (30 seconds)
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Create a secure HTTP client with TLS validation.
///
/// This client is configured with:
/// - Modern TLS (rustls) with built-in root certificates
/// - HTTPS-only mode (HTTP requests will fail)
/// - 30-second timeout for requests
///
/// # Errors
/// Returns an error if the client cannot be built (rare, usually system configuration issues).
pub fn create_secure_client() -> Result<Client, String> {
    Client::builder()
        .use_rustls_tls()
        .tls_built_in_root_certs(true)
        .https_only(true)
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("Failed to create secure HTTP client: {}", e))
}

/// Create a secure HTTP client with a custom timeout.
///
/// # Arguments
/// * `timeout_secs` - Timeout in seconds for requests
///
/// # Errors
/// Returns an error if the client cannot be built.
pub fn create_secure_client_with_timeout(timeout_secs: u64) -> Result<Client, String> {
    Client::builder()
        .use_rustls_tls()
        .tls_built_in_root_certs(true)
        .https_only(true)
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| format!("Failed to create secure HTTP client: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_secure_client() {
        let client = create_secure_client();
        assert!(client.is_ok(), "Failed to create client: {:?}", client);
    }

    #[test]
    fn test_create_secure_client_with_timeout() {
        let client = create_secure_client_with_timeout(60);
        assert!(client.is_ok(), "Failed to create client: {:?}", client);
    }

    #[tokio::test]
    async fn test_https_only_enforcement() {
        let client = create_secure_client().unwrap();

        // HTTP request should fail (HTTPS-only)
        let result = client.get("http://example.com").send().await;
        assert!(result.is_err(), "HTTP request should fail with HTTPS-only client");
    }
}
