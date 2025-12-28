use crate::secure_storage::{self, keys as secret_keys};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

/// Transcription provider options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TranscriptionProvider {
    /// Deepgram API - BYOK (Bring Your Own Key)
    Deepgram,
    /// Whisper via Replicate API - for subscribers
    WhisperApi,
    /// Local Whisper model - for one-time purchase users
    WhisperLocal,
}

impl Default for TranscriptionProvider {
    fn default() -> Self {
        TranscriptionProvider::Deepgram
    }
}

impl TranscriptionProvider {
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "whisperapi" | "whisper_api" | "whisper-api" => TranscriptionProvider::WhisperApi,
            "whisperlocal" | "whisper_local" | "whisper-local" => TranscriptionProvider::WhisperLocal,
            _ => TranscriptionProvider::Deepgram,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            TranscriptionProvider::Deepgram => "deepgram".to_string(),
            TranscriptionProvider::WhisperApi => "whisperapi".to_string(),
            TranscriptionProvider::WhisperLocal => "whisperlocal".to_string(),
        }
    }
}

/// Stored preferences that persist to disk.
/// NOTE: Sensitive fields (API keys, license key) are stored in the system keychain,
/// not in this JSON file. The fields here are kept for migration purposes only.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StoredPreferences {
    // DEPRECATED: These are now stored in keychain. Kept for migration from old versions.
    #[serde(skip_serializing, default)]
    pub deepgram_api_key: Option<String>,
    #[serde(skip_serializing, default)]
    pub groq_api_key: Option<String>,
    #[serde(skip_serializing, default)]
    pub anthropic_api_key: Option<String>,
    #[serde(skip_serializing, default)]
    pub license_key: Option<String>,

    // Non-sensitive preferences (stored in JSON file)
    pub recording_mode: Option<String>,
    pub hotkey: Option<String>,
    pub show_indicator: Option<bool>,
    pub play_sounds: Option<bool>,
    pub microphone: Option<String>,
    pub language: Option<String>,
    pub transcription_provider: Option<String>,
    pub onboarding_complete: Option<bool>,
    pub spoken_languages: Option<Vec<String>>,
}

impl StoredPreferences {
    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("murmur").join("preferences.json"))
    }

    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(prefs) = serde_json::from_str(&content) {
                        println!("Loaded preferences from {:?}", path);
                        return prefs;
                    }
                }
            }
        }
        StoredPreferences::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path().ok_or("Could not find config directory")?;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize preferences: {}", e))?;

        fs::write(&path, content)
            .map_err(|e| format!("Failed to write preferences: {}", e))?;

        println!("Saved preferences to {:?}", path);
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub deepgram_api_key: Option<String>,
    pub groq_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub recording_mode: String,
    pub hotkey: String,
    pub max_recording_duration: u32,
    pub audio_sample_rate: u32,
    pub audio_channels: u16,
    pub claude_model: String,
    pub deepgram_model: String,
    pub deepgram_language: String,
    pub show_indicator: bool,
    pub play_sounds: bool,
    pub microphone: String,
    pub language: String,
    // New: Transcription provider settings
    pub transcription_provider: TranscriptionProvider,
    pub license_key: Option<String>,
}

impl AppConfig {
    pub fn load() -> Self {
        // Load .env file if it exists (for development)
        dotenv().ok();

        // Load stored preferences from file (may contain old plaintext keys for migration)
        let stored = StoredPreferences::load();

        // Load secrets from keychain, migrating from plaintext if necessary
        // Priority: keychain > plaintext file (migrate) > env vars
        let deepgram_api_key = secure_storage::migrate_to_keychain(
            secret_keys::DEEPGRAM_API_KEY,
            stored.deepgram_api_key.clone(),
        )
        .ok()
        .flatten()
        .or_else(|| env::var("DEEPGRAM_API_KEY").ok().filter(|s| !s.is_empty()));

        let groq_api_key = secure_storage::migrate_to_keychain(
            secret_keys::GROQ_API_KEY,
            stored.groq_api_key.clone(),
        )
        .ok()
        .flatten()
        .or_else(|| env::var("GROQ_API_KEY").ok().filter(|s| !s.is_empty()));

        let anthropic_api_key = secure_storage::migrate_to_keychain(
            secret_keys::ANTHROPIC_API_KEY,
            stored.anthropic_api_key.clone(),
        )
        .ok()
        .flatten()
        .or_else(|| env::var("ANTHROPIC_API_KEY").ok().filter(|s| !s.is_empty()));

        let license_key = secure_storage::migrate_to_keychain(
            secret_keys::LICENSE_KEY,
            stored.license_key.clone(),
        )
        .ok()
        .flatten()
        .or_else(|| env::var("LICENSE_KEY").ok().filter(|s| !s.is_empty()));

        // If we migrated any keys, re-save preferences to remove plaintext keys from file
        if stored.deepgram_api_key.is_some()
            || stored.groq_api_key.is_some()
            || stored.anthropic_api_key.is_some()
            || stored.license_key.is_some()
        {
            // Re-save to remove plaintext keys (they're now skip_serializing)
            let _ = stored.save();
        }

        AppConfig {
            deepgram_api_key,
            groq_api_key,
            anthropic_api_key,
            recording_mode: stored
                .recording_mode
                .unwrap_or_else(|| {
                    env::var("DEFAULT_RECORDING_MODE").unwrap_or_else(|_| "push-to-talk".to_string())
                }),
            hotkey: stored
                .hotkey
                .unwrap_or_else(|| {
                    env::var("DEFAULT_HOTKEY").unwrap_or_else(|_| "Option+Space".to_string())
                }),
            max_recording_duration: env::var("MAX_RECORDING_DURATION")
                .unwrap_or_else(|_| "1800".to_string())
                .parse()
                .unwrap_or(1800),
            audio_sample_rate: env::var("AUDIO_SAMPLE_RATE")
                .unwrap_or_else(|_| "16000".to_string())
                .parse()
                .unwrap_or(16000),
            audio_channels: env::var("AUDIO_CHANNELS")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .unwrap_or(1),
            claude_model: env::var("CLAUDE_MODEL")
                .unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string()),
            deepgram_model: env::var("DEEPGRAM_MODEL")
                .unwrap_or_else(|_| "nova-2".to_string()),
            deepgram_language: env::var("DEEPGRAM_LANGUAGE")
                .unwrap_or_else(|_| "en-US".to_string()),
            show_indicator: stored.show_indicator.unwrap_or(true),
            play_sounds: stored.play_sounds.unwrap_or(true),
            microphone: stored.microphone.unwrap_or_else(|| "default".to_string()),
            language: stored.language.unwrap_or_else(|| "en-US".to_string()),
            transcription_provider: stored
                .transcription_provider
                .map(|s| TranscriptionProvider::from_string(&s))
                .unwrap_or_else(TranscriptionProvider::default),
            license_key,
        }
    }

    pub fn update_from_preferences(&mut self, prefs: Preferences) -> Result<(), String> {
        // Update in-memory config and store secrets in keychain
        if !prefs.deepgram_key.is_empty() {
            secure_storage::store_secret(secret_keys::DEEPGRAM_API_KEY, &prefs.deepgram_key)?;
            self.deepgram_api_key = Some(prefs.deepgram_key.clone());
        }
        if !prefs.anthropic_key.is_empty() {
            secure_storage::store_secret(secret_keys::ANTHROPIC_API_KEY, &prefs.anthropic_key)?;
            self.anthropic_api_key = Some(prefs.anthropic_key.clone());
        }
        self.recording_mode = prefs.recording_mode.clone();
        self.hotkey = prefs.hotkey.clone();
        self.show_indicator = prefs.show_indicator;
        self.play_sounds = prefs.play_sounds;
        self.microphone = prefs.microphone.clone();
        self.language = prefs.language.clone();

        // Update transcription provider
        if let Some(provider) = &prefs.transcription_provider {
            self.transcription_provider = TranscriptionProvider::from_string(provider);
        }

        // Store license key in keychain
        if let Some(license) = &prefs.license_key {
            if !license.is_empty() {
                secure_storage::store_secret(secret_keys::LICENSE_KEY, license)?;
                self.license_key = Some(license.clone());
            }
        }

        // Persist non-sensitive preferences to disk
        // Note: API keys are NOT stored here - they go to keychain
        let stored = StoredPreferences {
            // These fields are skip_serializing, so they won't be written to file
            deepgram_api_key: None,
            groq_api_key: None,
            anthropic_api_key: None,
            license_key: None,
            // Non-sensitive preferences
            recording_mode: Some(prefs.recording_mode),
            hotkey: Some(prefs.hotkey),
            show_indicator: Some(prefs.show_indicator),
            play_sounds: Some(prefs.play_sounds),
            microphone: Some(prefs.microphone),
            language: Some(prefs.language),
            transcription_provider: Some(self.transcription_provider.to_string()),
            onboarding_complete: prefs.onboarding_complete,
            spoken_languages: prefs.spoken_languages,
        };
        stored.save()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Preferences {
    pub recording_mode: String,
    pub hotkey: String,
    pub show_indicator: bool,
    pub play_sounds: bool,
    pub microphone: String,
    pub language: String,
    #[serde(rename = "deepgram_api_key")]
    pub deepgram_key: String,
    #[serde(rename = "anthropic_api_key")]
    pub anthropic_key: String,
    // New: Transcription provider settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcription_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_key: Option<String>,
    // Onboarding settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onboarding_complete: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spoken_languages: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== TranscriptionProvider Tests ====================

    #[test]
    fn test_transcription_provider_default() {
        let provider = TranscriptionProvider::default();
        assert_eq!(provider, TranscriptionProvider::Deepgram);
    }

    #[test]
    fn test_transcription_provider_from_string_deepgram() {
        assert_eq!(TranscriptionProvider::from_string("deepgram"), TranscriptionProvider::Deepgram);
        assert_eq!(TranscriptionProvider::from_string("DEEPGRAM"), TranscriptionProvider::Deepgram);
        assert_eq!(TranscriptionProvider::from_string("Deepgram"), TranscriptionProvider::Deepgram);
    }

    #[test]
    fn test_transcription_provider_from_string_whisper_api() {
        assert_eq!(TranscriptionProvider::from_string("whisperapi"), TranscriptionProvider::WhisperApi);
        assert_eq!(TranscriptionProvider::from_string("whisper_api"), TranscriptionProvider::WhisperApi);
        assert_eq!(TranscriptionProvider::from_string("whisper-api"), TranscriptionProvider::WhisperApi);
        assert_eq!(TranscriptionProvider::from_string("WHISPERAPI"), TranscriptionProvider::WhisperApi);
    }

    #[test]
    fn test_transcription_provider_from_string_whisper_local() {
        assert_eq!(TranscriptionProvider::from_string("whisperlocal"), TranscriptionProvider::WhisperLocal);
        assert_eq!(TranscriptionProvider::from_string("whisper_local"), TranscriptionProvider::WhisperLocal);
        assert_eq!(TranscriptionProvider::from_string("whisper-local"), TranscriptionProvider::WhisperLocal);
        assert_eq!(TranscriptionProvider::from_string("WHISPERLOCAL"), TranscriptionProvider::WhisperLocal);
    }

    #[test]
    fn test_transcription_provider_from_string_unknown_defaults_to_deepgram() {
        assert_eq!(TranscriptionProvider::from_string("unknown"), TranscriptionProvider::Deepgram);
        assert_eq!(TranscriptionProvider::from_string(""), TranscriptionProvider::Deepgram);
        assert_eq!(TranscriptionProvider::from_string("openai"), TranscriptionProvider::Deepgram);
    }

    #[test]
    fn test_transcription_provider_to_string() {
        assert_eq!(TranscriptionProvider::Deepgram.to_string(), "deepgram");
        assert_eq!(TranscriptionProvider::WhisperApi.to_string(), "whisperapi");
        assert_eq!(TranscriptionProvider::WhisperLocal.to_string(), "whisperlocal");
    }

    #[test]
    fn test_transcription_provider_serialization() {
        assert_eq!(serde_json::to_string(&TranscriptionProvider::Deepgram).unwrap(), "\"deepgram\"");
        assert_eq!(serde_json::to_string(&TranscriptionProvider::WhisperApi).unwrap(), "\"whisperapi\"");
        assert_eq!(serde_json::to_string(&TranscriptionProvider::WhisperLocal).unwrap(), "\"whisperlocal\"");
    }

    #[test]
    fn test_transcription_provider_deserialization() {
        let deepgram: TranscriptionProvider = serde_json::from_str("\"deepgram\"").unwrap();
        let whisper_api: TranscriptionProvider = serde_json::from_str("\"whisperapi\"").unwrap();
        let whisper_local: TranscriptionProvider = serde_json::from_str("\"whisperlocal\"").unwrap();

        assert_eq!(deepgram, TranscriptionProvider::Deepgram);
        assert_eq!(whisper_api, TranscriptionProvider::WhisperApi);
        assert_eq!(whisper_local, TranscriptionProvider::WhisperLocal);
    }

    // ==================== StoredPreferences Tests ====================

    #[test]
    fn test_stored_preferences_default() {
        let prefs = StoredPreferences::default();
        assert!(prefs.deepgram_api_key.is_none());
        assert!(prefs.groq_api_key.is_none());
        assert!(prefs.anthropic_api_key.is_none());
        assert!(prefs.recording_mode.is_none());
        assert!(prefs.hotkey.is_none());
        assert!(prefs.show_indicator.is_none());
        assert!(prefs.play_sounds.is_none());
        assert!(prefs.microphone.is_none());
        assert!(prefs.language.is_none());
        assert!(prefs.transcription_provider.is_none());
        assert!(prefs.license_key.is_none());
        assert!(prefs.onboarding_complete.is_none());
        assert!(prefs.spoken_languages.is_none());
    }

    #[test]
    fn test_stored_preferences_serialization() {
        let prefs = StoredPreferences {
            deepgram_api_key: Some("dg_key".to_string()),
            groq_api_key: Some("groq_key".to_string()),
            anthropic_api_key: Some("claude_key".to_string()),
            recording_mode: Some("push-to-talk".to_string()),
            hotkey: Some("Option+Space".to_string()),
            show_indicator: Some(true),
            play_sounds: Some(false),
            microphone: Some("default".to_string()),
            language: Some("en-US".to_string()),
            transcription_provider: Some("deepgram".to_string()),
            license_key: Some("license123".to_string()),
            onboarding_complete: Some(true),
            spoken_languages: Some(vec!["en".to_string(), "es".to_string()]),
        };

        let json = serde_json::to_string(&prefs).unwrap();
        // API keys should NOT be serialized (skip_serializing) - they go to keychain
        assert!(!json.contains("deepgram_api_key"), "API keys should not be serialized");
        assert!(!json.contains("groq_api_key"), "API keys should not be serialized");
        assert!(!json.contains("anthropic_api_key"), "API keys should not be serialized");
        assert!(!json.contains("license_key"), "License key should not be serialized");
        // Non-sensitive fields should be serialized
        assert!(json.contains("\"recording_mode\":\"push-to-talk\""));
        assert!(json.contains("\"show_indicator\":true"));
        assert!(json.contains("\"spoken_languages\":[\"en\",\"es\"]"));
    }

    #[test]
    fn test_stored_preferences_deserialization() {
        let json = r#"{
            "deepgram_api_key": "test_key",
            "recording_mode": "toggle",
            "show_indicator": false
        }"#;

        let prefs: StoredPreferences = serde_json::from_str(json).unwrap();
        assert_eq!(prefs.deepgram_api_key, Some("test_key".to_string()));
        assert_eq!(prefs.recording_mode, Some("toggle".to_string()));
        assert_eq!(prefs.show_indicator, Some(false));
        assert!(prefs.hotkey.is_none()); // Not in JSON, should be None
    }

    #[test]
    fn test_stored_preferences_partial_deserialization() {
        // Test that missing fields default to None
        let json = r#"{}"#;
        let prefs: StoredPreferences = serde_json::from_str(json).unwrap();
        assert!(prefs.deepgram_api_key.is_none());
        assert!(prefs.recording_mode.is_none());
    }

    // ==================== Preferences Tests ====================

    #[test]
    fn test_preferences_serialization_skips_none() {
        let prefs = Preferences {
            recording_mode: "push-to-talk".to_string(),
            hotkey: "Option+Space".to_string(),
            show_indicator: true,
            play_sounds: true,
            microphone: "default".to_string(),
            language: "en-US".to_string(),
            deepgram_key: "".to_string(),
            anthropic_key: "".to_string(),
            transcription_provider: None,
            license_key: None,
            onboarding_complete: None,
            spoken_languages: None,
        };

        let json = serde_json::to_string(&prefs).unwrap();
        // Optional None fields should be skipped
        assert!(!json.contains("transcription_provider"));
        assert!(!json.contains("license_key"));
        assert!(!json.contains("onboarding_complete"));
        assert!(!json.contains("spoken_languages"));
    }

    #[test]
    fn test_preferences_serialization_includes_some() {
        let prefs = Preferences {
            recording_mode: "toggle".to_string(),
            hotkey: "Cmd+Shift+M".to_string(),
            show_indicator: false,
            play_sounds: false,
            microphone: "USB Microphone".to_string(),
            language: "es-ES".to_string(),
            deepgram_key: "dg_key".to_string(),
            anthropic_key: "sk_key".to_string(),
            transcription_provider: Some("whisperapi".to_string()),
            license_key: Some("license".to_string()),
            onboarding_complete: Some(true),
            spoken_languages: Some(vec!["en".to_string()]),
        };

        let json = serde_json::to_string(&prefs).unwrap();
        assert!(json.contains("\"transcription_provider\":\"whisperapi\""));
        assert!(json.contains("\"license_key\":\"license\""));
        assert!(json.contains("\"onboarding_complete\":true"));
    }

    // ==================== Provider Roundtrip Tests ====================

    #[test]
    fn test_provider_roundtrip() {
        // Test that from_string -> to_string is consistent
        for provider_str in &["deepgram", "whisperapi", "whisperlocal"] {
            let provider = TranscriptionProvider::from_string(provider_str);
            let result = provider.to_string();
            assert_eq!(&result, *provider_str);
        }
    }
}