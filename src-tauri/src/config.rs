use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

/// Stored preferences that persist to disk.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StoredPreferences {
    // Non-sensitive preferences (stored in JSON file)
    pub recording_mode: Option<String>,
    pub hotkey: Option<String>,
    pub show_indicator: Option<bool>,
    pub play_sounds: Option<bool>,
    pub microphone: Option<String>,
    pub language: Option<String>,
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
    // Groq API key - loaded from environment variable
    pub groq_api_key: Option<String>,
    // Recording settings
    pub recording_mode: String,
    pub hotkey: String,
    pub max_recording_duration: u32,
    pub audio_sample_rate: u32,
    pub audio_channels: u16,
    // UI settings
    pub show_indicator: bool,
    pub play_sounds: bool,
    pub microphone: String,
    pub language: String,
}

impl AppConfig {
    pub fn load() -> Self {
        // Load .env file if it exists (for development)
        dotenv().ok();

        // Load stored preferences from file
        let stored = StoredPreferences::load();

        // Groq API key from environment variable only
        let groq_api_key = env::var("GROQ_API_KEY").ok().filter(|s| !s.is_empty());

        AppConfig {
            groq_api_key,
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
            show_indicator: stored.show_indicator.unwrap_or(true),
            play_sounds: stored.play_sounds.unwrap_or(true),
            microphone: stored.microphone.unwrap_or_else(|| "default".to_string()),
            language: stored.language.unwrap_or_else(|| "en-US".to_string()),
        }
    }

    pub fn update_from_preferences(&mut self, prefs: Preferences) -> Result<(), String> {
        // Update in-memory config
        self.recording_mode = prefs.recording_mode.clone();
        self.hotkey = prefs.hotkey.clone();
        self.show_indicator = prefs.show_indicator;
        self.play_sounds = prefs.play_sounds;
        self.microphone = prefs.microphone.clone();
        self.language = prefs.language.clone();

        // Persist non-sensitive preferences to disk
        let stored = StoredPreferences {
            recording_mode: Some(prefs.recording_mode),
            hotkey: Some(prefs.hotkey),
            show_indicator: Some(prefs.show_indicator),
            play_sounds: Some(prefs.play_sounds),
            microphone: Some(prefs.microphone),
            language: Some(prefs.language),
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onboarding_complete: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spoken_languages: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stored_preferences_default() {
        let prefs = StoredPreferences::default();
        assert!(prefs.recording_mode.is_none());
        assert!(prefs.hotkey.is_none());
        assert!(prefs.show_indicator.is_none());
        assert!(prefs.play_sounds.is_none());
        assert!(prefs.microphone.is_none());
        assert!(prefs.language.is_none());
        assert!(prefs.onboarding_complete.is_none());
        assert!(prefs.spoken_languages.is_none());
    }

    #[test]
    fn test_stored_preferences_serialization() {
        let prefs = StoredPreferences {
            recording_mode: Some("push-to-talk".to_string()),
            hotkey: Some("Option+Space".to_string()),
            show_indicator: Some(true),
            play_sounds: Some(false),
            microphone: Some("default".to_string()),
            language: Some("en-US".to_string()),
            onboarding_complete: Some(true),
            spoken_languages: Some(vec!["en".to_string(), "es".to_string()]),
        };

        let json = serde_json::to_string(&prefs).unwrap();
        assert!(json.contains("\"recording_mode\":\"push-to-talk\""));
        assert!(json.contains("\"show_indicator\":true"));
        assert!(json.contains("\"spoken_languages\":[\"en\",\"es\"]"));
    }

    #[test]
    fn test_stored_preferences_deserialization() {
        let json = r#"{
            "recording_mode": "toggle",
            "show_indicator": false
        }"#;

        let prefs: StoredPreferences = serde_json::from_str(json).unwrap();
        assert_eq!(prefs.recording_mode, Some("toggle".to_string()));
        assert_eq!(prefs.show_indicator, Some(false));
        assert!(prefs.hotkey.is_none());
    }
}
