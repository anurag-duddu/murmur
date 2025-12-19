//! Model download and management for local Whisper transcription.
//! Downloads models from HuggingFace and stores them in the app data directory.

use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

/// Default model for one-time purchase users (best accuracy/speed balance)
pub const DEFAULT_MODEL: &str = "ggml-large-v3-turbo-q5_0.bin";
pub const DEFAULT_MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q5_0.bin";
pub const DEFAULT_MODEL_SIZE_MB: u64 = 547;

/// Model download progress event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDownloadProgress {
    pub model_name: String,
    pub progress: f64, // 0.0 to 1.0
    pub downloaded_mb: f64,
    pub total_mb: f64,
    pub status: String, // "downloading", "completed", "error"
}

/// Model status information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    pub name: String,
    pub size_mb: u64,
    pub downloaded: bool,
    pub path: Option<String>,
}

/// Get the models directory path
pub fn get_models_directory() -> Result<PathBuf, String> {
    let app_data = dirs::data_dir().ok_or("Could not find app data directory")?;

    let models_dir = app_data.join("murmur").join("whisper_models");

    fs::create_dir_all(&models_dir)
        .map_err(|e| format!("Failed to create models directory: {}", e))?;

    Ok(models_dir)
}

/// Get the path to the default model (if downloaded)
pub fn get_model_path() -> Option<PathBuf> {
    let models_dir = get_models_directory().ok()?;
    let path = models_dir.join(DEFAULT_MODEL);

    if path.exists() {
        Some(path)
    } else {
        None
    }
}

/// Check if the default model is downloaded
pub fn is_model_downloaded() -> bool {
    get_model_path().is_some()
}

/// Get the status of the default model
pub fn get_model_status() -> ModelStatus {
    let path = get_model_path();
    ModelStatus {
        name: DEFAULT_MODEL.to_string(),
        size_mb: DEFAULT_MODEL_SIZE_MB,
        downloaded: path.is_some(),
        path: path.map(|p| p.to_string_lossy().to_string()),
    }
}

/// Download the default Whisper model with progress reporting
pub async fn download_model(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let models_dir = get_models_directory()?;
    let model_path = models_dir.join(DEFAULT_MODEL);

    // Check if already downloaded
    if model_path.exists() {
        println!("Model already downloaded at: {:?}", model_path);
        return Ok(model_path);
    }

    println!("Downloading Whisper model: {}", DEFAULT_MODEL);
    println!("From: {}", DEFAULT_MODEL_URL);

    let client = Client::new();

    let response = client
        .get(DEFAULT_MODEL_URL)
        .send()
        .await
        .map_err(|e| format!("Failed to start download: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    let total_size = response.content_length().unwrap_or(0);
    let total_mb = total_size as f64 / 1_000_000.0;

    // Create temporary file for downloading
    let temp_path = model_path.with_extension("tmp");
    let mut file =
        fs::File::create(&temp_path).map_err(|e| format!("Failed to create temp file: {}", e))?;

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    let mut last_progress_emit = std::time::Instant::now();

    // Emit initial progress
    emit_progress(app_handle, 0.0, 0.0, total_mb, "downloading");

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Download stream error: {}", e))?;

        file.write_all(&chunk)
            .map_err(|e| format!("Failed to write to file: {}", e))?;

        downloaded += chunk.len() as u64;

        // Emit progress every 100ms to avoid flooding
        if last_progress_emit.elapsed().as_millis() >= 100 {
            let progress = if total_size > 0 {
                downloaded as f64 / total_size as f64
            } else {
                0.0
            };

            emit_progress(
                app_handle,
                progress,
                downloaded as f64 / 1_000_000.0,
                total_mb,
                "downloading",
            );

            last_progress_emit = std::time::Instant::now();
        }
    }

    // Ensure all data is flushed
    file.flush()
        .map_err(|e| format!("Failed to flush file: {}", e))?;
    drop(file);

    // Rename temp file to final path
    fs::rename(&temp_path, &model_path)
        .map_err(|e| format!("Failed to finalize download: {}", e))?;

    // Emit completion
    emit_progress(app_handle, 1.0, total_mb, total_mb, "completed");

    println!("Model downloaded successfully: {:?}", model_path);
    Ok(model_path)
}

/// Delete the downloaded model
pub fn delete_model() -> Result<(), String> {
    if let Some(path) = get_model_path() {
        fs::remove_file(&path).map_err(|e| format!("Failed to delete model: {}", e))?;
        println!("Model deleted: {:?}", path);
    }
    Ok(())
}

/// Helper to emit download progress event
fn emit_progress(app_handle: &AppHandle, progress: f64, downloaded_mb: f64, total_mb: f64, status: &str) {
    let event = ModelDownloadProgress {
        model_name: DEFAULT_MODEL.to_string(),
        progress,
        downloaded_mb,
        total_mb,
        status: status.to_string(),
    };

    if let Err(e) = app_handle.emit("model-download-progress", &event) {
        eprintln!("Failed to emit download progress: {}", e);
    }
}
