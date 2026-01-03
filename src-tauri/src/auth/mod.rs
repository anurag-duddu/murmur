//! Authentication module for Keyhold.
//!
//! Provides WorkOS OAuth authentication with PKCE security and secure token
//! storage using macOS Keychain.

pub mod keychain;
pub mod pkce;
pub mod types;
pub mod workos;

use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_opener::OpenerExt;

pub use types::{AuthError, AuthState, AuthTokens, UserInfo};

use self::workos::WorkOsClient;

/// Global PKCE state stored during OAuth flow.
/// This is consumed when the callback is received.
static PKCE_STATE: Mutex<Option<types::PkceChallenge>> = Mutex::new(None);

/// Check if the user is currently authenticated.
///
/// Returns `true` if a refresh token exists (access token can be refreshed).
/// We check for refresh_token presence rather than access_token expiry because
/// access tokens are short-lived (5 min) but refresh tokens last much longer.
pub fn is_authenticated() -> bool {
    match keychain::get_tokens() {
        Ok(Some(tokens)) => !tokens.refresh_token.is_empty(),
        _ => false,
    }
}

/// Get the current authentication state for the frontend.
pub fn get_auth_state() -> AuthState {
    let tokens = keychain::get_tokens().ok().flatten();
    let user = keychain::get_user_info().ok().flatten();

    // Check for refresh token presence rather than access token expiry
    let is_authenticated = tokens.map(|t| !t.refresh_token.is_empty()).unwrap_or(false);

    AuthState {
        is_authenticated,
        user,
        is_loading: false,
    }
}

/// Start the OAuth authentication flow.
///
/// Generates PKCE challenge, stores it, and opens the browser to WorkOS.
pub fn start_auth_flow(app: &AppHandle) -> Result<(), AuthError> {
    log::info!("Starting OAuth authentication flow");

    // Generate PKCE challenge
    let pkce = pkce::generate_pkce();

    // Store PKCE state for callback verification
    {
        let mut state = PKCE_STATE.lock().unwrap();
        *state = Some(pkce.clone());
    }

    // Create WorkOS client and get authorization URL
    let client = WorkOsClient::new().map_err(|e| {
        log::error!("Failed to create WorkOS client: {}", e);
        e
    })?;
    let auth_url = client.get_authorization_url(&pkce);

    log::info!("Opening authorization URL in browser: {}", auth_url);

    // Open the authorization URL in the default browser
    match app.opener().open_url(&auth_url, None::<&str>) {
        Ok(()) => {
            log::info!("Browser opened successfully");
        }
        Err(e) => {
            log::error!("Failed to open browser: {}", e);
            return Err(AuthError::WorkOsError(format!(
                "Failed to open browser: {}",
                e
            )));
        }
    }

    Ok(())
}

/// Handle the OAuth callback URL.
///
/// Called when the app receives a `keyhold://auth/callback` deep link.
pub async fn handle_callback(app: &AppHandle, url: &str) -> Result<(), AuthError> {
    log::info!("Handling OAuth callback");

    // Parse the callback URL
    let callback = workos::parse_callback_url(url)?;

    // Get and clear the stored PKCE state
    let pkce = {
        let mut state = PKCE_STATE.lock().unwrap();
        state.take()
    };

    let pkce = pkce.ok_or_else(|| {
        log::error!("No PKCE state found - callback received without prior auth request");
        AuthError::StateMismatch
    })?;

    // Verify state matches (CSRF protection)
    if callback.state != pkce.state {
        log::error!(
            "State mismatch: expected {}, got {}",
            pkce.state,
            callback.state
        );
        return Err(AuthError::StateMismatch);
    }

    // Exchange the code for tokens
    let client = WorkOsClient::new()?;
    let (tokens, user_info) = client.exchange_code(&callback.code, &pkce.verifier).await?;

    // Store tokens and user info in keychain
    keychain::store_tokens(&tokens)?;
    keychain::store_user_info(&user_info)?;

    log::info!("Authentication successful for user: {}", user_info.email);

    // Emit auth state change event
    let auth_state = AuthState {
        is_authenticated: true,
        user: Some(user_info),
        is_loading: false,
    };

    let _ = app.emit("auth-state-changed", &auth_state);

    // Close login window and show appropriate next window
    if let Some(login_window) = app.get_webview_window("login") {
        let _ = login_window.close();
    }

    // Check if onboarding is needed
    let needs_onboarding = !crate::permissions::is_onboarding_complete();
    let has_accessibility = crate::permissions::check_accessibility_permission();

    if needs_onboarding || !has_accessibility {
        // Show onboarding window
        if let Some(onboarding) = app.get_webview_window("onboarding") {
            let _ = onboarding.show();
            let _ = onboarding.set_focus();
            // Notify onboarding window to start permission checks after a small delay
            let app_clone = app.clone();
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                let _ = app_clone.emit("start-onboarding", ());
            });
        }
    } else {
        // Just set the app to accessory mode (menu bar only)
        #[cfg(target_os = "macos")]
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    }

    Ok(())
}

/// Refresh the access token if it's expired or about to expire.
pub async fn refresh_if_needed() -> Result<(), AuthError> {
    let tokens = keychain::get_tokens()?.ok_or(AuthError::NotAuthenticated)?;

    if tokens.is_expired() {
        log::info!("Access token expired, refreshing...");

        let client = WorkOsClient::new()?;
        let new_tokens = client.refresh_token(&tokens.refresh_token).await?;
        keychain::store_tokens(&new_tokens)?;

        log::info!("Access token refreshed successfully");
    }

    Ok(())
}

/// Get the current access token, refreshing if necessary.
pub async fn get_access_token() -> Result<String, AuthError> {
    refresh_if_needed().await?;

    let tokens = keychain::get_tokens()?.ok_or(AuthError::NotAuthenticated)?;
    Ok(tokens.access_token)
}

/// Log out the current user.
///
/// Clears all stored authentication data from the keychain.
pub fn logout(app: &AppHandle) -> Result<(), AuthError> {
    log::info!("Logging out user");

    // Clear all auth data
    keychain::clear_all()?;

    // Emit auth state change
    let auth_state = AuthState {
        is_authenticated: false,
        user: None,
        is_loading: false,
    };
    let _ = app.emit("auth-state-changed", &auth_state);

    // Show login window
    if let Some(login_window) = app.get_webview_window("login") {
        let _ = login_window.show();
        let _ = login_window.set_focus();
        #[cfg(target_os = "macos")]
        app.set_activation_policy(tauri::ActivationPolicy::Regular);
    }

    // Hide other windows
    if let Some(main_window) = app.get_webview_window("main") {
        let _ = main_window.hide();
    }
    if let Some(onboarding) = app.get_webview_window("onboarding") {
        let _ = onboarding.hide();
    }

    Ok(())
}

/// Get the current user's information.
pub fn get_user_info() -> Result<Option<UserInfo>, AuthError> {
    keychain::get_user_info()
}
