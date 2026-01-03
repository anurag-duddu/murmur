//! PKCE (Proof Key for Code Exchange) implementation for OAuth 2.0 security.
//!
//! PKCE prevents authorization code interception attacks by requiring the client
//! to prove it initiated the authorization request.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::{Digest, Sha256};

use super::types::PkceChallenge;

/// Generate a new PKCE challenge for OAuth flow.
///
/// Creates:
/// - `verifier`: 32 bytes of cryptographically secure random data, base64url encoded
/// - `challenge`: SHA-256 hash of verifier, base64url encoded
/// - `state`: 16 bytes of random data for CSRF protection, base64url encoded
pub fn generate_pkce() -> PkceChallenge {
    // Generate 32 bytes (256 bits) of random data for verifier
    let mut verifier_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut verifier_bytes);
    let verifier = URL_SAFE_NO_PAD.encode(verifier_bytes);

    // SHA-256 hash the verifier to get challenge (S256 method)
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

    // Generate state for CSRF protection
    let mut state_bytes = [0u8; 16];
    OsRng.fill_bytes(&mut state_bytes);
    let state = URL_SAFE_NO_PAD.encode(state_bytes);

    PkceChallenge {
        verifier,
        challenge,
        state,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_generation() {
        let pkce = generate_pkce();

        // Verifier should be 43 characters (32 bytes base64url encoded without padding)
        assert_eq!(pkce.verifier.len(), 43);

        // Challenge should be 43 characters (SHA-256 = 32 bytes base64url encoded)
        assert_eq!(pkce.challenge.len(), 43);

        // State should be 22 characters (16 bytes base64url encoded without padding)
        assert_eq!(pkce.state.len(), 22);

        // Each generation should produce unique values
        let pkce2 = generate_pkce();
        assert_ne!(pkce.verifier, pkce2.verifier);
        assert_ne!(pkce.challenge, pkce2.challenge);
        assert_ne!(pkce.state, pkce2.state);
    }

    #[test]
    fn test_challenge_derivation() {
        let pkce = generate_pkce();

        // Verify that challenge is correctly derived from verifier
        let mut hasher = Sha256::new();
        hasher.update(pkce.verifier.as_bytes());
        let expected_challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

        assert_eq!(pkce.challenge, expected_challenge);
    }
}
