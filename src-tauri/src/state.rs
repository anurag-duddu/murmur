use serde::{Deserialize, Serialize};

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
    /// Transcript received, sending to Claude for enhancement
    Enhancing,
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
            RecordingState::Recording | RecordingState::Transcribing | RecordingState::Enhancing
        )
    }

    /// Check if we're in a busy state (recording or processing)
    pub fn is_busy(&self) -> bool {
        !matches!(self, RecordingState::Idle | RecordingState::Error)
    }
}

/// State change event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateChangeEvent {
    pub state: RecordingState,
    pub message: Option<String>,
    pub recording_duration_ms: Option<u64>,
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
    pub fn mic_permission_denied() -> Self {
        ErrorEvent {
            code: "MIC_PERMISSION_DENIED".to_string(),
            message: "Microphone access denied. Please grant permission in System Preferences.".to_string(),
            recoverable: true,
            fallback_text: None,
        }
    }

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

    pub fn network_error(msg: &str) -> Self {
        ErrorEvent {
            code: "NETWORK_ERROR".to_string(),
            message: format!("Network error: {}", msg),
            recoverable: true,
            fallback_text: None,
        }
    }

    // New: Whisper-related errors

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

    pub fn model_download_failed(msg: &str) -> Self {
        ErrorEvent {
            code: "MODEL_DOWNLOAD_FAILED".to_string(),
            message: format!("Failed to download model: {}", msg),
            recoverable: true,
            fallback_text: None,
        }
    }

    pub fn license_invalid(msg: &str) -> Self {
        ErrorEvent {
            code: "LICENSE_INVALID".to_string(),
            message: format!("License validation failed: {}", msg),
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
}
