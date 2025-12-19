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

/// Stored preferences that persist to disk
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StoredPreferences {
    pub deepgram_api_key: Option<String>,
    pub groq_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub recording_mode: Option<String>,
    pub hotkey: Option<String>,
    pub show_indicator: Option<bool>,
    pub play_sounds: Option<bool>,
    pub microphone: Option<String>,
    pub language: Option<String>,
    // New: Transcription provider settings
    pub transcription_provider: Option<String>,
    pub license_key: Option<String>,
    // Onboarding settings
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

        // Load stored preferences from file
        let stored = StoredPreferences::load();

        // Priority: stored preferences > env vars > defaults
        AppConfig {
            deepgram_api_key: stored.deepgram_api_key
                .or_else(|| env::var("DEEPGRAM_API_KEY").ok().filter(|s| !s.is_empty())),
            groq_api_key: stored.groq_api_key
                .or_else(|| env::var("GROQ_API_KEY").ok().filter(|s| !s.is_empty())),
            anthropic_api_key: stored.anthropic_api_key
                .or_else(|| env::var("ANTHROPIC_API_KEY").ok().filter(|s| !s.is_empty())),
            recording_mode: stored.recording_mode
                .unwrap_or_else(|| env::var("DEFAULT_RECORDING_MODE")
                    .unwrap_or_else(|_| "push-to-talk".to_string())),
            hotkey: stored.hotkey
                .unwrap_or_else(|| env::var("DEFAULT_HOTKEY")
                    .unwrap_or_else(|_| "Option+Space".to_string())),
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
            // New: Transcription provider - auto-detect based on available keys
            transcription_provider: stored.transcription_provider
                .map(|s| TranscriptionProvider::from_string(&s))
                .unwrap_or_else(TranscriptionProvider::default),
            license_key: stored.license_key
                .or_else(|| env::var("LICENSE_KEY").ok().filter(|s| !s.is_empty())),
        }
    }

    pub fn update_from_preferences(&mut self, prefs: Preferences) -> Result<(), String> {
        // Update in-memory config
        if !prefs.deepgram_key.is_empty() {
            self.deepgram_api_key = Some(prefs.deepgram_key.clone());
        }
        if !prefs.anthropic_key.is_empty() {
            self.anthropic_api_key = Some(prefs.anthropic_key.clone());
        }
        self.recording_mode = prefs.recording_mode.clone();
        self.hotkey = prefs.hotkey.clone();
        self.show_indicator = prefs.show_indicator;
        self.play_sounds = prefs.play_sounds;
        self.microphone = prefs.microphone.clone();
        self.language = prefs.language.clone();
        // New: Update transcription provider and license key
        if let Some(provider) = &prefs.transcription_provider {
            self.transcription_provider = TranscriptionProvider::from_string(provider);
        }
        if let Some(license) = &prefs.license_key {
            if !license.is_empty() {
                self.license_key = Some(license.clone());
            }
        }

        // Persist to disk
        let stored = StoredPreferences {
            deepgram_api_key: if prefs.deepgram_key.is_empty() {
                self.deepgram_api_key.clone()
            } else {
                Some(prefs.deepgram_key)
            },
            groq_api_key: self.groq_api_key.clone(),
            anthropic_api_key: if prefs.anthropic_key.is_empty() {
                self.anthropic_api_key.clone()
            } else {
                Some(prefs.anthropic_key)
            },
            recording_mode: Some(prefs.recording_mode),
            hotkey: Some(prefs.hotkey),
            show_indicator: Some(prefs.show_indicator),
            play_sounds: Some(prefs.play_sounds),
            microphone: Some(prefs.microphone),
            language: Some(prefs.language),
            // New: Save transcription provider and license key
            transcription_provider: Some(self.transcription_provider.to_string()),
            license_key: self.license_key.clone(),
            // Onboarding settings
            onboarding_complete: prefs.onboarding_complete,
            spoken_languages: prefs.spoken_languages,
        };
        stored.save()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    pub recording_mode: String,
    pub hotkey: String,
    pub show_indicator: bool,
    pub play_sounds: bool,
    pub microphone: String,
    pub language: String,
    pub deepgram_key: String,
    #[serde(rename = "claudeKey")]
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