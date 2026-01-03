//! WorkOS API client for OAuth authentication.
//!
//! Handles communication with WorkOS AuthKit for user authentication,
//! including authorization URL generation, token exchange, and token refresh.

use crate::http_client::get_client;

use super::types::{
    AuthError, AuthTokens, OAuthCallback, PkceChallenge, UserInfo, WorkOsAuthResponse,
};

const WORKOS_API_BASE: &str = "https://api.workos.com";

/// WorkOS OAuth client
pub struct WorkOsClient {
    client_id: String,
    redirect_uri: String,
}

impl WorkOsClient {
    /// Create a new WorkOS client from environment variables.
    pub fn new() -> Result<Self, AuthError> {
        let client_id = std::env::var("WORKOS_CLIENT_ID").map_err(|_| {
            AuthError::WorkOsError("WORKOS_CLIENT_ID environment variable not set".to_string())
        })?;

        Ok(Self {
            client_id,
            redirect_uri: "keyhold://auth/callback".to_string(),
        })
    }

    /// Generate the authorization URL for WorkOS AuthKit.
    ///
    /// This URL should be opened in the user's browser to initiate the OAuth flow.
    pub fn get_authorization_url(&self, pkce: &PkceChallenge) -> String {
        format!(
            "{}/user_management/authorize?client_id={}&redirect_uri={}&response_type=code&code_challenge={}&code_challenge_method=S256&state={}&provider=authkit",
            WORKOS_API_BASE,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(&pkce.challenge),
            urlencoding::encode(&pkce.state)
        )
    }

    /// Exchange an authorization code for access and refresh tokens.
    ///
    /// This is called after the user completes authentication and is redirected
    /// back to the app with an authorization code.
    pub async fn exchange_code(
        &self,
        code: &str,
        verifier: &str,
    ) -> Result<(AuthTokens, UserInfo), AuthError> {
        let client = get_client().map_err(|e| AuthError::NetworkError(e.to_string()))?;

        let api_key = std::env::var("WORKOS_API_KEY").map_err(|_| {
            AuthError::WorkOsError("WORKOS_API_KEY environment variable not set".to_string())
        })?;

        log::info!("Exchanging authorization code for tokens");

        let response = client
            .post(format!("{}/user_management/authenticate", WORKOS_API_BASE))
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&serde_json::json!({
                "client_id": self.client_id,
                "code": code,
                "code_verifier": verifier,
                "grant_type": "authorization_code"
            }))
            .send()
            .await
            .map_err(|e| {
                AuthError::NetworkError(format!("Token exchange request failed: {}", e))
            })?;

        let status = response.status();
        let response_text = response.text().await.unwrap_or_default();

        if !status.is_success() {
            log::error!("Token exchange failed: {} - {}", status, response_text);
            return Err(AuthError::WorkOsError(format!(
                "Token exchange failed: {} - {}",
                status, response_text
            )));
        }

        log::debug!("Token exchange response: {}", response_text);

        let auth_response: WorkOsAuthResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                log::error!("Failed to parse response: {} - Body: {}", e, response_text);
                AuthError::WorkOsError(format!("Failed to parse authentication response: {}", e))
            })?;

        // Calculate expiry timestamp
        // If expires_in is not provided, default to 5 minutes (300 seconds)
        // The access token is a JWT with its own expiry, but we use this for local cache refresh
        let expires_in = auth_response.expires_in.unwrap_or(300);
        let expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            + expires_in;

        let tokens = AuthTokens {
            access_token: auth_response.access_token,
            refresh_token: auth_response.refresh_token,
            expires_at,
        };

        let user_info = UserInfo::from(auth_response.user);

        log::info!("Successfully exchanged code for tokens");
        Ok((tokens, user_info))
    }

    /// Refresh an expired access token using a refresh token.
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthTokens, AuthError> {
        let client = get_client().map_err(|e| AuthError::NetworkError(e.to_string()))?;

        let api_key = std::env::var("WORKOS_API_KEY").map_err(|_| {
            AuthError::WorkOsError("WORKOS_API_KEY environment variable not set".to_string())
        })?;

        log::info!("Refreshing access token");

        let response = client
            .post(format!("{}/user_management/authenticate", WORKOS_API_BASE))
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&serde_json::json!({
                "client_id": self.client_id,
                "refresh_token": refresh_token,
                "grant_type": "refresh_token"
            }))
            .send()
            .await
            .map_err(|e| AuthError::NetworkError(format!("Token refresh request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            log::error!("Token refresh failed: {} - {}", status, error_text);
            return Err(AuthError::TokenExpired);
        }

        #[derive(serde::Deserialize)]
        struct RefreshResponse {
            access_token: String,
            refresh_token: String,
            expires_in: u64,
        }

        let refresh_response: RefreshResponse = response.json().await.map_err(|e| {
            AuthError::WorkOsError(format!("Failed to parse refresh response: {}", e))
        })?;

        let expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            + refresh_response.expires_in;

        log::info!("Successfully refreshed access token");

        Ok(AuthTokens {
            access_token: refresh_response.access_token,
            refresh_token: refresh_response.refresh_token,
            expires_at,
        })
    }
}

/// Parse an OAuth callback URL into its components.
///
/// Expected format: `keyhold://auth/callback?code=xxx&state=yyy`
pub fn parse_callback_url(url: &str) -> Result<OAuthCallback, AuthError> {
    let parsed = url::Url::parse(url).map_err(|e| {
        log::error!("Failed to parse callback URL: {} - {}", url, e);
        AuthError::InvalidCallback
    })?;

    // Extract query parameters
    let mut code = None;
    let mut state = None;

    for (key, value) in parsed.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.to_string()),
            "state" => state = Some(value.to_string()),
            _ => {}
        }
    }

    let code = code.ok_or_else(|| {
        log::error!("Missing 'code' parameter in callback URL");
        AuthError::InvalidCallback
    })?;

    let state = state.ok_or_else(|| {
        log::error!("Missing 'state' parameter in callback URL");
        AuthError::InvalidCallback
    })?;

    Ok(OAuthCallback { code, state })
}
