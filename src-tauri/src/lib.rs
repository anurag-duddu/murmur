use std::sync::Mutex;
use std::time::Instant;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Emitter, Manager, Runtime, State, WindowEvent,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

mod audio;
mod config;
mod error;
mod groq_llm;
mod http_client;
mod ide;
mod permissions;
mod platform;
mod rate_limit;
mod signing;
mod state;
mod styles;
mod whisper_api;

use audio::{encode_samples_to_wav, AudioRecorder};
use config::AppConfig;
use groq_llm::{GroqLlmClient, UserIntent};
use state::{DictationMode, ErrorEvent, RecordingState, StateChangeEvent, TranscriptionCompleteEvent};

// ============================================================================
// CONSTANTS
// ============================================================================

/// Offset from bottom of screen for overlay positioning (pixels)
const OVERLAY_BOTTOM_OFFSET: i32 = 300;

/// How long to display "Done!" state before hiding overlay (ms)
/// Reduced from 300ms for snappier feel while still showing completion
const DONE_DISPLAY_DELAY_MS: u64 = 100;

/// How long to wait for the target app to regain focus after hiding overlay (ms)
/// Reduced from 100ms - AppleScript activation is fast
const APP_FOCUS_WAIT_MS: u64 = 30;

/// Application state - single source of truth
pub struct AppState {
    recorder: Mutex<AudioRecorder>,
    config: Mutex<AppConfig>,
    recording_state: Mutex<RecordingState>,
    recording_start: Mutex<Option<Instant>>,
    /// Current mode: Dictation (default) or Command (when text is selected)
    dictation_mode: Mutex<DictationMode>,
    /// Selected text captured at recording start (for Command Mode)
    selected_text: Mutex<Option<String>>,
    /// Active app captured at recording start (for context-aware styles)
    active_style: Mutex<Option<styles::Style>>,
    /// Bundle ID of active app (for IDE detection)
    active_bundle_id: Mutex<Option<String>>,
    /// Workspace file index for file tagging (built on startup)
    workspace_index: Mutex<Option<ide::file_index::WorkspaceIndex>>,
}

impl AppState {
    fn new(config: AppConfig) -> Self {
        AppState {
            recorder: Mutex::new(AudioRecorder::new()),
            config: Mutex::new(config),
            recording_state: Mutex::new(RecordingState::Idle),
            recording_start: Mutex::new(None),
            dictation_mode: Mutex::new(DictationMode::Dictation),
            selected_text: Mutex::new(None),
            active_style: Mutex::new(None),
            active_bundle_id: Mutex::new(None),
            workspace_index: Mutex::new(None),
        }
    }

    fn get_state(&self) -> RecordingState {
        self.recording_state
            .lock()
            .map(|s| s.clone())
            .unwrap_or(RecordingState::Error)
    }

    fn set_state(&self, new_state: RecordingState) {
        if let Ok(mut state) = self.recording_state.lock() {
            *state = new_state;
        }
    }

    fn get_mode(&self) -> DictationMode {
        self.dictation_mode
            .lock()
            .map(|m| *m)
            .unwrap_or(DictationMode::Dictation)
    }

    fn set_mode(&self, mode: DictationMode) {
        if let Ok(mut m) = self.dictation_mode.lock() {
            *m = mode;
        }
    }

    fn get_selected_text(&self) -> Option<String> {
        self.selected_text
            .lock()
            .ok()
            .and_then(|t| t.clone())
    }

    fn set_selected_text(&self, text: Option<String>) {
        if let Ok(mut t) = self.selected_text.lock() {
            *t = text;
        }
    }

    fn get_active_style(&self) -> Option<styles::Style> {
        self.active_style
            .lock()
            .ok()
            .and_then(|s| s.clone())
    }

    fn set_active_style(&self, style: Option<styles::Style>) {
        if let Ok(mut s) = self.active_style.lock() {
            *s = style;
        }
    }

    fn get_active_bundle_id(&self) -> Option<String> {
        self.active_bundle_id
            .lock()
            .ok()
            .and_then(|b| b.clone())
    }

    fn set_active_bundle_id(&self, bundle_id: Option<String>) {
        if let Ok(mut b) = self.active_bundle_id.lock() {
            *b = bundle_id;
        }
    }

    fn get_workspace_index(&self) -> Option<ide::file_index::WorkspaceIndex> {
        self.workspace_index
            .lock()
            .ok()
            .and_then(|idx| idx.clone())
    }

    fn set_workspace_index(&self, index: Option<ide::file_index::WorkspaceIndex>) {
        if let Ok(mut idx) = self.workspace_index.lock() {
            *idx = index;
        }
    }

    fn get_recording_duration_ms(&self) -> Option<u64> {
        self.recording_start
            .lock()
            .ok()
            .and_then(|start| start.map(|s| s.elapsed().as_millis() as u64))
    }

    /// Execute a closure with read access to the config
    fn with_config<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&AppConfig) -> R,
    {
        self.config
            .lock()
            .map(|c| f(&c))
            .map_err(|e| format!("Failed to lock config: {}", e))
    }

    /// Execute a closure with mutable access to the config
    #[allow(dead_code)]
    fn with_config_mut<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&mut AppConfig) -> R,
    {
        self.config
            .lock()
            .map(|mut c| f(&mut c))
            .map_err(|e| format!("Failed to lock config: {}", e))
    }

    /// Execute a closure with mutable access to the recorder
    fn with_recorder_mut<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&mut AudioRecorder) -> R,
    {
        self.recorder
            .lock()
            .map(|mut r| f(&mut r))
            .map_err(|e| format!("Failed to lock recorder: {}", e))
    }

    /// Set the recording start time
    fn set_recording_start(&self, start: Option<Instant>) {
        if let Ok(mut s) = self.recording_start.lock() {
            *s = start;
        }
    }
}

/// Emit state change event - directly to overlay window for reliable delivery
fn emit_state_change(app: &AppHandle, state: &AppState, message: Option<String>) {
    let event = StateChangeEvent {
        state: state.get_state(),
        message,
        recording_duration_ms: state.get_recording_duration_ms(),
        mode: state.get_mode(),
    };

    // Emit directly to overlay window (not broadcast) for reliable delivery
    if let Some(overlay) = app.get_webview_window("overlay") {
        if let Err(e) = overlay.emit("state-changed", &event) {
            eprintln!("Failed to emit state change to overlay: {}", e);
        }
    }

    // Also broadcast to other windows (main window might want to know)
    let _ = app.emit("state-changed", &event);

    println!("State changed to: {:?}", event.state);
}

/// Emit error event
fn emit_error(app: &AppHandle, error: ErrorEvent) {
    if let Err(e) = app.emit("recording-error", &error) {
        eprintln!("Failed to emit error: {}", e);
    }
    println!("Error: {} - {}", error.code, error.message);
}

// ============================================================================
// TAURI COMMANDS
// ============================================================================

#[tauri::command]
fn get_recording_state(state: State<'_, AppState>) -> RecordingState {
    state.get_state()
}

/// Get full overlay state for initialization - like VoiceInk passing state to views
#[tauri::command]
fn get_overlay_state(state: State<'_, AppState>) -> StateChangeEvent {
    StateChangeEvent {
        state: state.get_state(),
        message: None,
        recording_duration_ms: state.get_recording_duration_ms(),
        mode: state.get_mode(),
    }
}

#[tauri::command]
async fn start_recording(app_handle: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    // Check if we can start
    let current_state = state.get_state();
    if !current_state.can_start_recording() {
        return Err(format!(
            "Cannot start recording from state: {:?}",
            current_state
        ));
    }

    // =========================================================================
    // PHASE 1: INSTANT RESPONSE (no blocking operations)
    // =========================================================================

    // Set state to Recording immediately (default to Dictation mode)
    state.set_state(RecordingState::Recording);
    state.set_mode(DictationMode::Dictation); // Default, may update async
    state.set_recording_start(Some(Instant::now()));

    // Show overlay IMMEDIATELY - no delay
    if let Some(overlay) = app_handle.get_webview_window("overlay") {
        position_overlay_center_bottom(&overlay, OVERLAY_BOTTOM_OFFSET);
        let _ = overlay.show();
    }

    // Emit initial state (Dictation mode by default)
    emit_state_change(&app_handle, &state, Some("Recording...".to_string()));

    // Start audio capture IMMEDIATELY
    let result = {
        let mut recorder = state
            .recorder
            .lock()
            .map_err(|e| format!("Failed to lock recorder: {}", e))?;
        let selected_mic = permissions::get_selected_microphone_name();
        recorder.start_recording_with_device(app_handle.clone(), selected_mic)
    };

    if let Err(e) = result {
        state.set_state(RecordingState::Error);
        emit_error(&app_handle, ErrorEvent::no_audio_device());
        return Err(e);
    }

    // =========================================================================
    // PHASE 2: ASYNC CONTEXT CAPTURE (happens while user speaks)
    // =========================================================================

    let app_handle_for_context = app_handle.clone();
    std::thread::spawn(move || {
        let state: tauri::State<'_, AppState> = app_handle_for_context.state();

        // Only proceed if we're still recording
        if state.get_state() != RecordingState::Recording {
            return;
        }

        // 1. Detect active app for context-aware styles
        let active_style = styles::get_current_style();
        state.set_active_style(Some(active_style));

        // 2. Detect selection - if found, switch to Command Mode
        match platform::selection::get_selected_text() {
            Ok(text) => {
                state.set_mode(DictationMode::Command);
                state.set_selected_text(Some(text));

                // Update overlay to show Command Mode
                if state.get_state() == RecordingState::Recording {
                    emit_state_change(
                        &app_handle_for_context,
                        &state,
                        Some("Command Mode".to_string()),
                    );
                }
            }
            Err(_) => {
                // Already set to Dictation by default, no change needed
            }
        }
    });

    Ok(())
}

#[tauri::command]
async fn stop_recording(app_handle: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    // Check if we can stop
    let current_state = state.get_state();
    if !current_state.can_stop_recording() {
        return Err(format!(
            "Cannot stop recording from state: {:?}",
            current_state
        ));
    }

    // Capture the bundle_id BEFORE processing clears it
    let bundle_id = state.get_active_bundle_id();

    // Use shared processing logic
    let final_text = process_recording_stop(&app_handle, &state).await?;

    // Hide overlay, reactivate previous app, then insert text
    let app_clone = app_handle.clone();
    std::thread::spawn(move || {
        // Brief delay to show "Done!" state
        std::thread::sleep(std::time::Duration::from_millis(DONE_DISPLAY_DELAY_MS));
        hide_overlay(&app_clone);
        // Reactivate the previous app explicitly
        if let Some(bid) = bundle_id {
            activate_app_by_bundle_id(&bid);
        }
        // Wait for the app to regain focus
        std::thread::sleep(std::time::Duration::from_millis(APP_FOCUS_WAIT_MS));
        // Insert text while preserving clipboard
        insert_text_directly(&final_text);
    });

    Ok(())
}

/// Configuration extracted from app state for recording stop processing
struct RecordingStopConfig {
    language: String,
    spoken_languages: Vec<String>,
}

/// Shared logic for stopping a recording and processing the audio.
/// Used by both the Tauri command `stop_recording` and the shortcut handler.
async fn process_recording_stop(
    app_handle: &AppHandle,
    state: &AppState,
) -> Result<String, String> {
    // Update state to transcribing
    state.set_state(RecordingState::Transcribing);
    emit_state_change(app_handle, state, Some("Processing audio...".to_string()));

    // Get configuration
    let stored = config::StoredPreferences::load();
    let spoken_langs = stored
        .spoken_languages
        .unwrap_or_else(|| vec!["en".to_string()]);

    let config = state.with_config(|cfg| RecordingStopConfig {
        language: cfg.language.clone(),
        spoken_languages: spoken_langs,
    })?;

    // Stop recording and get audio data (always use Whisper format)
    let audio_samples_16khz = state.with_recorder_mut(|recorder| {
        recorder.stop_recording_for_whisper()
    })??;

    // Check if we have audio
    if audio_samples_16khz.is_empty() {
        state.set_state(RecordingState::Error);
        emit_error(app_handle, ErrorEvent::no_audio_captured());
        hide_overlay(app_handle);
        return Err("No audio captured".to_string());
    }

    // Transcribe using Groq Whisper API
    emit_state_change(app_handle, state, Some("Transcribing...".to_string()));

    let wav = encode_samples_to_wav(&audio_samples_16khz, 16000)?;
    let client = whisper_api::WhisperApiClient::new()?;
    let transcript = client
        .transcribe(&wav, &config.language, &config.spoken_languages)
        .await
        .map_err(|e| {
            state.set_state(RecordingState::Error);
            emit_error(app_handle, ErrorEvent::whisper_error(&e));
            hide_overlay(app_handle);
            e
        })?;

    if transcript.is_empty() {
        state.set_state(RecordingState::Error);
        emit_error(app_handle, ErrorEvent::no_audio_captured());
        hide_overlay(app_handle);
        return Err("No speech detected".to_string());
    }

    #[cfg(debug_assertions)]
    println!("Transcript (groq): {}", transcript);

    // Apply IDE transformations if we're in a code editor
    let active_bundle_id = state.get_active_bundle_id();
    let workspace_index = state.get_workspace_index();
    let transcript = if let Some(ref bundle_id) = active_bundle_id {
        if ide::is_ide(bundle_id) {
            let ide_context = ide::get_ide_context(bundle_id);
            let ide_settings = ide::IDESettings::default();
            let transformed = ide::apply_ide_transformations(
                &transcript,
                &ide_context,
                &ide_settings,
                workspace_index.as_ref(),
            );
            #[cfg(debug_assertions)]
            if transformed != transcript {
                println!("IDE transformed: {}", transformed);
            }
            transformed
        } else {
            transcript
        }
    } else {
        transcript
    };

    // Get context for mode-based processing
    let current_mode = state.get_mode();
    let selected_text_for_transform = state.get_selected_text();
    let active_style = state.get_active_style();

    // Process based on mode
    let final_text = match current_mode {
        DictationMode::Command => {
            // Command mode: classify intent and either transform or dictate
            let selected_text = match selected_text_for_transform {
                Some(text) => text,
                None => {
                    #[cfg(debug_assertions)]
                    println!(
                        "Command Mode but no selected text, falling back to raw transcript"
                    );
                    transcript.clone()
                }
            };

            let groq_client = GroqLlmClient::new()?;

            // Classify intent
            state.set_state(RecordingState::Transforming);
            emit_state_change(app_handle, state, Some("Analyzing...".to_string()));

            let intent = match groq_client.classify_intent(&transcript).await {
                Ok(i) => i,
                Err(e) => {
                    #[cfg(debug_assertions)]
                    println!("Intent classification failed, defaulting to Dictation: {}", e);
                    UserIntent::Dictation
                }
            };

            match intent {
                UserIntent::Command => {
                    emit_state_change(app_handle, state, Some("Transforming...".to_string()));
                    match groq_client.transform_text(&selected_text, &transcript).await {
                        Ok(transformed) => {
                            #[cfg(debug_assertions)]
                            println!("Transformed text: {}", transformed);
                            transformed
                        }
                        Err(e) => {
                            #[cfg(debug_assertions)]
                            println!("Transformation failed, keeping original: {}", e);
                            emit_error(
                                app_handle,
                                ErrorEvent::groq_error(&e, Some(selected_text.clone())),
                            );
                            selected_text
                        }
                    }
                }
                UserIntent::Dictation => {
                    #[cfg(debug_assertions)]
                    println!("Intent: Dictation - will replace selection with new content");
                    emit_state_change(app_handle, state, Some("Enhancing...".to_string()));

                    let style_prompt = active_style.as_ref().map(|s| {
                        #[cfg(debug_assertions)]
                        println!("Applying style: {} ({})", s.name, s.id);
                        s.prompt_modifier.as_str()
                    });

                    match groq_client.enhance_text(&transcript, style_prompt).await {
                        Ok(enhanced) => {
                            #[cfg(debug_assertions)]
                            println!("Enhanced dictation (replacing selection): {}", enhanced);
                            enhanced
                        }
                        Err(e) => {
                            #[cfg(debug_assertions)]
                            println!("Enhancement failed, using raw transcript: {}", e);
                            emit_error(
                                app_handle,
                                ErrorEvent::groq_error(&e, Some(transcript.clone())),
                            );
                            transcript.clone()
                        }
                    }
                }
            }
        }
        DictationMode::Dictation => {
            // Dictation mode: enhance with Groq
            state.set_state(RecordingState::Enhancing);
            emit_state_change(app_handle, state, Some("Enhancing...".to_string()));

            let style_prompt = active_style.as_ref().map(|s| {
                #[cfg(debug_assertions)]
                println!("Applying style: {} ({})", s.name, s.id);
                s.prompt_modifier.as_str()
            });

            let groq_client = GroqLlmClient::new()?;

            #[cfg(debug_assertions)]
            println!("Before LLM enhancement: {}", transcript);

            match groq_client.enhance_text(&transcript, style_prompt).await {
                Ok(enhanced) => {
                    #[cfg(debug_assertions)]
                    println!("Enhanced with Groq: {}", enhanced);
                    enhanced
                }
                Err(groq_error) => {
                    #[cfg(debug_assertions)]
                    println!("Groq enhancement failed: {}", groq_error);
                    emit_error(
                        app_handle,
                        ErrorEvent::groq_error(&groq_error, Some(transcript.clone())),
                    );
                    transcript.clone()
                }
            }
        }
    };

    // Clean up punctuation attached to @-tagged filenames
    let final_text = ide::file_tagger::cleanup_tagged_punctuation(&final_text);

    // Emit completion
    let completion_event = TranscriptionCompleteEvent {
        raw_transcript: transcript,
        enhanced_text: final_text.clone(),
        copied_to_clipboard: false,
    };

    if let Err(e) = app_handle.emit("transcription-complete", &completion_event) {
        eprintln!("Failed to emit completion: {}", e);
    }

    // Reset state
    state.set_state(RecordingState::Idle);
    state.set_mode(DictationMode::Dictation);
    state.set_selected_text(None);
    state.set_active_style(None);
    state.set_active_bundle_id(None);
    state.set_recording_start(None);
    emit_state_change(app_handle, state, Some("Done!".to_string()));

    Ok(final_text)
}

#[tauri::command]
async fn cancel_recording(app_handle: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let current_state = state.get_state();
    if !current_state.can_cancel() {
        return Ok(()); // Nothing to cancel
    }

    // Stop recording if active
    if current_state == RecordingState::Recording {
        let mut recorder = state
            .recorder
            .lock()
            .map_err(|e| format!("Failed to lock recorder: {}", e))?;
        let _ = recorder.stop_recording(); // Discard audio
    }

    // Return to idle
    state.set_state(RecordingState::Idle);
    state.set_recording_start(None);
    emit_state_change(&app_handle, &state, Some("Cancelled".to_string()));
    hide_overlay(&app_handle);

    Ok(())
}

#[tauri::command]
async fn toggle_recording(app_handle: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let current_state = state.get_state();

    match current_state {
        RecordingState::Idle | RecordingState::Error => {
            start_recording(app_handle, state).await
        }
        RecordingState::Recording => {
            stop_recording(app_handle, state).await
        }
        _ => {
            // Transcribing or Enhancing - can't toggle, maybe cancel?
            Ok(())
        }
    }
}

#[tauri::command]
async fn show_preferences(app_handle: AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window
            .show()
            .map_err(|e| format!("Failed to show window: {}", e))?;
        window
            .set_focus()
            .map_err(|e| format!("Failed to focus window: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
async fn update_preferences(
    state: State<'_, AppState>,
    preferences: config::Preferences,
) -> Result<(), String> {
    let mut config = state
        .config
        .lock()
        .map_err(|e| format!("Failed to lock config: {}", e))?;
    config.update_from_preferences(preferences)?;
    Ok(())
}

#[tauri::command]
async fn get_preferences(state: State<'_, AppState>) -> Result<config::Preferences, String> {
    let config = state
        .config
        .lock()
        .map_err(|e| format!("Failed to lock config: {}", e))?;

    // Load stored preferences to get spoken languages and language onboarding status
    let stored = config::StoredPreferences::load();

    Ok(config::Preferences {
        recording_mode: config.recording_mode.clone(),
        hotkey: config.hotkey.clone(),
        show_indicator: config.show_indicator,
        play_sounds: config.play_sounds,
        microphone: config.microphone.clone(),
        language: config.language.clone(),
        onboarding_complete: stored.onboarding_complete,
        spoken_languages: stored.spoken_languages,
    })
}

// ============================================================================
// PERMISSION AND ONBOARDING COMMANDS
// ============================================================================

#[tauri::command]
fn check_permissions() -> permissions::PermissionStatus {
    permissions::PermissionStatus {
        microphone: permissions::check_microphone_permission(),
        accessibility: permissions::check_accessibility_permission(),
    }
}

#[tauri::command]
fn get_microphones() -> Vec<permissions::MicrophoneDevice> {
    permissions::get_microphone_devices()
}

#[tauri::command]
fn request_microphone_permission() -> bool {
    permissions::request_microphone_permission()
}

#[tauri::command]
fn open_accessibility_settings() -> Result<(), String> {
    permissions::open_accessibility_settings()
}

#[tauri::command]
fn set_selected_microphone(device_id: String) -> Result<(), String> {
    // Input validation: device_id must be non-empty and reasonable length
    let trimmed = device_id.trim();
    if trimmed.is_empty() {
        return Err("Device ID cannot be empty".to_string());
    }
    if trimmed.len() > 512 {
        return Err("Device ID is too long".to_string());
    }

    // SECURITY: Validate character set to prevent path traversal and injection
    // Allow: alphanumeric, spaces, hyphens, underscores, dots, parentheses, colons
    // (common in device names like "MacBook Pro Microphone (Built-in)")
    if !trimmed.chars().all(|c| {
        c.is_alphanumeric()
            || c == ' '
            || c == '-'
            || c == '_'
            || c == '.'
            || c == '('
            || c == ')'
            || c == ':'
            || c == '\''
    }) {
        return Err("Device ID contains invalid characters".to_string());
    }

    // SECURITY: Prevent path traversal
    if trimmed.contains("..") || trimmed.contains('/') || trimmed.contains('\\') {
        return Err("Device ID contains invalid path characters".to_string());
    }

    permissions::set_selected_microphone(trimmed)
}

#[tauri::command]
fn is_onboarding_complete() -> bool {
    permissions::is_onboarding_complete()
}

#[tauri::command]
fn complete_onboarding(app_handle: AppHandle) -> Result<(), String> {
    permissions::mark_onboarding_complete()?;

    // Hide onboarding window and show that app is ready
    if let Some(onboarding) = app_handle.get_webview_window("onboarding") {
        let _ = onboarding.close();
    }

    println!("Onboarding completed, app is ready!");
    Ok(())
}

// ============================================================================
// WORKSPACE INDEX COMMANDS
// ============================================================================

/// Sensitive directories that should never be indexed for security reasons.
/// This list covers common locations for credentials, secrets, and private keys.
const BLOCKED_DIRECTORIES: &[&str] = &[
    // SSH and GPG keys
    ".ssh",
    ".gnupg",
    ".gpg",
    // Cloud provider credentials
    ".aws",
    ".kube",
    ".docker",
    ".config/gcloud",
    ".azure",
    ".config/doctl",
    // Package manager credentials
    ".npm",
    ".cargo/credentials",
    ".cargo/credentials.toml",
    ".pypirc",
    ".gem/credentials",
    // macOS keychains
    "Library/Keychains",
    // Environment and secret files (as directory components)
    ".env",
    ".env.local",
    ".env.production",
    ".env.development",
    // Password managers
    ".password-store",
    ".config/1Password",
    ".config/Bitwarden",
    // Other sensitive locations
    ".netrc",
    ".git-credentials",
    ".config/gh",  // GitHub CLI
    ".config/hub", // Hub CLI
    "credentials",
    "secrets",
    ".secrets",
];

/// Validate that a path is safe to index
fn validate_workspace_path(path: &std::path::Path) -> Result<std::path::PathBuf, String> {
    // Canonicalize to resolve symlinks and get absolute path
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("Failed to resolve path: {}", e))?;

    // Get home directory
    let home_dir = dirs::home_dir().ok_or("Could not determine home directory")?;

    // Path must be under home directory (security: prevent indexing system dirs)
    if !canonical.starts_with(&home_dir) {
        return Err(format!(
            "Workspace must be within your home directory: {}",
            home_dir.display()
        ));
    }

    // Check against blocked directories
    let relative_to_home = canonical
        .strip_prefix(&home_dir)
        .map_err(|_| "Path is not under home directory")?;

    let path_str = relative_to_home.to_string_lossy();
    for blocked in BLOCKED_DIRECTORIES {
        if path_str.starts_with(blocked) || path_str.contains(&format!("/{}/", blocked)) {
            return Err(format!(
                "Cannot index sensitive directory: {}",
                blocked
            ));
        }
    }

    Ok(canonical)
}

/// Set the workspace root and build the file index
#[tauri::command]
fn set_workspace_root(
    state: State<'_, AppState>,
    path: String,
) -> Result<WorkspaceIndexStatus, String> {
    use std::path::Path;

    let workspace_path = Path::new(&path);

    if !workspace_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    if !workspace_path.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }

    // Validate path security (canonicalize, check boundaries, block sensitive dirs)
    let validated_path = validate_workspace_path(workspace_path)?;

    println!("[WORKSPACE] Building index for: {}", validated_path.display());

    match ide::file_index::WorkspaceIndex::build(&validated_path) {
        Ok(index) => {
            let file_count = index.file_count();
            let files_skipped = index.files_skipped;
            state.set_workspace_index(Some(index));
            println!("[WORKSPACE] Index built: {} files indexed, {} skipped", file_count, files_skipped);
            Ok(WorkspaceIndexStatus {
                indexed: true,
                root: Some(validated_path.to_string_lossy().to_string()),
                file_count,
                files_skipped,
            })
        }
        Err(e) => {
            println!("[WORKSPACE] Failed to build index: {}", e);
            Err(e)
        }
    }
}

/// Get the current workspace index status
#[tauri::command]
fn get_workspace_status(state: State<'_, AppState>) -> WorkspaceIndexStatus {
    match state.get_workspace_index() {
        Some(index) => WorkspaceIndexStatus {
            indexed: true,
            root: Some(index.root.to_string_lossy().to_string()),
            file_count: index.file_count(),
            files_skipped: index.files_skipped,
        },
        None => WorkspaceIndexStatus {
            indexed: false,
            root: None,
            file_count: 0,
            files_skipped: 0,
        },
    }
}

/// Clear the workspace index
#[tauri::command]
fn clear_workspace_index(state: State<'_, AppState>) {
    state.set_workspace_index(None);
    println!("[WORKSPACE] Index cleared");
}

/// Workspace index status response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceIndexStatus {
    pub indexed: bool,
    pub root: Option<String>,
    pub file_count: usize,
    pub files_skipped: usize,
}

/// Insert text directly at cursor position
/// Uses AppleScript keystroke for ASCII, clipboard paste for Unicode
fn insert_text_directly(text: &str) {
    #[cfg(target_os = "macos")]
    {
        // Normalize newlines to spaces - pressing Enter in chat apps sends the message,
        // which is not the intended behavior for dictation
        let normalized = text
            .replace("\r\n", " ")
            .replace("\r", " ")
            .replace("\n", " ");

        // Collapse multiple spaces into one
        let clean_text: String = normalized
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");

        // Check if text contains non-ASCII characters (Unicode)
        let has_unicode = clean_text.chars().any(|c| !c.is_ascii());

        if has_unicode {
            // For Unicode text (Hindi, Telugu, Tamil, etc.), use clipboard paste
            // AppleScript's keystroke command doesn't handle non-ASCII characters
            println!("Inserting Unicode text via clipboard paste...");
            insert_via_clipboard_preserving(&clean_text);
        } else {
            // For ASCII-only text, use keystroke (faster, no clipboard impact)
            println!("Inserting ASCII text via keystroke...");
            insert_via_keystroke(&clean_text);
        }
    }
}

/// Escape text for safe inclusion in AppleScript double-quoted strings.
/// Handles all characters that could break out of the string or cause injection.
///
/// SECURITY: This function prevents AppleScript injection by escaping:
/// - Backslashes (must be first to avoid double-escaping)
/// - Double quotes (could break out of string)
/// - Ampersands (AppleScript concatenation operator - could inject code)
/// - Carriage returns (removed)
/// - Tabs (converted to AppleScript tab concatenation)
#[cfg(target_os = "macos")]
fn escape_applescript_string(text: &str) -> String {
    text.replace("\\", "\\\\")       // Backslash must be first
        .replace("\"", "\\\"")        // Double quotes
        .replace("&", "\" & \"&\" & \"") // Escape ampersands to prevent injection
        .replace("\r", "")            // Remove carriage returns (handled separately)
        .replace("\t", "\" & tab & \"")  // Tabs as AppleScript concatenation
}

/// Insert ASCII text using AppleScript keystroke (doesn't touch clipboard)
#[cfg(target_os = "macos")]
fn insert_via_keystroke(text: &str) {
    use std::process::Command;

    // Escape text for AppleScript string using robust escaping
    let escaped_text = escape_applescript_string(text);

    // Use keystroke to type text directly
    let script = if text.contains('\n') {
        // Handle multi-line text by splitting and using return key
        let lines: Vec<&str> = text.split('\n').collect();
        let mut script_parts = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let escaped_line = escape_applescript_string(line);
            if !escaped_line.is_empty() {
                script_parts.push(format!("keystroke \"{}\"", escaped_line));
            }
            if i < lines.len() - 1 {
                script_parts.push("key code 36".to_string()); // Return key
            }
        }

        format!(
            r#"tell application "System Events"
    {}
end tell"#,
            script_parts.join("\n    ")
        )
    } else {
        format!(
            r#"tell application "System Events"
    keystroke "{}"
end tell"#,
            escaped_text
        )
    };

    let result = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output();

    match result {
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if output.status.success() && stderr.is_empty() {
                println!("Text inserted via keystroke (clipboard untouched)");
            } else if stderr.contains("not allowed") || stderr.contains("assistive") || stderr.contains("1002") {
                eprintln!("=======================================================");
                eprintln!("ACCESSIBILITY PERMISSION REQUIRED");
                eprintln!("Go to: System Settings > Privacy & Security > Accessibility");
                eprintln!("Add Murmur.app and ensure it's enabled");
                eprintln!("Then QUIT and RELAUNCH the app");
                eprintln!("=======================================================");
            } else if !stderr.is_empty() {
                eprintln!("osascript stderr: {}", stderr);
            }
        }
        Err(e) => {
            eprintln!("Failed to execute osascript: {}", e);
        }
    }
}

/// Insert text via clipboard, preserving the user's original clipboard contents
#[cfg(target_os = "macos")]
fn insert_via_clipboard_preserving(text: &str) {
    use std::process::Command;

    // Escape text for AppleScript string using robust escaping
    let escaped_text = escape_applescript_string(text);

    // AppleScript that:
    // 1. Saves current clipboard
    // 2. Sets clipboard to our text
    // 3. Pastes (Cmd+V)
    // 4. Restores original clipboard after a brief delay
    let script = format!(
        r#"
        -- Save original clipboard
        set originalClipboard to the clipboard

        -- Set clipboard to our text
        set the clipboard to "{}"

        -- Small delay to ensure clipboard is ready
        delay 0.05

        -- Paste using Cmd+V
        tell application "System Events"
            keystroke "v" using command down
        end tell

        -- Delay before restoring (give paste time to complete)
        delay 0.15

        -- Restore original clipboard
        try
            set the clipboard to originalClipboard
        end try
        "#,
        escaped_text
    );

    let result = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output();

    match result {
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if output.status.success() && stderr.is_empty() {
                println!("Unicode text inserted via clipboard (original clipboard restored)");
            } else if stderr.contains("not allowed") || stderr.contains("assistive") || stderr.contains("1002") {
                eprintln!("=======================================================");
                eprintln!("ACCESSIBILITY PERMISSION REQUIRED");
                eprintln!("Go to: System Settings > Privacy & Security > Accessibility");
                eprintln!("Add Murmur.app and ensure it's enabled");
                eprintln!("Then QUIT and RELAUNCH the app");
                eprintln!("=======================================================");
            } else if !stderr.is_empty() {
                eprintln!("osascript stderr: {}", stderr);
            }
        }
        Err(e) => {
            eprintln!("Failed to execute osascript: {}", e);
        }
    }
}

fn hide_overlay(app: &AppHandle) {
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.hide();
    }
}

/// Activate an application by its bundle ID using AppleScript.
/// Uses AppleScript instead of `open -b` to avoid launching new instances.
/// This is necessary because hiding a menu bar app's overlay doesn't automatically
/// return focus to the previously active application.
#[cfg(target_os = "macos")]
fn activate_app_by_bundle_id(bundle_id: &str) {
    use std::process::Command;

    // Don't try to activate ourselves - that could cause issues
    if bundle_id == "com.idstuart.murmur" {
        return;
    }

    // Use AppleScript to activate the app - this only activates existing instances,
    // unlike `open -b` which can launch new ones
    let script = format!(
        r#"tell application id "{}" to activate"#,
        bundle_id.replace("\"", "\\\"")
    );

    let result = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output();

    match result {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                #[cfg(debug_assertions)]
                eprintln!("Failed to activate app {}: {}", bundle_id, stderr);
            }
        }
        Err(e) => {
            #[cfg(debug_assertions)]
            eprintln!("Failed to run osascript: {}", e);
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn activate_app_by_bundle_id(_bundle_id: &str) {
    // No-op on non-macOS platforms
}

/// Position overlay window at center-bottom of the primary monitor with offset from bottom
fn position_overlay_center_bottom(overlay: &tauri::WebviewWindow, bottom_offset: i32) {
    use tauri::PhysicalPosition;

    // Get the primary monitor
    if let Ok(Some(monitor)) = overlay.primary_monitor() {
        let screen_size = monitor.size();
        let screen_position = monitor.position();

        // Get overlay window size
        if let Ok(overlay_size) = overlay.outer_size() {
            // Calculate center-bottom position
            let x = screen_position.x + ((screen_size.width as i32 - overlay_size.width as i32) / 2);
            let y = screen_position.y + (screen_size.height as i32 - overlay_size.height as i32 - bottom_offset);

            let _ = overlay.set_position(PhysicalPosition::new(x, y));
        }
    }
}

// ============================================================================
// TRAY AND SETUP
// ============================================================================

fn create_tray_menu(app: &AppHandle) -> Result<Menu<impl Runtime>, Box<dyn std::error::Error>> {
    let menu = Menu::new(app)?;

    let start_dictation =
        MenuItem::with_id(app, "start_dictation", "Start Dictation", true, None::<&str>)?;
    menu.append(&start_dictation)?;

    let separator = MenuItem::new(app, "-", false, None::<&str>)?;
    menu.append(&separator)?;

    let preferences = MenuItem::with_id(app, "preferences", "Preferences...", true, None::<&str>)?;
    menu.append(&preferences)?;

    let separator2 = MenuItem::new(app, "-", false, None::<&str>)?;
    menu.append(&separator2)?;

    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    menu.append(&quit)?;

    Ok(menu)
}

fn setup_system_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let menu = create_tray_menu(&app.handle())?;

    let icon = app
        .default_window_icon()
        .ok_or("Failed to load icon")?
        .clone();

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(true)
        .icon(icon)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => app.exit(0),
            "preferences" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "start_dictation" => {
                let _ = app.emit("toggle-recording", ());
            }
            _ => {}
        })
        .on_tray_icon_event(|_tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                // Could show overlay or menu on click
            }
        })
        .build(app)?;

    app.manage(_tray);
    Ok(())
}

/// Get the key code for a single key name (used by parse_hotkey)
fn get_key_code(key: &str) -> Option<tauri_plugin_global_shortcut::Code> {
    use tauri_plugin_global_shortcut::Code;

    // Letters a-z
    if key.len() == 1 {
        let c = key.chars().next()?;
        return match c {
            'a' => Some(Code::KeyA),
            'b' => Some(Code::KeyB),
            'c' => Some(Code::KeyC),
            'd' => Some(Code::KeyD),
            'e' => Some(Code::KeyE),
            'f' => Some(Code::KeyF),
            'g' => Some(Code::KeyG),
            'h' => Some(Code::KeyH),
            'i' => Some(Code::KeyI),
            'j' => Some(Code::KeyJ),
            'k' => Some(Code::KeyK),
            'l' => Some(Code::KeyL),
            'm' => Some(Code::KeyM),
            'n' => Some(Code::KeyN),
            'o' => Some(Code::KeyO),
            'p' => Some(Code::KeyP),
            'q' => Some(Code::KeyQ),
            'r' => Some(Code::KeyR),
            's' => Some(Code::KeyS),
            't' => Some(Code::KeyT),
            'u' => Some(Code::KeyU),
            'v' => Some(Code::KeyV),
            'w' => Some(Code::KeyW),
            'x' => Some(Code::KeyX),
            'y' => Some(Code::KeyY),
            'z' => Some(Code::KeyZ),
            '0' => Some(Code::Digit0),
            '1' => Some(Code::Digit1),
            '2' => Some(Code::Digit2),
            '3' => Some(Code::Digit3),
            '4' => Some(Code::Digit4),
            '5' => Some(Code::Digit5),
            '6' => Some(Code::Digit6),
            '7' => Some(Code::Digit7),
            '8' => Some(Code::Digit8),
            '9' => Some(Code::Digit9),
            _ => None,
        };
    }

    // Multi-character keys
    match key {
        "space" => Some(Code::Space),
        "f1" => Some(Code::F1),
        "f2" => Some(Code::F2),
        "f3" => Some(Code::F3),
        "f4" => Some(Code::F4),
        "f5" => Some(Code::F5),
        "f6" => Some(Code::F6),
        "f7" => Some(Code::F7),
        "f8" => Some(Code::F8),
        "f9" => Some(Code::F9),
        "f10" => Some(Code::F10),
        "f11" => Some(Code::F11),
        "f12" => Some(Code::F12),
        "escape" | "esc" => Some(Code::Escape),
        "enter" | "return" => Some(Code::Enter),
        "tab" => Some(Code::Tab),
        "backspace" => Some(Code::Backspace),
        "delete" => Some(Code::Delete),
        _ => None,
    }
}

/// Parse a hotkey string like "Cmd+Shift+D" or "Option+Space" into a Shortcut
fn parse_hotkey(hotkey: &str) -> Option<Shortcut> {
    let parts: Vec<&str> = hotkey.split('+').map(|s| s.trim()).collect();
    if parts.is_empty() {
        return None;
    }

    let mut modifiers = Modifiers::empty();
    let mut key_code = None;

    for part in parts {
        let lower = part.to_lowercase();
        match lower.as_str() {
            // Modifiers
            "cmd" | "command" | "super" | "meta" => modifiers |= Modifiers::SUPER,
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "alt" | "option" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            "fn" => {
                #[cfg(debug_assertions)]
                println!("Warning: Fn key is not supported in global shortcuts, ignoring");
            }
            // Key codes - use lookup function
            _ => {
                if let Some(code) = get_key_code(&lower) {
                    key_code = Some(code);
                } else {
                    #[cfg(debug_assertions)]
                    println!("Unknown key: {}", part);
                }
            }
        }
    }

    key_code.map(|code| Shortcut::new(Some(modifiers), code))
}

/// Internal function to start recording from shortcut
///
/// ARCHITECTURE: Instant response, async context capture
/// 1. INSTANT: Show overlay, start recording (user sees immediate feedback)
/// 2. ASYNC: Capture context (app detection, selection) while user speaks
/// 3. Context is ready by the time recording stops
fn shortcut_start_recording(app_handle: &AppHandle) {
    let state: tauri::State<'_, AppState> = app_handle.state();

    // Check if we can start
    let current_state = state.get_state();
    if !current_state.can_start_recording() {
        println!("Cannot start recording from state: {:?}", current_state);
        return;
    }

    // =========================================================================
    // INSTANT OVERLAY ARCHITECTURE
    // =========================================================================
    // The Accessibility API (get_selected_text) can take 500ms-1s+ depending
    // on the target app. To ensure instant overlay response:
    //
    // 1. Capture active app ONLY before overlay (fast: ~10-20ms via lsappinfo)
    // 2. Show overlay IMMEDIATELY (user sees instant feedback)
    // 3. Capture selection ASYNC in background thread
    // 4. Selection detection completes while user speaks (~1-5 seconds)
    // 5. Command Mode is ready by the time recording stops
    // =========================================================================

    let hotkey_start = std::time::Instant::now();

    // 1. Capture active app (fast: ~10-20ms via lsappinfo)
    let active_app_before_overlay = styles::detection::get_active_app();
    println!(
        "[TIMING] get_active_app: {:?} - bundle: {:?}",
        hotkey_start.elapsed(),
        active_app_before_overlay.as_ref().map(|a| &a.bundle_id)
    );

    // =========================================================================
    // PHASE 1: INSTANT RESPONSE - Show overlay NOW
    // =========================================================================

    // Default to Dictation mode (switches to Command when selection detected)
    state.set_state(RecordingState::Recording);
    state.set_mode(DictationMode::Dictation);
    state.set_recording_start(Some(std::time::Instant::now()));

    // Show overlay IMMEDIATELY - no blocking operations before this
    if let Some(overlay) = app_handle.get_webview_window("overlay") {
        position_overlay_center_bottom(&overlay, OVERLAY_BOTTOM_OFFSET);
        let _ = overlay.show();
    }
    #[cfg(debug_assertions)]
    println!("[TIMING] Hotkey-to-overlay: {:?}", hotkey_start.elapsed());

    // Emit initial state (Dictation mode)
    emit_state_change(app_handle, &state, Some("Recording...".to_string()));

    // =========================================================================
    // PHASE 2: ASYNC SELECTION DETECTION (while user speaks)
    // =========================================================================
    // Selection detection can take 500ms-1s+ but user speaks for 1-5+ seconds
    // So selection will be ready before recording stops

    let app_handle_for_selection = app_handle.clone();
    std::thread::spawn(move || {
        let state: tauri::State<'_, AppState> = app_handle_for_selection.state();

        // Only proceed if we're still recording
        if state.get_state() != RecordingState::Recording {
            return;
        }

        let selection_start = std::time::Instant::now();
        let selection = platform::selection::get_selected_text().ok();
        #[cfg(debug_assertions)]
        println!(
            "[TIMING] get_selected_text (async): {:?} - {} chars",
            selection_start.elapsed(),
            selection.as_ref().map(|s| s.len()).unwrap_or(0)
        );

        if let Some(text) = selection {
            // Check if still recording before switching mode
            if state.get_state() == RecordingState::Recording {
                state.set_mode(DictationMode::Command);
                state.set_selected_text(Some(text));
                emit_state_change(
                    &app_handle_for_selection,
                    &state,
                    Some("Command Mode".to_string()),
                );
                #[cfg(debug_assertions)]
                println!("[MODE] Switched to Command Mode");
            }
        }
    });

    // Start audio capture IMMEDIATELY
    let result = {
        let mut recorder = match state.recorder.lock() {
            Ok(r) => r,
            Err(e) => {
                println!("Failed to lock recorder: {}", e);
                state.set_state(RecordingState::Error);
                return;
            }
        };
        let selected_mic = permissions::get_selected_microphone_name();
        recorder.start_recording_with_device(app_handle.clone(), selected_mic)
    };

    if let Err(e) = result {
        state.set_state(RecordingState::Error);
        emit_error(app_handle, ErrorEvent::no_audio_device());
        println!("Failed to start recording: {}", e);
        return;
    }

    // =========================================================================
    // PHASE 2: ASYNC CONTEXT PROCESSING (happens while user speaks)
    // =========================================================================
    // Note: App and selection were already captured BEFORE overlay
    // This phase just processes context data that doesn't need to block

    let active_app_captured = active_app_before_overlay;
    let app_handle_for_context = app_handle.clone();

    std::thread::spawn(move || {
        let state: tauri::State<'_, AppState> = app_handle_for_context.state();

        // Only proceed if we're still recording
        if state.get_state() != RecordingState::Recording {
            return;
        }

        // Process the captured app context (already captured before overlay)
        let bundle_id = active_app_captured.as_ref().map(|a| a.bundle_id.clone());

        // Get style for the active app
        let active_style = match active_app_captured {
            Some(ref app) => styles::get_style_for_app(app),
            None => styles::get_default_style(),
        };

        let is_ide = bundle_id.as_ref().map(|b| ide::is_ide(b)).unwrap_or(false);
        #[cfg(debug_assertions)]
        println!(
            "Context processed (async): style={} ({}), bundle={:?}, is_ide={}",
            active_style.name, active_style.id, bundle_id, is_ide
        );

        // Store context in state for use during transcription processing
        state.set_active_style(Some(active_style));
        state.set_active_bundle_id(bundle_id);
    });
}

/// Internal function to stop recording from shortcut
fn shortcut_stop_recording(app_handle: AppHandle) {
    let app_handle_clone = app_handle.clone();

    // Spawn async task to handle the stop
    tauri::async_runtime::spawn(async move {
        let state: tauri::State<'_, AppState> = app_handle_clone.state();

        // Check if we can stop
        let current_state = state.get_state();
        if !current_state.can_stop_recording() {
            #[cfg(debug_assertions)]
            println!("Cannot stop recording from state: {:?}", current_state);
            return;
        }

        // Capture the bundle_id BEFORE processing clears it
        let bundle_id = state.get_active_bundle_id();

        // Use shared processing logic
        match process_recording_stop(&app_handle_clone, &state).await {
            Ok(final_text) => {
                // Hide overlay, reactivate previous app, then insert text
                let app_for_hide = app_handle_clone.clone();
                std::thread::spawn(move || {
                    // Brief delay to show "Done!" state
                    std::thread::sleep(std::time::Duration::from_millis(DONE_DISPLAY_DELAY_MS));
                    hide_overlay(&app_for_hide);
                    // Reactivate the previous app explicitly
                    if let Some(bid) = bundle_id {
                        activate_app_by_bundle_id(&bid);
                    }
                    // Wait for the app to regain focus
                    std::thread::sleep(std::time::Duration::from_millis(APP_FOCUS_WAIT_MS));
                    // Insert text (this replaces selection in Command Mode, inserts at cursor in Dictation Mode)
                    insert_text_directly(&final_text);
                });
            }
            Err(e) => {
                #[cfg(debug_assertions)]
                eprintln!("Recording stop failed: {}", e);
                // Error state and overlay hiding already handled in process_recording_stop
            }
        }
    });
}

/// Internal function to toggle recording from shortcut
fn shortcut_toggle_recording(app_handle: &AppHandle) {
    let state: tauri::State<'_, AppState> = app_handle.state();
    let current_state = state.get_state();

    match current_state {
        RecordingState::Idle | RecordingState::Error => {
            shortcut_start_recording(app_handle);
        }
        RecordingState::Recording => {
            shortcut_stop_recording(app_handle.clone());
        }
        _ => {
            // Transcribing or Enhancing - can't toggle
        }
    }
}

fn setup_global_shortcuts(
    app: &App,
    hotkey: &str,
    _mode: &str, // Not used anymore - we read dynamically from config
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse the hotkey string
    let shortcut = match parse_hotkey(hotkey) {
        Some(s) => s,
        None => {
            // Default to Option+Space if parsing fails
            println!("Failed to parse hotkey '{}', using default Option+Space", hotkey);
            Shortcut::new(Some(Modifiers::ALT), tauri_plugin_global_shortcut::Code::Space)
        }
    };

    println!("Registering global shortcut: {:?}", shortcut);

    // Register the shortcut with key state handling
    // IMPORTANT: Read recording_mode dynamically from config each time,
    // so changes in preferences take effect immediately
    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, _shortcut, event| {
                // Read current mode from config (not captured at startup)
                let is_push_to_talk = {
                    let state: tauri::State<'_, AppState> = app.state();
                    let result = match state.config.lock() {
                        Ok(config) => config.recording_mode == "push-to-talk",
                        Err(_) => false, // Default to toggle on error
                    };
                    result
                };

                match event.state {
                    ShortcutState::Pressed => {
                        println!("Shortcut pressed, mode: {}", if is_push_to_talk { "push-to-talk" } else { "toggle" });
                        if is_push_to_talk {
                            // Push-to-talk: start recording on press
                            shortcut_start_recording(app);
                        } else {
                            // Toggle mode: toggle on press
                            shortcut_toggle_recording(app);
                        }
                    }
                    ShortcutState::Released => {
                        if is_push_to_talk {
                            // Push-to-talk: stop recording on release
                            println!("Shortcut released (push-to-talk)");
                            shortcut_stop_recording(app.clone());
                        }
                        // Toggle mode: do nothing on release
                    }
                }
            })
            .build(),
    )?;

    // Register the specific shortcut
    app.global_shortcut().register(shortcut)?;

    println!("Global shortcut registered successfully");

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = AppConfig::load();
    let initial_hotkey = config.hotkey.clone();
    let initial_mode = config.recording_mode.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Another instance tried to launch - bring existing app to focus
            println!("Another instance attempted to start, bringing existing window to focus");
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new(config))
        .invoke_handler(tauri::generate_handler![
            get_recording_state,
            get_overlay_state,
            start_recording,
            stop_recording,
            cancel_recording,
            toggle_recording,
            show_preferences,
            update_preferences,
            get_preferences,
            check_permissions,
            get_microphones,
            request_microphone_permission,
            open_accessibility_settings,
            set_selected_microphone,
            is_onboarding_complete,
            complete_onboarding,
            // Workspace index commands
            set_workspace_root,
            get_workspace_status,
            clear_workspace_index,
        ])
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            setup_system_tray(app)?;

            // Register global shortcut
            setup_global_shortcuts(app, &initial_hotkey, &initial_mode)?;

            // NOTE: Workspace auto-indexing disabled - use set_workspace_root command if needed.

            // Hide main window on start
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }

            // Pre-position overlay at bottom-center (while hidden)
            // This prevents flash when first shown
            if let Some(overlay) = app.get_webview_window("overlay") {
                position_overlay_center_bottom(&overlay, OVERLAY_BOTTOM_OFFSET);
                let _ = overlay.hide();
            }

            // Check if onboarding is needed
            if !permissions::is_onboarding_complete() {
                // Show onboarding window
                if let Some(onboarding) = app.get_webview_window("onboarding") {
                    let _ = onboarding.show();
                    let _ = onboarding.set_focus();
                    // Change activation policy to regular app during onboarding
                    #[cfg(target_os = "macos")]
                    app.set_activation_policy(tauri::ActivationPolicy::Regular);
                }
            } else {
                // Hide onboarding window if it exists
                if let Some(onboarding) = app.get_webview_window("onboarding") {
                    let _ = onboarding.hide();
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    // AppleScript tests - only run on macOS where the function is available
    #[cfg(target_os = "macos")]
    mod applescript_tests {
        use super::*;

        #[test]
        fn test_escape_applescript_string_basic() {
            assert_eq!(escape_applescript_string("hello world"), "hello world");
        }

        #[test]
        fn test_escape_applescript_string_quotes() {
            assert_eq!(escape_applescript_string(r#"say "hello""#), r#"say \"hello\""#);
        }

        #[test]
        fn test_escape_applescript_string_backslash() {
            assert_eq!(escape_applescript_string(r"path\to\file"), r"path\\to\\file");
        }

        #[test]
        fn test_escape_applescript_string_tabs() {
            assert_eq!(escape_applescript_string("col1\tcol2"), r#"col1" & tab & "col2"#);
        }

        #[test]
        fn test_escape_applescript_string_carriage_return() {
            assert_eq!(escape_applescript_string("line1\r\nline2"), "line1\nline2");
        }

        #[test]
        fn test_escape_applescript_string_complex() {
            let input = r#"She said "hello\" and left"#;
            let expected = r#"She said \"hello\\\" and left"#;
            assert_eq!(escape_applescript_string(input), expected);
        }

        #[test]
        fn test_escape_applescript_string_empty() {
            assert_eq!(escape_applescript_string(""), "");
        }

        #[test]
        fn test_escape_applescript_string_unicode() {
            assert_eq!(escape_applescript_string("Hello 世界 🌍"), "Hello 世界 🌍");
        }
    }

    #[test]
    fn test_validate_workspace_path_home_subdir() {
        // A directory under home should be valid
        if let Some(home) = dirs::home_dir() {
            let test_path = home.join("Projects");
            if test_path.exists() {
                let result = validate_workspace_path(&test_path);
                assert!(result.is_ok(), "Valid home subdir should be accepted");
            }
        }
    }

    #[test]
    fn test_validate_workspace_path_blocks_ssh() {
        // .ssh should be blocked
        if let Some(home) = dirs::home_dir() {
            let ssh_path = home.join(".ssh");
            if ssh_path.exists() {
                let result = validate_workspace_path(&ssh_path);
                assert!(result.is_err(), ".ssh should be blocked");
                assert!(result.unwrap_err().contains("sensitive directory"));
            }
        }
    }

    #[test]
    fn test_validate_workspace_path_blocks_system() {
        // System directories should be blocked
        let system_path = std::path::Path::new("/usr");
        if system_path.exists() {
            let result = validate_workspace_path(system_path);
            assert!(result.is_err(), "System directory should be blocked");
            assert!(result.unwrap_err().contains("home directory"));
        }
    }

    #[test]
    fn test_blocked_directories_list() {
        // Verify blocked directories list includes critical paths
        assert!(BLOCKED_DIRECTORIES.contains(&".ssh"));
        assert!(BLOCKED_DIRECTORIES.contains(&".gnupg"));
        assert!(BLOCKED_DIRECTORIES.contains(&".aws"));
    }
}
