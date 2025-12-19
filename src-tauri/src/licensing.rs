//! License validation using LemonSqueezy API.
//! Handles both subscription and one-time (lifetime) licenses.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const LEMONSQUEEZY_API_URL: &str = "https://api.lemonsqueezy.com/v1/licenses/validate";

/// License tier types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LicenseTier {
    /// Free tier - BYOK only, no license required
    Free,
    /// Subscription tier - cloud-based Whisper API
    Subscription,
    /// Lifetime tier - one-time purchase, local model
    Lifetime,
}

impl Default for LicenseTier {
    fn default() -> Self {
        LicenseTier::Free
    }
}

/// License information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseInfo {
    pub tier: LicenseTier,
    pub license_key: Option<String>,
    pub valid: bool,
    pub customer_email: Option<String>,
    pub expires_at: Option<String>,
    pub error: Option<String>,
}

impl Default for LicenseInfo {
    fn default() -> Self {
        LicenseInfo {
            tier: LicenseTier::Free,
            license_key: None,
            valid: false,
            customer_email: None,
            expires_at: None,
            error: None,
        }
    }
}

/// Cached license stored on disk
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CachedLicense {
    license_key: Option<String>,
    tier: LicenseTier,
    valid: bool,
    validated_at: Option<String>,
}

impl CachedLicense {
    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("murmur").join("license.json"))
    }

    fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(cached) = serde_json::from_str(&content) {
                        return cached;
                    }
                }
            }
        }
        CachedLicense::default()
    }

    fn save(&self) -> Result<(), String> {
        let path = Self::config_path().ok_or("Could not find config directory")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize license: {}", e))?;

        fs::write(&path, content).map_err(|e| format!("Failed to write license: {}", e))?;

        Ok(())
    }
}

/// LemonSqueezy API response
#[derive(Debug, Deserialize)]
struct LemonSqueezyResponse {
    valid: bool,
    error: Option<String>,
    license_key: Option<LemonSqueezyLicenseKey>,
}

#[derive(Debug, Deserialize)]
struct LemonSqueezyLicenseKey {
    status: String,
    #[serde(rename = "user_email")]
    user_email: Option<String>,
    expires_at: Option<String>,
}

/// Validate a license key with LemonSqueezy
pub async fn validate_license(license_key: &str) -> Result<LicenseInfo, String> {
    if license_key.is_empty() {
        return Ok(LicenseInfo::default());
    }

    let client = Client::new();

    let response = client
        .post(LEMONSQUEEZY_API_URL)
        .header("Accept", "application/json")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[("license_key", license_key)])
        .send()
        .await
        .map_err(|e| format!("Failed to validate license: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("License validation failed ({}): {}", status, error_text));
    }

    let ls_response: LemonSqueezyResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse license response: {}", e))?;

    if !ls_response.valid {
        return Ok(LicenseInfo {
            tier: LicenseTier::Free,
            license_key: Some(license_key.to_string()),
            valid: false,
            error: ls_response.error,
            ..Default::default()
        });
    }

    // Determine tier based on license status
    // In production, you'd check the product_id or variant_id to distinguish
    // For now, we'll assume all valid licenses are lifetime (one-time purchase)
    let tier = if let Some(ref key_info) = ls_response.license_key {
        if key_info.expires_at.is_some() {
            LicenseTier::Subscription
        } else {
            LicenseTier::Lifetime
        }
    } else {
        LicenseTier::Lifetime
    };

    let info = LicenseInfo {
        tier: tier.clone(),
        license_key: Some(license_key.to_string()),
        valid: true,
        customer_email: ls_response
            .license_key
            .as_ref()
            .and_then(|k| k.user_email.clone()),
        expires_at: ls_response
            .license_key
            .as_ref()
            .and_then(|k| k.expires_at.clone()),
        error: None,
    };

    // Cache the validated license
    let cached = CachedLicense {
        license_key: Some(license_key.to_string()),
        tier,
        valid: true,
        validated_at: Some(chrono::Utc::now().to_rfc3339()),
    };
    let _ = cached.save();

    Ok(info)
}

/// Get cached license info (for offline use)
pub fn get_cached_license() -> LicenseInfo {
    let cached = CachedLicense::load();

    LicenseInfo {
        tier: cached.tier,
        license_key: cached.license_key,
        valid: cached.valid,
        ..Default::default()
    }
}

/// Check if user has an active subscription
pub fn has_subscription() -> bool {
    let cached = CachedLicense::load();
    cached.valid && cached.tier == LicenseTier::Subscription
}

/// Check if user has a lifetime license
pub fn has_lifetime_license() -> bool {
    let cached = CachedLicense::load();
    cached.valid && cached.tier == LicenseTier::Lifetime
}

/// Clear cached license (for logout/deactivation)
pub fn clear_license() -> Result<(), String> {
    let cached = CachedLicense::default();
    cached.save()
}

/// Activate a license key
pub async fn activate_license(license_key: &str) -> Result<LicenseInfo, String> {
    let info = validate_license(license_key).await?;

    if info.valid {
        println!("License activated: {:?}", info.tier);
    } else {
        println!(
            "License activation failed: {}",
            info.error.as_deref().unwrap_or("Unknown error")
        );
    }

    Ok(info)
}
