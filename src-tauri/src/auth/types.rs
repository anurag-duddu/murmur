//! Authentication type definitions for WorkOS OAuth flow.

use serde::{Deserialize, Serialize};

/// Stored authentication tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    /// Unix timestamp when access token expires
    pub expires_at: u64,
}

impl AuthTokens {
    /// Check if the access token has expired (with 5 minute buffer)
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        // Consider expired 5 minutes before actual expiry
        now >= self.expires_at.saturating_sub(300)
    }
}

/// User information from WorkOS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub profile_picture_url: Option<String>,
}

/// Authentication state exposed to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthState {
    pub is_authenticated: bool,
    pub user: Option<UserInfo>,
    pub is_loading: bool,
}

impl Default for AuthState {
    fn default() -> Self {
        Self {
            is_authenticated: false,
            user: None,
            is_loading: true,
        }
    }
}

/// PKCE challenge data stored during OAuth flow
#[derive(Debug, Clone)]
pub struct PkceChallenge {
    pub verifier: String,
    pub challenge: String,
    pub state: String,
}

/// WorkOS authentication response
#[derive(Debug, Deserialize)]
pub struct WorkOsAuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    /// Token lifetime in seconds (optional - WorkOS may not always include this)
    #[serde(default)]
    pub expires_in: Option<u64>,
    pub user: WorkOsUser,
    /// Authentication method used (e.g., "GoogleOAuth", "Password")
    #[serde(default)]
    pub authentication_method: Option<String>,
}

/// User object from WorkOS API
#[derive(Debug, Deserialize)]
pub struct WorkOsUser {
    pub id: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub profile_picture_url: Option<String>,
}

impl From<WorkOsUser> for UserInfo {
    fn from(user: WorkOsUser) -> Self {
        Self {
            id: user.id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            profile_picture_url: user.profile_picture_url,
        }
    }
}

/// Parsed OAuth callback parameters
#[derive(Debug)]
pub struct OAuthCallback {
    pub code: String,
    pub state: String,
}

/// Authentication errors
#[derive(Debug, Clone, Serialize)]
pub enum AuthError {
    NotAuthenticated,
    TokenExpired,
    InvalidCallback,
    StateMismatch,
    NetworkError(String),
    StorageError(String),
    WorkOsError(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::NotAuthenticated => write!(f, "Not authenticated"),
            AuthError::TokenExpired => write!(f, "Session expired"),
            AuthError::InvalidCallback => write!(f, "Invalid authentication callback"),
            AuthError::StateMismatch => write!(f, "Authentication state mismatch"),
            AuthError::NetworkError(e) => write!(f, "Network error: {}", e),
            AuthError::StorageError(e) => write!(f, "Storage error: {}", e),
            AuthError::WorkOsError(e) => write!(f, "Authentication error: {}", e),
        }
    }
}

impl std::error::Error for AuthError {}
