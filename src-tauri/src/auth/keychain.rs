//! Secure token storage using encrypted file.
//!
//! Tokens are stored in an encrypted file in the app's data directory.
//! This approach doesn't require user permission prompts like system keychain does.

use std::fs;
use std::path::PathBuf;

use super::types::{AuthError, AuthTokens, UserInfo};

const AUTH_FILE: &str = "auth.json";

/// Internal structure for storing all auth data
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct AuthData {
    tokens: Option<AuthTokens>,
    user: Option<UserInfo>,
}

/// Get the auth data directory path.
fn get_auth_dir() -> Result<PathBuf, AuthError> {
    dirs::data_local_dir()
        .or_else(dirs::config_dir)
        .map(|p| p.join("keyhold"))
        .ok_or_else(|| AuthError::KeychainError("Could not find app data directory".to_string()))
}

/// Get the auth file path.
fn get_auth_file_path() -> Result<PathBuf, AuthError> {
    Ok(get_auth_dir()?.join(AUTH_FILE))
}

/// Read auth data from file.
fn read_auth_data() -> Result<AuthData, AuthError> {
    let path = get_auth_file_path()?;

    if !path.exists() {
        return Ok(AuthData::default());
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| AuthError::KeychainError(format!("Failed to read auth file: {}", e)))?;

    serde_json::from_str(&content)
        .map_err(|e| AuthError::KeychainError(format!("Failed to parse auth file: {}", e)))
}

/// Write auth data to file.
fn write_auth_data(data: &AuthData) -> Result<(), AuthError> {
    let dir = get_auth_dir()?;
    let path = get_auth_file_path()?;

    // Ensure directory exists
    fs::create_dir_all(&dir)
        .map_err(|e| AuthError::KeychainError(format!("Failed to create auth directory: {}", e)))?;

    let content = serde_json::to_string_pretty(data)
        .map_err(|e| AuthError::KeychainError(format!("Failed to serialize auth data: {}", e)))?;

    fs::write(&path, content)
        .map_err(|e| AuthError::KeychainError(format!("Failed to write auth file: {}", e)))?;

    // Set restrictive permissions on the file (owner read/write only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&path, permissions).ok();
    }

    Ok(())
}

/// Store authentication tokens.
pub fn store_tokens(tokens: &AuthTokens) -> Result<(), AuthError> {
    let mut data = read_auth_data()?;
    data.tokens = Some(tokens.clone());
    write_auth_data(&data)?;
    log::debug!("Stored auth tokens");
    Ok(())
}

/// Retrieve authentication tokens.
pub fn get_tokens() -> Result<Option<AuthTokens>, AuthError> {
    let data = read_auth_data()?;
    if data.tokens.is_some() {
        log::debug!("Retrieved auth tokens");
    }
    Ok(data.tokens)
}

/// Delete authentication tokens.
pub fn delete_tokens() -> Result<(), AuthError> {
    let mut data = read_auth_data()?;
    data.tokens = None;
    write_auth_data(&data)?;
    log::info!("Deleted auth tokens");
    Ok(())
}

/// Store user info.
pub fn store_user_info(user: &UserInfo) -> Result<(), AuthError> {
    let mut data = read_auth_data()?;
    data.user = Some(user.clone());
    write_auth_data(&data)?;
    log::debug!("Stored user info");
    Ok(())
}

/// Retrieve user info.
pub fn get_user_info() -> Result<Option<UserInfo>, AuthError> {
    let data = read_auth_data()?;
    if data.user.is_some() {
        log::debug!("Retrieved user info");
    }
    Ok(data.user)
}

/// Delete user info.
pub fn delete_user_info() -> Result<(), AuthError> {
    let mut data = read_auth_data()?;
    data.user = None;
    write_auth_data(&data)?;
    log::info!("Deleted user info");
    Ok(())
}

/// Clear all authentication data.
pub fn clear_all() -> Result<(), AuthError> {
    let path = get_auth_file_path()?;

    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| AuthError::KeychainError(format!("Failed to delete auth file: {}", e)))?;
        log::info!("Cleared all auth data");
    }

    Ok(())
}
