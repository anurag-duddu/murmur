//! Unified error types for the Murmur application.
//!
//! This module provides structured error types that can be serialized
//! for frontend consumption while maintaining useful debug information.

use serde::{Deserialize, Serialize};

/// Application error type for structured error handling.
///
/// All variants can be serialized to JSON for frontend consumption.
/// The `From<AppError> for String` implementation ensures Tauri compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AppError {
    /// No audio input device available
    NoAudioDevice,

    /// Recording produced no audio data
    NoAudioCaptured,

    /// Transcription service failed
    TranscriptionFailed {
        provider: String,
        message: String,
    },

    /// Text enhancement/transformation failed
    EnhancementFailed {
        message: String,
        /// Fallback text to use (usually raw transcript)
        fallback: Option<String>,
    },

    /// Configuration error
    ConfigError(String),

    /// Permission denied (microphone, accessibility, etc.)
    PermissionDenied(String),

    /// Network/API error
    NetworkError(String),

    /// License validation error
    LicenseError(String),

    /// Rate limit exceeded
    RateLimitExceeded {
        service: String,
        retry_after_ms: Option<u64>,
    },

    /// Model not available (for local Whisper)
    ModelNotAvailable(String),

    /// State error (invalid state transition)
    InvalidState(String),

    /// Input validation error
    ValidationError(String),

    /// Generic internal error
    InternalError(String),
}

impl AppError {
    /// Create a user-friendly error message.
    /// This sanitizes internal details for display to users.
    pub fn user_message(&self) -> String {
        match self {
            AppError::NoAudioDevice => {
                "No microphone found. Please connect a microphone and try again.".to_string()
            }
            AppError::NoAudioCaptured => {
                "No audio was captured. Please speak louder or check your microphone.".to_string()
            }
            AppError::TranscriptionFailed { provider, .. } => {
                format!("Transcription failed ({}). Please try again.", provider)
            }
            AppError::EnhancementFailed { fallback, .. } => {
                if fallback.is_some() {
                    "Text enhancement failed. Using original transcription.".to_string()
                } else {
                    "Text enhancement failed. Please try again.".to_string()
                }
            }
            AppError::ConfigError(_) => {
                "Configuration error. Please check your settings.".to_string()
            }
            AppError::PermissionDenied(perm) => {
                format!("{} permission required. Please grant access in System Settings.", perm)
            }
            AppError::NetworkError(_) => {
                "Network error. Please check your internet connection.".to_string()
            }
            AppError::LicenseError(_) => {
                "License validation failed. Please check your license key.".to_string()
            }
            AppError::RateLimitExceeded { service, .. } => {
                format!("Too many requests to {}. Please wait a moment.", service)
            }
            AppError::ModelNotAvailable(_) => {
                "Whisper model not available. Please download the model first.".to_string()
            }
            AppError::InvalidState(_) => {
                "Invalid operation. Please try again.".to_string()
            }
            AppError::ValidationError(msg) => {
                format!("Invalid input: {}", msg)
            }
            AppError::InternalError(_) => {
                "An unexpected error occurred. Please try again.".to_string()
            }
        }
    }

    /// Get the error code for programmatic handling
    pub fn code(&self) -> &'static str {
        match self {
            AppError::NoAudioDevice => "NO_AUDIO_DEVICE",
            AppError::NoAudioCaptured => "NO_AUDIO_CAPTURED",
            AppError::TranscriptionFailed { .. } => "TRANSCRIPTION_FAILED",
            AppError::EnhancementFailed { .. } => "ENHANCEMENT_FAILED",
            AppError::ConfigError(_) => "CONFIG_ERROR",
            AppError::PermissionDenied(_) => "PERMISSION_DENIED",
            AppError::NetworkError(_) => "NETWORK_ERROR",
            AppError::LicenseError(_) => "LICENSE_ERROR",
            AppError::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
            AppError::ModelNotAvailable(_) => "MODEL_NOT_AVAILABLE",
            AppError::InvalidState(_) => "INVALID_STATE",
            AppError::ValidationError(_) => "VALIDATION_ERROR",
            AppError::InternalError(_) => "INTERNAL_ERROR",
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use the user-friendly message for Display
        write!(f, "{}", self.user_message())
    }
}

impl std::error::Error for AppError {}

/// Convert AppError to String for Tauri command compatibility.
/// Tauri commands typically return `Result<T, String>`.
impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        // Return user-friendly message for Tauri error responses
        error.user_message()
    }
}

/// Sanitize an error message by removing potentially sensitive information.
/// Use this for error messages that will be shown to users or logged.
#[allow(dead_code)]
pub fn sanitize_error_message(msg: &str) -> String {
    let mut sanitized = msg.to_string();

    // Remove file paths (Unix-style)
    let path_pattern = regex::Regex::new(r"/[\w./\-]+").ok();
    if let Some(re) = path_pattern {
        sanitized = re.replace_all(&sanitized, "[path]").to_string();
    }

    // Remove API keys (common patterns)
    let key_patterns = [
        r"sk-[a-zA-Z0-9]{20,}",           // OpenAI/Anthropic style
        r"dg_[a-zA-Z0-9]{20,}",           // Deepgram
        r"gsk_[a-zA-Z0-9]{20,}",          // Groq
        r"[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}", // UUIDs
    ];

    for pattern in &key_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            sanitized = re.replace_all(&sanitized, "[redacted]").to_string();
        }
    }

    // Remove IP addresses
    if let Ok(re) = regex::Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b") {
        sanitized = re.replace_all(&sanitized, "[ip]").to_string();
    }

    // Remove email addresses
    if let Ok(re) = regex::Regex::new(r"\b[\w.-]+@[\w.-]+\.\w+\b") {
        sanitized = re.replace_all(&sanitized, "[email]").to_string();
    }

    sanitized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_codes() {
        assert_eq!(AppError::NoAudioDevice.code(), "NO_AUDIO_DEVICE");
        assert_eq!(AppError::NoAudioCaptured.code(), "NO_AUDIO_CAPTURED");
        assert_eq!(
            AppError::TranscriptionFailed {
                provider: "test".to_string(),
                message: "error".to_string()
            }
            .code(),
            "TRANSCRIPTION_FAILED"
        );
    }

    #[test]
    fn test_app_error_user_messages() {
        let error = AppError::NoAudioDevice;
        assert!(error.user_message().contains("microphone"));

        let error = AppError::PermissionDenied("Microphone".to_string());
        assert!(error.user_message().contains("Microphone"));
        assert!(error.user_message().contains("permission"));
    }

    #[test]
    fn test_app_error_to_string() {
        let error = AppError::NetworkError("connection failed".to_string());
        let s: String = error.into();
        assert!(s.contains("Network error"));
    }

    #[test]
    fn test_app_error_serialization() {
        let error = AppError::TranscriptionFailed {
            provider: "deepgram".to_string(),
            message: "API error".to_string(),
        };
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("transcription_failed"));
        assert!(json.contains("deepgram"));
    }

    #[test]
    fn test_sanitize_error_message_paths() {
        let msg = "Failed to read /Users/john/secret/config.json";
        let sanitized = sanitize_error_message(msg);
        assert!(!sanitized.contains("/Users"));
        assert!(sanitized.contains("[path]"));
    }

    #[test]
    fn test_sanitize_error_message_api_keys() {
        let msg = "API key sk-abc123def456ghi789jkl012 is invalid";
        let sanitized = sanitize_error_message(msg);
        assert!(!sanitized.contains("sk-abc"));
        assert!(sanitized.contains("[redacted]"));
    }

    #[test]
    fn test_sanitize_error_message_ip() {
        let msg = "Connection refused to 192.168.1.100:8080";
        let sanitized = sanitize_error_message(msg);
        assert!(!sanitized.contains("192.168"));
        assert!(sanitized.contains("[ip]"));
    }

    #[test]
    fn test_sanitize_error_message_email() {
        let msg = "Failed to authenticate user@example.com";
        let sanitized = sanitize_error_message(msg);
        assert!(!sanitized.contains("user@example"));
        assert!(sanitized.contains("[email]"));
    }

    #[test]
    fn test_sanitize_error_message_preserves_normal_text() {
        let msg = "Connection timeout after 30 seconds";
        let sanitized = sanitize_error_message(msg);
        assert_eq!(sanitized, msg);
    }
}
