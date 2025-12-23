use std::sync::Mutex;
use std::time::Instant;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Emitter, Manager, Runtime, State, WindowEvent,
};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

mod audio;
mod claude;
mod config;
mod deepgram;
mod groq_llm;
mod ide;
mod licensing;
mod model_manager;
mod permissions;
mod platform;
mod state;
mod styles;
mod transcription;
mod whisper_api;
mod whisper_local;

use audio::AudioRecorder;
use claude::ClaudeClient;
use config::{AppConfig, TranscriptionProvider};
use deepgram::DeepgramClient;
use groq_llm::{GroqLlmClient, UserIntent};
use state::{DictationMode, ErrorEvent, RecordingState, StateChangeEvent, TranscriptionCompleteEvent};

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
    if let Ok(mut start) = state.recording_start.lock() {
        *start = Some(Instant::now());
    }

    // Show overlay IMMEDIATELY - no delay
    if let Some(overlay) = app_handle.get_webview_window("overlay") {
        position_overlay_center_bottom(&overlay, 300);
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
        recorder.start_recording(app_handle.clone())
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

    // Update state to transcribing
    state.set_state(RecordingState::Transcribing);
    emit_state_change(
        &app_handle,
        &state,
        Some("Processing audio...".to_string()),
    );

    // Get configuration first to determine which audio format we need
    let (provider, deepgram_key, groq_key, anthropic_key, claude_model, language, spoken_languages) = {
        let config = state
            .config
            .lock()
            .map_err(|e| format!("Failed to lock config: {}", e))?;

        // Load spoken languages from stored preferences
        let stored = config::StoredPreferences::load();
        let spoken_langs = stored.spoken_languages.unwrap_or_else(|| vec!["en".to_string()]);

        (
            config.transcription_provider.clone(),
            config.deepgram_api_key.clone(),
            config.groq_api_key.clone(),
            config.anthropic_api_key.clone(),
            config.claude_model.clone(),
            config.language.clone(),
            spoken_langs,
        )
    };

    // Stop recording and get audio data in the appropriate format
    let (audio_wav, audio_samples_16khz) = {
        let mut recorder = state
            .recorder
            .lock()
            .map_err(|e| format!("Failed to lock recorder: {}", e))?;

        match provider {
            TranscriptionProvider::Deepgram => {
                // Deepgram uses WAV format
                let wav = recorder.stop_recording()?;
                (wav, Vec::new())
            }
            TranscriptionProvider::WhisperApi | TranscriptionProvider::WhisperLocal => {
                // Whisper providers need 16kHz f32 samples AND WAV for API
                // First get the samples for Whisper
                let samples = recorder.stop_recording_for_whisper()?;
                // We also need WAV for Whisper API (it accepts WAV over HTTP)
                // Re-create WAV from the original samples
                let wav = Vec::new(); // Will re-fetch if needed
                (wav, samples)
            }
        }
    };

    // Check if we have audio
    let has_audio = match provider {
        TranscriptionProvider::Deepgram => !audio_wav.is_empty(),
        _ => !audio_samples_16khz.is_empty(),
    };

    if !has_audio {
        state.set_state(RecordingState::Error);
        emit_error(&app_handle, ErrorEvent::no_audio_captured());
        return Err("No audio captured".to_string());
    }

    // Transcribe using the configured provider
    emit_state_change(
        &app_handle,
        &state,
        Some("Transcribing...".to_string()),
    );

    let transcript = match &provider {
        TranscriptionProvider::Deepgram => {
            let api_key = deepgram_key.ok_or("Deepgram API key not configured")?;
            let client = DeepgramClient::new(api_key, Some(language.clone()));
            client.transcribe_audio(audio_wav).await.map_err(|e| {
                state.set_state(RecordingState::Error);
                emit_error(&app_handle, ErrorEvent::deepgram_error(&e));
                hide_overlay(&app_handle);
                e
            })?
        }
        TranscriptionProvider::WhisperApi => {
            // For Whisper API, we need to re-encode the samples as WAV
            let wav = encode_samples_to_wav(&audio_samples_16khz, 16000)?;
            // WhisperApiClient uses groq_api_key from config or GROQ_API_KEY env var
            let client = whisper_api::WhisperApiClient::new(groq_key.unwrap_or_default());
            client.transcribe(&wav, &language, &spoken_languages).await.map_err(|e| {
                state.set_state(RecordingState::Error);
                emit_error(&app_handle, ErrorEvent::whisper_error(&e));
                hide_overlay(&app_handle);
                e
            })?
        }
        TranscriptionProvider::WhisperLocal => {
            // Check if model is downloaded
            if !model_manager::is_model_downloaded() {
                state.set_state(RecordingState::Error);
                emit_error(&app_handle, ErrorEvent::model_not_loaded());
                hide_overlay(&app_handle);
                return Err("Whisper model not downloaded".to_string());
            }
            whisper_local::WhisperLocalClient::transcribe(&audio_samples_16khz, &language)
                .await
                .map_err(|e| {
                    state.set_state(RecordingState::Error);
                    emit_error(&app_handle, ErrorEvent::whisper_error(&e));
                    hide_overlay(&app_handle);
                    e
                })?
        }
    };

    if transcript.is_empty() {
        state.set_state(RecordingState::Error);
        emit_error(&app_handle, ErrorEvent::no_audio_captured());
        hide_overlay(&app_handle);
        return Err("No speech detected".to_string());
    }

    println!("Transcript ({}): {}", provider.to_string(), transcript);

    // Update state to enhancing
    state.set_state(RecordingState::Enhancing);
    emit_state_change(&app_handle, &state, Some("Enhancing...".to_string()));

    // Enhance with Claude (if API key is available)
    let enhanced_text = if let Some(api_key) = anthropic_key {
        let claude_client = ClaudeClient::new(api_key, Some(claude_model));
        match claude_client.enhance_text(&transcript, None).await {
            Ok(t) => t,
            Err(e) => {
                // Fallback to raw transcript
                println!("Claude enhancement failed, using raw transcript: {}", e);
                emit_error(
                    &app_handle,
                    ErrorEvent::claude_error(&e, Some(transcript.clone())),
                );
                transcript.clone()
            }
        }
    } else {
        // No Claude API key, use raw transcript
        transcript.clone()
    };

    println!("Enhanced: {}", enhanced_text);

    // Emit completion (text will be inserted, not just copied)
    let completion_event = TranscriptionCompleteEvent {
        raw_transcript: transcript,
        enhanced_text: enhanced_text.clone(),
        copied_to_clipboard: false, // We're inserting directly now
    };

    if let Err(e) = app_handle.emit("transcription-complete", &completion_event) {
        eprintln!("Failed to emit completion: {}", e);
    }

    // Return to idle
    state.set_state(RecordingState::Idle);
    if let Ok(mut start) = state.recording_start.lock() {
        *start = None;
    }
    emit_state_change(&app_handle, &state, Some("Done!".to_string()));

    // Hide overlay, wait for previous app to regain focus, then insert text
    let app_clone = app_handle.clone();
    let text_to_insert = enhanced_text.clone();
    std::thread::spawn(move || {
        // Brief delay to show "Done!" state
        std::thread::sleep(std::time::Duration::from_millis(300));
        hide_overlay(&app_clone);
        // Wait for the previous app to regain focus
        std::thread::sleep(std::time::Duration::from_millis(100));
        // Insert text while preserving clipboard
        insert_text_directly(&text_to_insert);
    });

    Ok(())
}

/// Helper to encode f32 samples to WAV format
fn encode_samples_to_wav(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>, String> {
    let mut buffer = Vec::new();

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    {
        let mut writer = hound::WavWriter::new(std::io::Cursor::new(&mut buffer), spec)
            .map_err(|e| format!("Failed to create WAV writer: {}", e))?;

        for &sample in samples {
            let amplitude = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            writer
                .write_sample(amplitude)
                .map_err(|e| format!("Failed to write sample: {}", e))?;
        }

        writer
            .finalize()
            .map_err(|e| format!("Failed to finalize WAV: {}", e))?;
    }

    Ok(buffer)
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
    if let Ok(mut start) = state.recording_start.lock() {
        *start = None;
    }
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
        deepgram_key: config.deepgram_api_key.clone().unwrap_or_default(),
        anthropic_key: config.anthropic_api_key.clone().unwrap_or_default(),
        // New: Transcription provider settings
        transcription_provider: Some(config.transcription_provider.to_string()),
        license_key: config.license_key.clone(),
        // Language onboarding - stored in preferences (separate from permission onboarding)
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
    permissions::set_selected_microphone(&device_id)
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
// MODEL AND LICENSE MANAGEMENT COMMANDS
// ============================================================================

/// Get the status of the Whisper model
#[tauri::command]
fn get_model_status() -> model_manager::ModelStatus {
    model_manager::get_model_status()
}

/// Download the Whisper model
#[tauri::command]
async fn download_model(app_handle: AppHandle) -> Result<String, String> {
    let path = model_manager::download_model(&app_handle).await?;
    Ok(path.to_string_lossy().to_string())
}

/// Delete the downloaded Whisper model
#[tauri::command]
fn delete_model() -> Result<(), String> {
    model_manager::delete_model()
}

/// Validate a license key
#[tauri::command]
async fn validate_license(license_key: String) -> Result<licensing::LicenseInfo, String> {
    licensing::validate_license(&license_key).await
}

/// Activate a license key
#[tauri::command]
async fn activate_license(
    state: State<'_, AppState>,
    license_key: String,
) -> Result<licensing::LicenseInfo, String> {
    let info = licensing::activate_license(&license_key).await?;

    // Update config with the license key if valid
    if info.valid {
        let mut config = state
            .config
            .lock()
            .map_err(|e| format!("Failed to lock config: {}", e))?;

        config.license_key = Some(license_key);

        // Auto-select provider based on license tier
        match info.tier {
            licensing::LicenseTier::Subscription => {
                config.transcription_provider = TranscriptionProvider::WhisperApi;
            }
            licensing::LicenseTier::Lifetime => {
                config.transcription_provider = TranscriptionProvider::WhisperLocal;
            }
            _ => {}
        }
    }

    Ok(info)
}

/// Get cached license info
#[tauri::command]
fn get_license_info() -> licensing::LicenseInfo {
    licensing::get_cached_license()
}

/// Clear/deactivate license
#[tauri::command]
fn clear_license() -> Result<(), String> {
    licensing::clear_license()
}

/// Get current transcription provider
#[tauri::command]
fn get_transcription_provider(state: State<'_, AppState>) -> Result<String, String> {
    let config = state
        .config
        .lock()
        .map_err(|e| format!("Failed to lock config: {}", e))?;

    Ok(config.transcription_provider.to_string())
}

/// Set transcription provider
#[tauri::command]
fn set_transcription_provider(
    state: State<'_, AppState>,
    provider: String,
) -> Result<(), String> {
    let mut config = state
        .config
        .lock()
        .map_err(|e| format!("Failed to lock config: {}", e))?;

    config.transcription_provider = TranscriptionProvider::from_string(&provider);
    println!("Transcription provider set to: {:?}", config.transcription_provider);

    Ok(())
}

// ============================================================================
// WORKSPACE INDEX COMMANDS
// ============================================================================

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

    println!("[WORKSPACE] Building index for: {}", path);

    match ide::file_index::WorkspaceIndex::build(workspace_path) {
        Ok(index) => {
            let file_count = index.file_count();
            let files_skipped = index.files_skipped;
            state.set_workspace_index(Some(index));
            println!("[WORKSPACE] Index built: {} files indexed, {} skipped", file_count, files_skipped);
            Ok(WorkspaceIndexStatus {
                indexed: true,
                root: Some(path),
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

/// Try to auto-detect a workspace root from the current directory.
/// Walks up from the current directory looking for project markers.
fn auto_detect_workspace() -> Option<std::path::PathBuf> {
    use std::path::Path;

    // Project markers that indicate a workspace root
    const PROJECT_MARKERS: &[&str] = &[
        ".git",
        "package.json",
        "Cargo.toml",
        "pyproject.toml",
        "go.mod",
        "pom.xml",
        "build.gradle",
        ".project",
    ];

    // Start from current directory
    let mut current = std::env::current_dir().ok()?;

    // Walk up looking for project markers (max 10 levels)
    for _ in 0..10 {
        for marker in PROJECT_MARKERS {
            if current.join(marker).exists() {
                println!("[WORKSPACE] Auto-detected project root: {} (found {})",
                    current.display(), marker);
                return Some(current);
            }
        }

        // Move up one directory
        if !current.pop() {
            break;
        }
    }

    None
}

/// Build workspace index from auto-detected or configured path
fn initialize_workspace_index(state: &AppState) {
    // Try to auto-detect workspace
    if let Some(workspace_path) = auto_detect_workspace() {
        match ide::file_index::WorkspaceIndex::build(&workspace_path) {
            Ok(index) => {
                let file_count = index.file_count();
                println!("[WORKSPACE] Auto-indexed {} files from {}",
                    file_count, workspace_path.display());
                state.set_workspace_index(Some(index));
            }
            Err(e) => {
                println!("[WORKSPACE] Failed to build auto-detected index: {}", e);
            }
        }
    } else {
        println!("[WORKSPACE] No workspace auto-detected - file tagging disabled until workspace is set");
    }
}

/// Insert text directly at cursor position
/// Uses AppleScript keystroke for ASCII, clipboard paste for Unicode
fn insert_text_directly(text: &str) {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        // Check if text contains non-ASCII characters (Unicode)
        let has_unicode = text.chars().any(|c| !c.is_ascii());

        if has_unicode {
            // For Unicode text (Hindi, Telugu, Tamil, etc.), use clipboard paste
            // AppleScript's keystroke command doesn't handle non-ASCII characters
            println!("Inserting Unicode text via clipboard paste...");
            insert_via_clipboard_preserving(text);
        } else {
            // For ASCII-only text, use keystroke (faster, no clipboard impact)
            println!("Inserting ASCII text via keystroke...");
            insert_via_keystroke(text);
        }
    }
}

/// Insert ASCII text using AppleScript keystroke (doesn't touch clipboard)
#[cfg(target_os = "macos")]
fn insert_via_keystroke(text: &str) {
    use std::process::Command;

    // Escape text for AppleScript string
    let escaped_text = text
        .replace("\\", "\\\\")
        .replace("\"", "\\\"");

    // Use keystroke to type text directly
    let script = if text.contains('\n') {
        // Handle multi-line text by splitting and using return key
        let lines: Vec<&str> = text.split('\n').collect();
        let mut script_parts = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let escaped_line = line.replace("\\", "\\\\").replace("\"", "\\\"");
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

    // Escape text for AppleScript string (need to escape backslashes and quotes)
    let escaped_text = text
        .replace("\\", "\\\\")
        .replace("\"", "\\\"");

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

/// Parse a hotkey string like "Cmd+Shift+D" or "Option+Space" into a Shortcut
fn parse_hotkey(hotkey: &str) -> Option<Shortcut> {
    use tauri_plugin_global_shortcut::Code;

    let parts: Vec<&str> = hotkey.split('+').map(|s| s.trim()).collect();
    if parts.is_empty() {
        return None;
    }

    let mut modifiers = Modifiers::empty();
    let mut key_code = None;

    for part in parts {
        match part.to_lowercase().as_str() {
            "cmd" | "command" | "super" | "meta" => modifiers |= Modifiers::SUPER,
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "alt" | "option" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            "fn" => {
                // Fn key is not supported as a modifier in global shortcuts
                // We'll ignore it and use the other parts
                println!("Warning: Fn key is not supported in global shortcuts, ignoring");
            }
            // Key codes
            "space" => key_code = Some(Code::Space),
            "a" => key_code = Some(Code::KeyA),
            "b" => key_code = Some(Code::KeyB),
            "c" => key_code = Some(Code::KeyC),
            "d" => key_code = Some(Code::KeyD),
            "e" => key_code = Some(Code::KeyE),
            "f" => key_code = Some(Code::KeyF),
            "g" => key_code = Some(Code::KeyG),
            "h" => key_code = Some(Code::KeyH),
            "i" => key_code = Some(Code::KeyI),
            "j" => key_code = Some(Code::KeyJ),
            "k" => key_code = Some(Code::KeyK),
            "l" => key_code = Some(Code::KeyL),
            "m" => key_code = Some(Code::KeyM),
            "n" => key_code = Some(Code::KeyN),
            "o" => key_code = Some(Code::KeyO),
            "p" => key_code = Some(Code::KeyP),
            "q" => key_code = Some(Code::KeyQ),
            "r" => key_code = Some(Code::KeyR),
            "s" => key_code = Some(Code::KeyS),
            "t" => key_code = Some(Code::KeyT),
            "u" => key_code = Some(Code::KeyU),
            "v" => key_code = Some(Code::KeyV),
            "w" => key_code = Some(Code::KeyW),
            "x" => key_code = Some(Code::KeyX),
            "y" => key_code = Some(Code::KeyY),
            "z" => key_code = Some(Code::KeyZ),
            "1" => key_code = Some(Code::Digit1),
            "2" => key_code = Some(Code::Digit2),
            "3" => key_code = Some(Code::Digit3),
            "4" => key_code = Some(Code::Digit4),
            "5" => key_code = Some(Code::Digit5),
            "6" => key_code = Some(Code::Digit6),
            "7" => key_code = Some(Code::Digit7),
            "8" => key_code = Some(Code::Digit8),
            "9" => key_code = Some(Code::Digit9),
            "0" => key_code = Some(Code::Digit0),
            "f1" => key_code = Some(Code::F1),
            "f2" => key_code = Some(Code::F2),
            "f3" => key_code = Some(Code::F3),
            "f4" => key_code = Some(Code::F4),
            "f5" => key_code = Some(Code::F5),
            "f6" => key_code = Some(Code::F6),
            "f7" => key_code = Some(Code::F7),
            "f8" => key_code = Some(Code::F8),
            "f9" => key_code = Some(Code::F9),
            "f10" => key_code = Some(Code::F10),
            "f11" => key_code = Some(Code::F11),
            "f12" => key_code = Some(Code::F12),
            "escape" | "esc" => key_code = Some(Code::Escape),
            "enter" | "return" => key_code = Some(Code::Enter),
            "tab" => key_code = Some(Code::Tab),
            "backspace" => key_code = Some(Code::Backspace),
            "delete" => key_code = Some(Code::Delete),
            _ => {
                println!("Unknown key: {}", part);
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

    // Capture context BEFORE showing overlay (while original app has focus)
    // These are quick calls (~10-30ms total) that must happen before overlay steals focus

    // 1. Capture active app
    let active_app_before_overlay = styles::detection::get_active_app();
    println!(
        "Active app captured: {:?}",
        active_app_before_overlay.as_ref().map(|a| &a.bundle_id)
    );

    // 2. Capture selection (for Command Mode detection)
    let selection_before_overlay = platform::selection::get_selected_text().ok();
    let has_selection = selection_before_overlay.is_some();
    println!(
        "Selection captured: {} chars",
        selection_before_overlay.as_ref().map(|s| s.len()).unwrap_or(0)
    );

    // =========================================================================
    // PHASE 1: INSTANT RESPONSE (no blocking operations)
    // =========================================================================

    // Set state to Recording immediately
    // Mode is set based on whether we captured a selection
    state.set_state(RecordingState::Recording);
    if has_selection {
        state.set_mode(DictationMode::Command);
        state.set_selected_text(selection_before_overlay.clone());
    } else {
        state.set_mode(DictationMode::Dictation);
    }
    if let Ok(mut start) = state.recording_start.lock() {
        *start = Some(std::time::Instant::now());
    }

    // Show overlay IMMEDIATELY - no delay, no blocking calls before this
    if let Some(overlay) = app_handle.get_webview_window("overlay") {
        position_overlay_center_bottom(&overlay, 300);
        let _ = overlay.show();
    }

    // Emit initial state with correct mode
    let initial_status = if has_selection {
        "Command Mode".to_string()
    } else {
        "Recording...".to_string()
    };
    emit_state_change(app_handle, &state, Some(initial_status));

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
        recorder.start_recording(app_handle.clone())
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
            println!("Cannot stop recording from state: {:?}", current_state);
            return;
        }

        // Update state to transcribing
        state.set_state(RecordingState::Transcribing);
        emit_state_change(
            &app_handle_clone,
            &state,
            Some("Processing audio...".to_string()),
        );

        // Get configuration first to determine which audio format we need
        let (provider, deepgram_key, groq_key, anthropic_key, claude_model, language, spoken_languages) = {
            let config = match state.config.lock() {
                Ok(c) => c,
                Err(e) => {
                    println!("Failed to lock config: {}", e);
                    state.set_state(RecordingState::Error);
                    hide_overlay(&app_handle_clone);
                    return;
                }
            };

            // Load spoken languages from stored preferences
            let stored = config::StoredPreferences::load();
            let spoken_langs = stored.spoken_languages.unwrap_or_else(|| vec!["en".to_string()]);

            (
                config.transcription_provider.clone(),
                config.deepgram_api_key.clone(),
                config.groq_api_key.clone(),
                config.anthropic_api_key.clone(),
                config.claude_model.clone(),
                config.language.clone(),
                spoken_langs,
            )
        };

        // Stop recording and get audio data in the appropriate format
        let (audio_wav, audio_samples_16khz) = {
            let mut recorder = match state.recorder.lock() {
                Ok(r) => r,
                Err(e) => {
                    println!("Failed to lock recorder: {}", e);
                    state.set_state(RecordingState::Error);
                    return;
                }
            };

            match provider {
                TranscriptionProvider::Deepgram => {
                    match recorder.stop_recording() {
                        Ok(wav) => (wav, Vec::new()),
                        Err(e) => {
                            println!("Failed to stop recording: {}", e);
                            state.set_state(RecordingState::Error);
                            hide_overlay(&app_handle_clone);
                            return;
                        }
                    }
                }
                TranscriptionProvider::WhisperApi | TranscriptionProvider::WhisperLocal => {
                    match recorder.stop_recording_for_whisper() {
                        Ok(samples) => (Vec::new(), samples),
                        Err(e) => {
                            println!("Failed to stop recording: {}", e);
                            state.set_state(RecordingState::Error);
                            hide_overlay(&app_handle_clone);
                            return;
                        }
                    }
                }
            }
        };

        // Check if we have audio
        let has_audio = match provider {
            TranscriptionProvider::Deepgram => !audio_wav.is_empty(),
            _ => !audio_samples_16khz.is_empty(),
        };

        if !has_audio {
            state.set_state(RecordingState::Error);
            emit_error(&app_handle_clone, ErrorEvent::no_audio_captured());
            hide_overlay(&app_handle_clone);
            return;
        }

        // Transcribe using the configured provider
        emit_state_change(
            &app_handle_clone,
            &state,
            Some("Transcribing...".to_string()),
        );

        let transcript = match &provider {
            TranscriptionProvider::Deepgram => {
                let api_key = match deepgram_key {
                    Some(k) => k,
                    None => {
                        println!("Deepgram API key not configured");
                        state.set_state(RecordingState::Error);
                        emit_error(&app_handle_clone, ErrorEvent::no_transcription_provider());
                        hide_overlay(&app_handle_clone);
                        return;
                    }
                };
                let client = DeepgramClient::new(api_key, Some(language.clone()));
                match client.transcribe_audio(audio_wav).await {
                    Ok(t) => t,
                    Err(e) => {
                        state.set_state(RecordingState::Error);
                        emit_error(&app_handle_clone, ErrorEvent::deepgram_error(&e));
                        hide_overlay(&app_handle_clone);
                        return;
                    }
                }
            }
            TranscriptionProvider::WhisperApi => {
                let wav = match encode_samples_to_wav(&audio_samples_16khz, 16000) {
                    Ok(w) => w,
                    Err(e) => {
                        println!("Failed to encode WAV: {}", e);
                        state.set_state(RecordingState::Error);
                        hide_overlay(&app_handle_clone);
                        return;
                    }
                };
                // WhisperApiClient uses groq_api_key from config or GROQ_API_KEY env var
                let client = whisper_api::WhisperApiClient::new(groq_key.clone().unwrap_or_default());
                match client.transcribe(&wav, &language, &spoken_languages).await {
                    Ok(t) => t,
                    Err(e) => {
                        state.set_state(RecordingState::Error);
                        emit_error(&app_handle_clone, ErrorEvent::whisper_error(&e));
                        hide_overlay(&app_handle_clone);
                        return;
                    }
                }
            }
            TranscriptionProvider::WhisperLocal => {
                if !model_manager::is_model_downloaded() {
                    println!("Whisper model not downloaded");
                    state.set_state(RecordingState::Error);
                    emit_error(&app_handle_clone, ErrorEvent::model_not_loaded());
                    hide_overlay(&app_handle_clone);
                    return;
                }
                match whisper_local::WhisperLocalClient::transcribe(&audio_samples_16khz, &language).await {
                    Ok(t) => t,
                    Err(e) => {
                        state.set_state(RecordingState::Error);
                        emit_error(&app_handle_clone, ErrorEvent::whisper_error(&e));
                        hide_overlay(&app_handle_clone);
                        return;
                    }
                }
            }
        };

        if transcript.is_empty() {
            state.set_state(RecordingState::Error);
            emit_error(&app_handle_clone, ErrorEvent::no_audio_captured());
            hide_overlay(&app_handle_clone);
            return;
        }

        println!("Transcript ({}): {}", provider.to_string(), transcript);

        // Apply IDE transformations if we're in a code editor
        let active_bundle_id = state.get_active_bundle_id();
        let workspace_index = state.get_workspace_index();
        let transcript = if let Some(ref bundle_id) = active_bundle_id {
            if ide::is_ide(bundle_id) {
                let ide_context = ide::get_ide_context(bundle_id);
                let ide_settings = ide::IDESettings::default();

                // Pass workspace index if available - file tagging only works with an index
                let transformed = ide::apply_ide_transformations(
                    &transcript,
                    &ide_context,
                    &ide_settings,
                    workspace_index.as_ref(),
                );

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

        // Get the current mode, selected text, and active style
        let current_mode = state.get_mode();
        let selected_text_for_transform = state.get_selected_text();
        let active_style = state.get_active_style();

        // Process based on mode
        let final_text = match current_mode {
            DictationMode::Command => {
                // Selection detected - but is this a command or new content?
                // Use LLM to classify intent before deciding how to process

                let selected_text = match selected_text_for_transform {
                    Some(text) => text,
                    None => {
                        // This shouldn't happen, but fall back to dictation
                        println!("Command Mode but no selected text, falling back to raw transcript");
                        transcript.clone()
                    }
                };

                let groq_client = GroqLlmClient::new(groq_key.clone().unwrap_or_default());

                // First, classify intent: is this a command or new content?
                state.set_state(RecordingState::Transforming);
                emit_state_change(&app_handle_clone, &state, Some("Analyzing...".to_string()));

                let intent = match groq_client.classify_intent(&transcript).await {
                    Ok(i) => i,
                    Err(e) => {
                        println!("Intent classification failed, defaulting to Dictation: {}", e);
                        UserIntent::Dictation // Default to dictation on error - safer UX
                    }
                };

                match intent {
                    UserIntent::Command => {
                        // User wants to transform the selected text
                        emit_state_change(&app_handle_clone, &state, Some("Transforming...".to_string()));

                        match groq_client.transform_text(&selected_text, &transcript).await {
                            Ok(transformed) => {
                                println!("Transformed text: {}", transformed);
                                transformed
                            }
                            Err(e) => {
                                println!("Transformation failed, keeping original: {}", e);
                                emit_error(
                                    &app_handle_clone,
                                    ErrorEvent::groq_error(&e, Some(selected_text.clone())),
                                );
                                // On error, keep the original selected text
                                selected_text
                            }
                        }
                    }
                    UserIntent::Dictation => {
                        // User wants to replace selection with new dictated content
                        // Enhance the transcript and use it to replace the selection
                        println!("Intent: Dictation - will replace selection with new content");
                        emit_state_change(&app_handle_clone, &state, Some("Enhancing...".to_string()));

                        // Get style prompt modifier (if style was captured)
                        let style_prompt = active_style.as_ref().map(|s| {
                            println!("Applying style: {} ({})", s.name, s.id);
                            s.prompt_modifier.as_str()
                        });

                        match groq_client.enhance_text(&transcript, style_prompt).await {
                            Ok(enhanced) => {
                                println!("Enhanced dictation (replacing selection): {}", enhanced);
                                enhanced
                            }
                            Err(e) => {
                                println!("Enhancement failed, using raw transcript: {}", e);
                                emit_error(
                                    &app_handle_clone,
                                    ErrorEvent::groq_error(&e, Some(transcript.clone())),
                                );
                                transcript.clone()
                            }
                        }
                    }
                }
            }
            DictationMode::Dictation => {
                // Dictation Mode: enhance transcription with context-aware style
                state.set_state(RecordingState::Enhancing);
                emit_state_change(&app_handle_clone, &state, Some("Enhancing...".to_string()));

                // Get style prompt modifier (if style was captured)
                let style_prompt = active_style.as_ref().map(|s| {
                    println!("Applying style: {} ({})", s.name, s.id);
                    s.prompt_modifier.as_str()
                });

                // Use Groq LLM for enhancement (primary), Claude as fallback
                let groq_client = GroqLlmClient::new(groq_key.clone().unwrap_or_default());
                println!("Before LLM enhancement: {}", transcript);
                match groq_client.enhance_text(&transcript, style_prompt).await {
                    Ok(enhanced) => {
                        println!("Enhanced with Groq: {}", enhanced);
                        enhanced
                    }
                    Err(groq_error) => {
                        println!("Groq enhancement failed: {}", groq_error);
                        // Fallback to Claude if available
                        if let Some(api_key) = anthropic_key {
                            let claude_client = ClaudeClient::new(api_key, Some(claude_model));
                            match claude_client.enhance_text(&transcript, style_prompt).await {
                                Ok(t) => {
                                    println!("Fallback enhanced with Claude: {}", t);
                                    t
                                }
                                Err(e) => {
                                    println!("Claude fallback also failed: {}", e);
                                    emit_error(
                                        &app_handle_clone,
                                        ErrorEvent::groq_error(&groq_error, Some(transcript.clone())),
                                    );
                                    transcript.clone()
                                }
                            }
                        } else {
                            emit_error(
                                &app_handle_clone,
                                ErrorEvent::groq_error(&groq_error, Some(transcript.clone())),
                            );
                            transcript.clone()
                        }
                    }
                }
            }
        };

        // Clean up any punctuation attached to @-tagged filenames
        // (LLM may add punctuation like "@components.json?" which breaks references)
        let final_text = ide::file_tagger::cleanup_tagged_punctuation(&final_text);

        // Emit completion
        let completion_event = TranscriptionCompleteEvent {
            raw_transcript: transcript,
            enhanced_text: final_text.clone(),
            copied_to_clipboard: false,
        };

        if let Err(e) = app_handle_clone.emit("transcription-complete", &completion_event) {
            eprintln!("Failed to emit completion: {}", e);
        }

        // Reset state
        state.set_state(RecordingState::Idle);
        state.set_mode(DictationMode::Dictation);
        state.set_selected_text(None);
        state.set_active_style(None);
        state.set_active_bundle_id(None);
        if let Ok(mut start) = state.recording_start.lock() {
            *start = None;
        }
        emit_state_change(&app_handle_clone, &state, Some("Done!".to_string()));

        // Hide overlay, wait for previous app to regain focus, then insert text
        let app_for_hide = app_handle_clone.clone();
        let text_to_insert = final_text.clone();
        std::thread::spawn(move || {
            // Brief delay to show "Done!" state
            std::thread::sleep(std::time::Duration::from_millis(300));
            hide_overlay(&app_for_hide);
            // Wait for the previous app to regain focus
            std::thread::sleep(std::time::Duration::from_millis(100));
            // Insert text (this replaces selection in Command Mode, inserts at cursor in Dictation Mode)
            insert_text_directly(&text_to_insert);
        });
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
            // New: Model and license management
            get_model_status,
            download_model,
            delete_model,
            validate_license,
            activate_license,
            get_license_info,
            clear_license,
            get_transcription_provider,
            set_transcription_provider,
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

            // NOTE: Workspace indexing disabled - Phase 3 file tagging is incomplete
            // and doesn't trigger IDE file picker. Re-enable when fixed.
            // initialize_workspace_index can be called via set_workspace_root command if needed.

            // Hide main window on start
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }

            // Hide overlay on start
            if let Some(overlay) = app.get_webview_window("overlay") {
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
