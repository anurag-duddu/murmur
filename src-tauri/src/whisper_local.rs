//! Local Whisper transcription using whisper-rs (whisper.cpp bindings).
//! This is used for one-time purchase users who download the model locally.

use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::sync::Mutex;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::model_manager;

/// Global Whisper context (loaded once, reused for all transcriptions)
static WHISPER_CONTEXT: OnceLock<Mutex<Option<WhisperContext>>> = OnceLock::new();

pub struct WhisperLocalClient;

impl WhisperLocalClient {
    /// Transcribe audio samples using local Whisper model
    ///
    /// # Arguments
    /// * `samples` - Audio samples as f32, must be 16kHz mono
    /// * `language` - Language code (e.g., "en", "auto")
    pub async fn transcribe(samples: &[f32], language: &str) -> Result<String, String> {
        // Get or initialize the context
        let context_mutex = WHISPER_CONTEXT.get_or_init(|| Mutex::new(None));
        let mut context_guard = context_mutex.lock().await;

        // Load model if not already loaded
        if context_guard.is_none() {
            let model_path = model_manager::get_model_path()
                .ok_or("Whisper model not downloaded. Please download the model first.")?;

            println!("Loading Whisper model from: {:?}", model_path);
            let ctx = Self::load_model(&model_path)?;
            *context_guard = Some(ctx);
            println!("Whisper model loaded successfully");
        }

        let ctx = context_guard
            .as_ref()
            .ok_or("Failed to get Whisper context")?;

        // Configure transcription parameters
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // Set language
        if language != "auto" {
            // Convert "en-US" to "en"
            let lang_code = language.split('-').next().unwrap_or(language);
            params.set_language(Some(lang_code));
        }

        // Performance optimizations
        let num_threads = std::cmp::min(8, num_cpus::get() as i32).max(1);
        params.set_n_threads(num_threads);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_single_segment(false);
        params.set_no_context(true);
        params.set_temperature(0.0); // Greedy decoding for speed

        println!(
            "Transcribing {} samples with {} threads...",
            samples.len(),
            num_threads
        );

        // Create state and run inference
        let mut state = ctx
            .create_state()
            .map_err(|e| format!("Failed to create Whisper state: {}", e))?;

        state
            .full(params, samples)
            .map_err(|e| format!("Whisper transcription failed: {}", e))?;

        // Collect all segments
        let num_segments = state
            .full_n_segments()
            .map_err(|e| format!("Failed to get segment count: {}", e))?;

        let mut transcript = String::new();
        for i in 0..num_segments {
            if let Ok(segment_text) = state.full_get_segment_text(i) {
                transcript.push_str(&segment_text);
            }
        }

        let result = transcript.trim().to_string();
        println!("Local Whisper transcript: {}", result);

        Ok(result)
    }

    /// Load Whisper model from disk
    fn load_model(model_path: &PathBuf) -> Result<WhisperContext, String> {
        let params = WhisperContextParameters::default();
        // Note: GPU acceleration is enabled by default on macOS via Metal

        WhisperContext::new_with_params(&model_path.to_string_lossy(), params)
            .map_err(|e| format!("Failed to load Whisper model: {}", e))
    }
}
