//! Model download and management for local Whisper transcription.
//! Downloads models from HuggingFace and stores them in the app data directory.

use crate::rate_limit::{check_rate_limit, Service};
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};

/// Default model for one-time purchase users (best accuracy/speed balance)
pub const DEFAULT_MODEL: &str = "ggml-large-v3-turbo-q5_0.bin";
pub const DEFAULT_MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q5_0.bin";
pub const DEFAULT_MODEL_SIZE_MB: u64 = 547;

/// Expected SHA256 checksum for the model file.
/// This ensures the downloaded file hasn't been corrupted or tampered with.
/// Checksum obtained from: https://huggingface.co/ggerganov/whisper.cpp/blob/main/ggml-large-v3-turbo-q5_0.bin
pub const DEFAULT_MODEL_SHA256: &str = "e050f7ed11eb01a952f09d89db6e2d49e9c3cc4e4f4e9c69c01a6f22a74b7e5b";

/// Calculate SHA256 checksum of a file
fn calculate_file_sha256(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path)
        .map_err(|e| format!("Failed to open file for checksum: {}", e))?;

    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)
            .map_err(|e| format!("Failed to read file for checksum: {}", e))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Verify the integrity of a downloaded model file
fn verify_model_checksum(path: &Path, expected: &str) -> Result<(), String> {
    println!("Verifying model checksum...");

    let actual = calculate_file_sha256(path)?;

    if actual != expected {
        return Err(format!(
            "Checksum mismatch! Expected: {}, Got: {}. The download may be corrupted.",
            expected, actual
        ));
    }

    println!("Checksum verified: {}", actual);
    Ok(())
}

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

    // Check rate limit before making download request
    check_rate_limit(Service::ModelDownload)?;

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

    // Verify checksum before finalizing
    emit_progress(app_handle, 1.0, total_mb, total_mb, "verifying");
    if let Err(e) = verify_model_checksum(&temp_path, DEFAULT_MODEL_SHA256) {
        // Checksum failed - delete the corrupted file and report error
        let _ = fs::remove_file(&temp_path);
        emit_progress(app_handle, 0.0, 0.0, total_mb, "error");
        return Err(format!("Model integrity check failed: {}. Please try downloading again.", e));
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_calculate_file_sha256() {
        // Create a temp file with known content
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"hello world").unwrap();
        temp_file.flush().unwrap();

        let checksum = calculate_file_sha256(temp_file.path()).unwrap();

        // SHA256 of "hello world"
        assert_eq!(
            checksum,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_verify_model_checksum_success() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test content").unwrap();
        temp_file.flush().unwrap();

        // Calculate the actual checksum first
        let actual_checksum = calculate_file_sha256(temp_file.path()).unwrap();

        // Verification should pass with correct checksum
        let result = verify_model_checksum(temp_file.path(), &actual_checksum);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_model_checksum_failure() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test content").unwrap();
        temp_file.flush().unwrap();

        // Verification should fail with wrong checksum
        let result = verify_model_checksum(temp_file.path(), "0000000000000000000000000000000000000000000000000000000000000000");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Checksum mismatch"));
    }

    #[test]
    fn test_model_constants_valid() {
        // Verify constants are properly formatted
        assert!(!DEFAULT_MODEL.is_empty());
        assert!(DEFAULT_MODEL.ends_with(".bin"));
        assert!(DEFAULT_MODEL_URL.starts_with("https://"));
        assert_eq!(DEFAULT_MODEL_SHA256.len(), 64); // SHA256 hex length
    }
}
