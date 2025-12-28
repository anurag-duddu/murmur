//! Secure storage for sensitive credentials using the system keychain.
//!
//! This module provides secure storage for API keys and other secrets using
//! the macOS Keychain (via the `keyring` crate). Secrets are never written
//! to disk in plaintext.

use keyring::Entry;

/// Service name used for all keychain entries
const SERVICE_NAME: &str = "com.idstuart.murmur";

/// Keys for different credentials stored in keychain
pub mod keys {
    pub const DEEPGRAM_API_KEY: &str = "deepgram_api_key";
    pub const GROQ_API_KEY: &str = "groq_api_key";
    pub const ANTHROPIC_API_KEY: &str = "anthropic_api_key";
    pub const LICENSE_KEY: &str = "license_key";
}

/// Store a secret in the system keychain.
///
/// # Arguments
/// * `key` - The key name (use constants from `keys` module)
/// * `value` - The secret value to store
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(String)` with error message on failure
pub fn store_secret(key: &str, value: &str) -> Result<(), String> {
    if value.is_empty() {
        // Don't store empty values, delete instead
        return delete_secret(key);
    }

    let entry = Entry::new(SERVICE_NAME, key)
        .map_err(|e| format!("Failed to create keychain entry for '{}': {}", key, e))?;

    entry
        .set_password(value)
        .map_err(|e| format!("Failed to store secret '{}' in keychain: {}", key, e))
}

/// Retrieve a secret from the system keychain.
///
/// # Arguments
/// * `key` - The key name (use constants from `keys` module)
///
/// # Returns
/// * `Some(String)` with the secret value if found
/// * `None` if not found or on error
pub fn get_secret(key: &str) -> Option<String> {
    let entry = Entry::new(SERVICE_NAME, key).ok()?;

    match entry.get_password() {
        Ok(password) => {
            if password.is_empty() {
                None
            } else {
                Some(password)
            }
        }
        Err(keyring::Error::NoEntry) => None,
        Err(e) => {
            eprintln!("Warning: Failed to retrieve secret '{}' from keychain: {}", key, e);
            None
        }
    }
}

/// Delete a secret from the system keychain.
///
/// # Arguments
/// * `key` - The key name to delete
///
/// # Returns
/// * `Ok(())` on success or if key doesn't exist
/// * `Err(String)` with error message on failure
pub fn delete_secret(key: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, key)
        .map_err(|e| format!("Failed to create keychain entry for '{}': {}", key, e))?;

    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // Already deleted
        Err(e) => Err(format!("Failed to delete secret '{}' from keychain: {}", key, e)),
    }
}

/// Check if a secret exists in the keychain.
///
/// # Arguments
/// * `key` - The key name to check
///
/// # Returns
/// * `true` if the secret exists and is non-empty
/// * `false` otherwise
pub fn has_secret(key: &str) -> bool {
    get_secret(key).is_some()
}

/// Migrate a secret from plaintext (e.g., from preferences file) to keychain.
/// After successful migration, returns the secret value.
///
/// # Arguments
/// * `key` - The key name
/// * `plaintext_value` - The value from the old plaintext storage
///
/// # Returns
/// * `Ok(Some(String))` with the secret if migration succeeded
/// * `Ok(None)` if there was no value to migrate
/// * `Err(String)` if migration failed
pub fn migrate_to_keychain(key: &str, plaintext_value: Option<String>) -> Result<Option<String>, String> {
    match plaintext_value {
        Some(value) if !value.is_empty() => {
            // Check if already in keychain
            if let Some(existing) = get_secret(key) {
                // Keychain already has a value, prefer it
                return Ok(Some(existing));
            }
            // Migrate to keychain
            store_secret(key, &value)?;
            Ok(Some(value))
        }
        _ => {
            // No plaintext value, check keychain
            Ok(get_secret(key))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Generate a unique test key to avoid conflicts between parallel tests
    fn unique_test_key(prefix: &str) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("test_murmur_{}_{}", prefix, timestamp)
    }

    // Helper to check if keychain is accessible (may fail in CI or sandboxed environments)
    fn keychain_accessible() -> bool {
        let test_key = unique_test_key("access_check");
        let result = store_secret(&test_key, "test");
        let _ = delete_secret(&test_key);
        result.is_ok()
    }

    #[test]
    fn test_store_and_get_secret() {
        if !keychain_accessible() {
            eprintln!("Skipping test: keychain not accessible");
            return;
        }

        let test_key = unique_test_key("store_get");
        let test_value = "test_secret_value_12345";

        // Store
        let result = store_secret(&test_key, test_value);
        assert!(result.is_ok(), "Failed to store: {:?}", result);

        // Retrieve
        let retrieved = get_secret(&test_key);
        assert_eq!(retrieved, Some(test_value.to_string()));

        // Cleanup
        let _ = delete_secret(&test_key);
    }

    #[test]
    fn test_delete_secret() {
        if !keychain_accessible() {
            eprintln!("Skipping test: keychain not accessible");
            return;
        }

        let test_key = unique_test_key("delete");
        let test_value = "to_be_deleted";

        // Store first
        let _ = store_secret(&test_key, test_value);

        // Delete
        let result = delete_secret(&test_key);
        assert!(result.is_ok());

        // Verify deleted
        let retrieved = get_secret(&test_key);
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_get_nonexistent_secret() {
        let result = get_secret("nonexistent_key_that_should_not_exist_xyz123");
        assert!(result.is_none());
    }

    #[test]
    fn test_delete_nonexistent_secret() {
        // Should not error when deleting non-existent key
        let result = delete_secret("another_nonexistent_key_abc456");
        assert!(result.is_ok());
    }

    #[test]
    fn test_store_empty_deletes() {
        if !keychain_accessible() {
            eprintln!("Skipping test: keychain not accessible");
            return;
        }

        let test_key = unique_test_key("empty_store");
        let _ = store_secret(&test_key, "initial_value");

        // Storing empty should delete
        let result = store_secret(&test_key, "");
        assert!(result.is_ok());

        // Should be gone
        assert!(get_secret(&test_key).is_none());
    }

    #[test]
    fn test_has_secret() {
        if !keychain_accessible() {
            eprintln!("Skipping test: keychain not accessible");
            return;
        }

        let test_key = unique_test_key("has_secret");

        // Initially should not exist
        assert!(!has_secret(&test_key));

        // After storing
        let store_result = store_secret(&test_key, "some_value");
        assert!(store_result.is_ok(), "Failed to store: {:?}", store_result);
        assert!(has_secret(&test_key));

        // Cleanup
        let _ = delete_secret(&test_key);
        assert!(!has_secret(&test_key));
    }

    #[test]
    fn test_migrate_to_keychain_with_value() {
        if !keychain_accessible() {
            eprintln!("Skipping test: keychain not accessible");
            return;
        }

        let test_key = unique_test_key("migrate");
        let test_value = "migrated_value";

        // Migrate from plaintext
        let result = migrate_to_keychain(&test_key, Some(test_value.to_string()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(test_value.to_string()));

        // Verify it's in keychain
        assert_eq!(get_secret(&test_key), Some(test_value.to_string()));

        // Cleanup
        let _ = delete_secret(&test_key);
    }

    #[test]
    fn test_migrate_to_keychain_no_value() {
        let test_key = unique_test_key("migrate_none");

        // Migrate with no value
        let result = migrate_to_keychain(&test_key, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }
}
