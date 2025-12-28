use serde::{Deserialize, Serialize};

/// The mode of operation based on whether text is selected
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DictationMode {
    /// Normal dictation - transcribe speech and insert text
    Dictation,
    /// Command mode - transform selected text using voice command
    Command,
}

impl Default for DictationMode {
    fn default() -> Self {
        DictationMode::Dictation
    }
}

/// Application recording state machine
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordingState {
    /// Ready to record, not doing anything
    Idle,
    /// Actively capturing audio from microphone
    Recording,
    /// Audio captured, sending to Deepgram for transcription
    Transcribing,
    /// Transcript received, sending to Claude for enhancement (Dictation Mode)
    Enhancing,
    /// Command Mode: transforming selected text with voice command
    Transforming,
    /// Something went wrong
    Error,
}

impl Default for RecordingState {
    fn default() -> Self {
        RecordingState::Idle
    }
}

impl RecordingState {
    /// Check if we can start recording from current state
    pub fn can_start_recording(&self) -> bool {
        matches!(self, RecordingState::Idle | RecordingState::Error)
    }

    /// Check if we can stop recording from current state
    pub fn can_stop_recording(&self) -> bool {
        matches!(self, RecordingState::Recording)
    }

    /// Check if we can cancel from current state
    pub fn can_cancel(&self) -> bool {
        matches!(
            self,
            RecordingState::Recording
                | RecordingState::Transcribing
                | RecordingState::Enhancing
                | RecordingState::Transforming
        )
    }
}

/// State change event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateChangeEvent {
    pub state: RecordingState,
    pub message: Option<String>,
    pub recording_duration_ms: Option<u64>,
    /// The mode of operation (dictation or command)
    #[serde(default)]
    pub mode: DictationMode,
}

/// Audio level event payload for waveform visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioLevelEvent {
    /// RMS level normalized 0.0 to 1.0
    pub level: f32,
    /// Peak level normalized 0.0 to 1.0
    pub peak: f32,
}

/// Transcription complete event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionCompleteEvent {
    pub raw_transcript: String,
    pub enhanced_text: String,
    pub copied_to_clipboard: bool,
}

/// Error event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEvent {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
    pub fallback_text: Option<String>,
}

impl ErrorEvent {
    pub fn no_audio_device() -> Self {
        ErrorEvent {
            code: "NO_AUDIO_DEVICE".to_string(),
            message: "No microphone found. Please connect a microphone.".to_string(),
            recoverable: true,
            fallback_text: None,
        }
    }

    pub fn no_audio_captured() -> Self {
        ErrorEvent {
            code: "NO_AUDIO_CAPTURED".to_string(),
            message: "No audio was captured. Please try again.".to_string(),
            recoverable: true,
            fallback_text: None,
        }
    }

    pub fn deepgram_error(msg: &str) -> Self {
        ErrorEvent {
            code: "DEEPGRAM_ERROR".to_string(),
            message: format!("Transcription failed: {}", msg),
            recoverable: true,
            fallback_text: None,
        }
    }

    pub fn claude_error(msg: &str, fallback: Option<String>) -> Self {
        ErrorEvent {
            code: "CLAUDE_ERROR".to_string(),
            message: format!("Enhancement failed: {}", msg),
            recoverable: true,
            fallback_text: fallback,
        }
    }

    pub fn whisper_error(msg: &str) -> Self {
        ErrorEvent {
            code: "WHISPER_ERROR".to_string(),
            message: format!("Transcription failed: {}", msg),
            recoverable: true,
            fallback_text: None,
        }
    }

    pub fn model_not_loaded() -> Self {
        ErrorEvent {
            code: "MODEL_NOT_LOADED".to_string(),
            message: "Whisper model not loaded. Please download a model in settings.".to_string(),
            recoverable: true,
            fallback_text: None,
        }
    }

    pub fn no_transcription_provider() -> Self {
        ErrorEvent {
            code: "NO_TRANSCRIPTION_PROVIDER".to_string(),
            message: "No transcription provider configured. Please set up Deepgram API key or activate a license.".to_string(),
            recoverable: true,
            fallback_text: None,
        }
    }

    pub fn groq_error(msg: &str, fallback: Option<String>) -> Self {
        ErrorEvent {
            code: "GROQ_ERROR".to_string(),
            message: format!("Groq LLM error: {}", msg),
            recoverable: true,
            fallback_text: fallback,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== DictationMode Tests ====================

    #[test]
    fn test_dictation_mode_default() {
        let mode = DictationMode::default();
        assert_eq!(mode, DictationMode::Dictation);
    }

    #[test]
    fn test_dictation_mode_serialization() {
        let dictation = DictationMode::Dictation;
        let command = DictationMode::Command;

        assert_eq!(serde_json::to_string(&dictation).unwrap(), "\"dictation\"");
        assert_eq!(serde_json::to_string(&command).unwrap(), "\"command\"");
    }

    #[test]
    fn test_dictation_mode_deserialization() {
        let dictation: DictationMode = serde_json::from_str("\"dictation\"").unwrap();
        let command: DictationMode = serde_json::from_str("\"command\"").unwrap();

        assert_eq!(dictation, DictationMode::Dictation);
        assert_eq!(command, DictationMode::Command);
    }

    // ==================== RecordingState Tests ====================

    #[test]
    fn test_recording_state_default() {
        let state = RecordingState::default();
        assert_eq!(state, RecordingState::Idle);
    }

    #[test]
    fn test_recording_state_serialization() {
        assert_eq!(serde_json::to_string(&RecordingState::Idle).unwrap(), "\"idle\"");
        assert_eq!(serde_json::to_string(&RecordingState::Recording).unwrap(), "\"recording\"");
        assert_eq!(serde_json::to_string(&RecordingState::Transcribing).unwrap(), "\"transcribing\"");
        assert_eq!(serde_json::to_string(&RecordingState::Enhancing).unwrap(), "\"enhancing\"");
        assert_eq!(serde_json::to_string(&RecordingState::Transforming).unwrap(), "\"transforming\"");
        assert_eq!(serde_json::to_string(&RecordingState::Error).unwrap(), "\"error\"");
    }

    #[test]
    fn test_can_start_recording_from_idle() {
        let state = RecordingState::Idle;
        assert!(state.can_start_recording());
    }

    #[test]
    fn test_can_start_recording_from_error() {
        let state = RecordingState::Error;
        assert!(state.can_start_recording());
    }

    #[test]
    fn test_cannot_start_recording_while_recording() {
        let state = RecordingState::Recording;
        assert!(!state.can_start_recording());
    }

    #[test]
    fn test_cannot_start_recording_while_transcribing() {
        let state = RecordingState::Transcribing;
        assert!(!state.can_start_recording());
    }

    #[test]
    fn test_cannot_start_recording_while_enhancing() {
        let state = RecordingState::Enhancing;
        assert!(!state.can_start_recording());
    }

    #[test]
    fn test_cannot_start_recording_while_transforming() {
        let state = RecordingState::Transforming;
        assert!(!state.can_start_recording());
    }

    #[test]
    fn test_can_stop_recording_while_recording() {
        let state = RecordingState::Recording;
        assert!(state.can_stop_recording());
    }

    #[test]
    fn test_cannot_stop_recording_from_idle() {
        let state = RecordingState::Idle;
        assert!(!state.can_stop_recording());
    }

    #[test]
    fn test_cannot_stop_recording_while_transcribing() {
        let state = RecordingState::Transcribing;
        assert!(!state.can_stop_recording());
    }

    #[test]
    fn test_can_cancel_while_recording() {
        let state = RecordingState::Recording;
        assert!(state.can_cancel());
    }

    #[test]
    fn test_can_cancel_while_transcribing() {
        let state = RecordingState::Transcribing;
        assert!(state.can_cancel());
    }

    #[test]
    fn test_can_cancel_while_enhancing() {
        let state = RecordingState::Enhancing;
        assert!(state.can_cancel());
    }

    #[test]
    fn test_can_cancel_while_transforming() {
        let state = RecordingState::Transforming;
        assert!(state.can_cancel());
    }

    #[test]
    fn test_cannot_cancel_from_idle() {
        let state = RecordingState::Idle;
        assert!(!state.can_cancel());
    }

    #[test]
    fn test_cannot_cancel_from_error() {
        let state = RecordingState::Error;
        assert!(!state.can_cancel());
    }

    // ==================== StateChangeEvent Tests ====================

    #[test]
    fn test_state_change_event_serialization() {
        let event = StateChangeEvent {
            state: RecordingState::Recording,
            message: Some("Recording started".to_string()),
            recording_duration_ms: Some(1500),
            mode: DictationMode::Dictation,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"state\":\"recording\""));
        assert!(json.contains("\"message\":\"Recording started\""));
        assert!(json.contains("\"recordingDurationMs\":1500"));
        assert!(json.contains("\"mode\":\"dictation\""));
    }

    #[test]
    fn test_state_change_event_with_none_values() {
        let event = StateChangeEvent {
            state: RecordingState::Idle,
            message: None,
            recording_duration_ms: None,
            mode: DictationMode::default(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"state\":\"idle\""));
        assert!(json.contains("\"message\":null"));
    }

    // ==================== AudioLevelEvent Tests ====================

    #[test]
    fn test_audio_level_event_serialization() {
        let event = AudioLevelEvent {
            level: 0.75,
            peak: 0.95,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"level\":0.75"));
        assert!(json.contains("\"peak\":0.95"));
    }

    // ==================== TranscriptionCompleteEvent Tests ====================

    #[test]
    fn test_transcription_complete_event_serialization() {
        let event = TranscriptionCompleteEvent {
            raw_transcript: "hello world".to_string(),
            enhanced_text: "Hello, world!".to_string(),
            copied_to_clipboard: true,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"rawTranscript\":\"hello world\""));
        assert!(json.contains("\"enhancedText\":\"Hello, world!\""));
        assert!(json.contains("\"copiedToClipboard\":true"));
    }

    // ==================== ErrorEvent Tests ====================

    #[test]
    fn test_error_event_no_audio_device() {
        let event = ErrorEvent::no_audio_device();
        assert_eq!(event.code, "NO_AUDIO_DEVICE");
        assert!(event.message.contains("No microphone found"));
        assert!(event.recoverable);
    }

    #[test]
    fn test_error_event_no_audio_captured() {
        let event = ErrorEvent::no_audio_captured();
        assert_eq!(event.code, "NO_AUDIO_CAPTURED");
        assert!(event.message.contains("No audio was captured"));
        assert!(event.recoverable);
    }

    #[test]
    fn test_error_event_deepgram_error() {
        let event = ErrorEvent::deepgram_error("API timeout");
        assert_eq!(event.code, "DEEPGRAM_ERROR");
        assert!(event.message.contains("API timeout"));
        assert!(event.recoverable);
    }

    #[test]
    fn test_error_event_claude_error_with_fallback() {
        let event = ErrorEvent::claude_error("Rate limited", Some("raw text".to_string()));
        assert_eq!(event.code, "CLAUDE_ERROR");
        assert!(event.message.contains("Rate limited"));
        assert_eq!(event.fallback_text, Some("raw text".to_string()));
    }

    #[test]
    fn test_error_event_claude_error_without_fallback() {
        let event = ErrorEvent::claude_error("API error", None);
        assert_eq!(event.code, "CLAUDE_ERROR");
        assert!(event.fallback_text.is_none());
    }

    #[test]
    fn test_error_event_whisper_error() {
        let event = ErrorEvent::whisper_error("Model inference failed");
        assert_eq!(event.code, "WHISPER_ERROR");
        assert!(event.message.contains("Model inference failed"));
    }

    #[test]
    fn test_error_event_model_not_loaded() {
        let event = ErrorEvent::model_not_loaded();
        assert_eq!(event.code, "MODEL_NOT_LOADED");
        assert!(event.message.contains("not loaded"));
    }

    #[test]
    fn test_error_event_no_transcription_provider() {
        let event = ErrorEvent::no_transcription_provider();
        assert_eq!(event.code, "NO_TRANSCRIPTION_PROVIDER");
        assert!(event.message.contains("No transcription provider"));
    }

    #[test]
    fn test_error_event_groq_error() {
        let event = ErrorEvent::groq_error("API limit", Some("fallback".to_string()));
        assert_eq!(event.code, "GROQ_ERROR");
        assert!(event.message.contains("API limit"));
        assert_eq!(event.fallback_text, Some("fallback".to_string()));
    }

    #[test]
    fn test_error_event_serialization() {
        let event = ErrorEvent {
            code: "TEST_ERROR".to_string(),
            message: "Test message".to_string(),
            recoverable: false,
            fallback_text: Some("fallback".to_string()),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"code\":\"TEST_ERROR\""));
        assert!(json.contains("\"message\":\"Test message\""));
        assert!(json.contains("\"recoverable\":false"));
        assert!(json.contains("\"fallbackText\":\"fallback\""));
    }
}
