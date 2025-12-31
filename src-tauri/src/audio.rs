use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use rubato::{FftFixedIn, Resampler};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

use crate::state::AudioLevelEvent;

/// Whisper requires 16kHz audio
const WHISPER_SAMPLE_RATE: u32 = 16000;

pub struct AudioRecorder {
    audio_data: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    is_recording: Arc<AtomicBool>,
    // For audio level metering
    recent_samples: Arc<Mutex<Vec<f32>>>,
}

impl AudioRecorder {
    pub fn new() -> Self {
        AudioRecorder {
            audio_data: Arc::new(Mutex::new(Vec::new())),
            sample_rate: 48000,
            is_recording: Arc::new(AtomicBool::new(false)),
            recent_samples: Arc::new(Mutex::new(Vec::with_capacity(4800))), // ~100ms at 48kHz
        }
    }

    pub fn start_recording_with_device(
        &mut self,
        app_handle: AppHandle,
        device_name: Option<String>,
    ) -> Result<(), String> {
        println!("Starting audio recording...");
        if let Some(ref name) = device_name {
            println!("Using selected device: {}", name);
        }

        // Clear previous audio data
        if let Ok(mut data) = self.audio_data.lock() {
            data.clear();
        }
        if let Ok(mut samples) = self.recent_samples.lock() {
            samples.clear();
        }

        // Set recording flag
        self.is_recording.store(true, Ordering::SeqCst);

        let audio_data = self.audio_data.clone();
        let is_recording = self.is_recording.clone();
        let recent_samples = self.recent_samples.clone();

        // Spawn audio capture thread
        std::thread::spawn(move || {
            if let Err(e) =
                Self::capture_audio(audio_data, is_recording, recent_samples, app_handle, device_name)
            {
                eprintln!("Audio capture error: {}", e);
            }
        });

        Ok(())
    }

    pub fn stop_recording(&mut self) -> Result<Vec<u8>, String> {
        println!("Stopping audio recording...");

        // Set recording flag to false
        self.is_recording.store(false, Ordering::SeqCst);

        // Wait a bit for the recording thread to finish
        std::thread::sleep(Duration::from_millis(150));

        // Get the audio data and convert to WAV
        let audio_data = self
            .audio_data
            .lock()
            .map_err(|e| format!("Failed to lock audio data: {}", e))?;

        println!("Audio data samples: {}", audio_data.len());

        if audio_data.is_empty() {
            return Err("No audio data recorded".to_string());
        }

        // Convert to WAV format
        self.convert_to_wav(&audio_data)
    }

    /// Stop recording and return resampled audio for Whisper (16kHz)
    pub fn stop_recording_for_whisper(&mut self) -> Result<Vec<f32>, String> {
        println!("Stopping audio recording for Whisper...");

        // Set recording flag to false
        self.is_recording.store(false, Ordering::SeqCst);

        // Wait a bit for the recording thread to finish
        std::thread::sleep(Duration::from_millis(150));

        // Get the audio data
        let audio_data = self
            .audio_data
            .lock()
            .map_err(|e| format!("Failed to lock audio data: {}", e))?;

        println!("Audio data samples: {} at {}Hz", audio_data.len(), self.sample_rate);

        if audio_data.is_empty() {
            return Err("No audio data recorded".to_string());
        }

        // Resample to 16kHz for Whisper
        let resampled = self.resample_to_16khz(&audio_data)?;
        println!("Resampled to {} samples at 16kHz", resampled.len());

        Ok(resampled)
    }

    /// Resample audio from current sample rate to 16kHz for Whisper
    fn resample_to_16khz(&self, samples: &[f32]) -> Result<Vec<f32>, String> {
        if self.sample_rate == WHISPER_SAMPLE_RATE {
            // Already at 16kHz, no resampling needed
            return Ok(samples.to_vec());
        }

        let input_rate = self.sample_rate as usize;
        let output_rate = WHISPER_SAMPLE_RATE as usize;

        // Create resampler
        // Using 1024 samples per chunk for good quality
        let chunk_size = 1024;
        let mut resampler = FftFixedIn::<f32>::new(
            input_rate,
            output_rate,
            chunk_size,
            2,  // sub-chunks for quality
            1,  // mono
        )
        .map_err(|e| format!("Failed to create resampler: {}", e))?;

        let mut output = Vec::new();

        // Process in chunks
        for chunk in samples.chunks(chunk_size) {
            if chunk.len() == chunk_size {
                let input = vec![chunk.to_vec()];
                match resampler.process(&input, None) {
                    Ok(resampled) => {
                        if !resampled.is_empty() {
                            output.extend_from_slice(&resampled[0]);
                        }
                    }
                    Err(e) => {
                        eprintln!("Resampling chunk failed: {}", e);
                    }
                }
            } else if !chunk.is_empty() {
                // Handle remaining samples (pad with zeros)
                let mut padded = chunk.to_vec();
                padded.resize(chunk_size, 0.0);
                let input = vec![padded];
                match resampler.process(&input, None) {
                    Ok(resampled) => {
                        if !resampled.is_empty() {
                            // Only take the proportional amount
                            let ratio = output_rate as f32 / input_rate as f32;
                            let expected_len = (chunk.len() as f32 * ratio).ceil() as usize;
                            let take_len = expected_len.min(resampled[0].len());
                            output.extend_from_slice(&resampled[0][..take_len]);
                        }
                    }
                    Err(e) => {
                        eprintln!("Resampling final chunk failed: {}", e);
                    }
                }
            }
        }

        Ok(output)
    }

    fn capture_audio(
        audio_data: Arc<Mutex<Vec<f32>>>,
        is_recording: Arc<AtomicBool>,
        recent_samples: Arc<Mutex<Vec<f32>>>,
        app_handle: AppHandle,
        device_name: Option<String>,
    ) -> Result<(), String> {
        let host = cpal::default_host();

        // Find the device by name, or fall back to default
        let device = if let Some(ref name) = device_name {
            // Try to find the device by name
            let found_device = host
                .input_devices()
                .ok()
                .and_then(|mut devices| devices.find(|d| d.name().ok().as_ref() == Some(name)));

            match found_device {
                Some(d) => {
                    println!("Found selected device: {}", name);
                    d
                }
                None => {
                    println!(
                        "Selected device '{}' not found, falling back to default",
                        name
                    );
                    host.default_input_device()
                        .ok_or("No input device available")?
                }
            }
        } else {
            host.default_input_device()
                .ok_or("No input device available")?
        };

        println!("Using input device: {}", device.name().unwrap_or_default());

        let config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get default input config: {}", e))?;

        println!("Input config: {:?}", config);

        let err_fn = |err| eprintln!("Audio stream error: {}", err);

        // Clone for the audio level thread
        let recent_samples_for_meter = recent_samples.clone();
        let is_recording_for_meter = is_recording.clone();
        let app_handle_for_meter = app_handle.clone();

        // Spawn audio level metering thread (30fps)
        std::thread::spawn(move || {
            let mut last_emit = Instant::now();
            let emit_interval = Duration::from_millis(33); // ~30fps
            let mut frame_count = 0u32;

            while is_recording_for_meter.load(Ordering::SeqCst) {
                if last_emit.elapsed() >= emit_interval {
                    let (level, peak) = if let Ok(mut samples) = recent_samples_for_meter.lock() {
                        let result = calculate_levels(&samples);
                        samples.clear();
                        result
                    } else {
                        (0.0, 0.0)
                    };

                    // Debug: log every 30 frames (~1 second)
                    frame_count += 1;
                    if frame_count % 30 == 0 {
                        println!("Audio level: {:.3}, peak: {:.3}", level, peak);
                    }

                    let event = AudioLevelEvent { level, peak };

                    // Emit directly to the overlay window (not broadcast)
                    // This ensures the overlay receives events even after being shown/hidden
                    if let Some(overlay) = app_handle_for_meter.get_webview_window("overlay") {
                        let _ = overlay.emit("audio-level", &event);
                    }

                    last_emit = Instant::now();
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        });

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => Self::build_input_stream::<f32>(
                &device,
                &config.into(),
                audio_data.clone(),
                recent_samples.clone(),
                is_recording.clone(),
                err_fn,
            )?,
            cpal::SampleFormat::I16 => Self::build_input_stream::<i16>(
                &device,
                &config.into(),
                audio_data.clone(),
                recent_samples.clone(),
                is_recording.clone(),
                err_fn,
            )?,
            cpal::SampleFormat::U16 => Self::build_input_stream::<u16>(
                &device,
                &config.into(),
                audio_data.clone(),
                recent_samples.clone(),
                is_recording.clone(),
                err_fn,
            )?,
            _ => return Err("Unsupported sample format".to_string()),
        };

        stream
            .play()
            .map_err(|e| format!("Failed to play stream: {}", e))?;

        // Keep the stream alive while recording
        while is_recording.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(50));
        }

        drop(stream);
        println!("Audio capture stopped");

        Ok(())
    }

    fn build_input_stream<T>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        audio_data: Arc<Mutex<Vec<f32>>>,
        recent_samples: Arc<Mutex<Vec<f32>>>,
        is_recording: Arc<AtomicBool>,
        err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
    ) -> Result<cpal::Stream, String>
    where
        T: cpal::Sample + cpal::SizedSample + Send + 'static,
        f32: FromSample<T>,
    {
        let channels = config.channels as usize;

        let stream = device
            .build_input_stream(
                config,
                move |data: &[T], _: &cpal::InputCallbackInfo| {
                    if is_recording.load(Ordering::SeqCst) {
                        // Convert to mono f32 samples
                        let mut mono_samples: Vec<f32> = Vec::with_capacity(data.len() / channels);

                        for frame in data.chunks(channels) {
                            let sum: f32 = frame.iter().map(|&s| f32::from_sample(s)).sum();
                            let mono_sample = sum / channels as f32;
                            mono_samples.push(mono_sample);
                        }

                        // Store for WAV output
                        if let Ok(mut audio) = audio_data.lock() {
                            audio.extend_from_slice(&mono_samples);
                        }

                        // Store for level metering
                        if let Ok(mut recent) = recent_samples.lock() {
                            recent.extend_from_slice(&mono_samples);
                            // Keep only recent samples (prevent unbounded growth)
                            let len = recent.len();
                            if len > 9600 {
                                // ~200ms at 48kHz
                                recent.drain(0..len - 4800);
                            }
                        }
                    }
                },
                err_fn,
                None,
            )
            .map_err(|e| format!("Failed to build input stream: {}", e))?;

        Ok(stream)
    }

    fn convert_to_wav(&self, samples: &[f32]) -> Result<Vec<u8>, String> {
        let result = encode_samples_to_wav(samples, self.sample_rate)?;
        #[cfg(debug_assertions)]
        println!("WAV buffer size: {} bytes", result.len());
        Ok(result)
    }
}

/// Encode f32 audio samples to WAV format.
/// This is the canonical WAV encoding function used throughout the app.
pub fn encode_samples_to_wav(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>, String> {
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

impl Default for AudioRecorder {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate RMS level and peak from audio samples using proper dB normalization
/// Uses -60dB to 0dB range like VoiceInk for consistent, non-flickery visualization
fn calculate_levels(samples: &[f32]) -> (f32, f32) {
    if samples.is_empty() {
        return (0.0, 0.0);
    }

    let mut sum_squares = 0.0;
    let mut peak = 0.0f32;

    for &sample in samples {
        let abs_sample = sample.abs();
        sum_squares += sample * sample;
        if abs_sample > peak {
            peak = abs_sample;
        }
    }

    let rms = (sum_squares / samples.len() as f32).sqrt();

    // Convert RMS to dB (reference level is 1.0)
    // Add small epsilon to avoid log(0)
    let rms_db = 20.0 * (rms + 1e-10).log10();
    let peak_db = 20.0 * (peak + 1e-10).log10();

    // Normalize dB to 0-1 range using -60dB to 0dB range
    // Below -60dB = complete silence (noise floor)
    // At 0dB = maximum level
    const MIN_DB: f32 = -60.0;
    const MAX_DB: f32 = 0.0;

    let level = if rms_db < MIN_DB {
        0.0
    } else if rms_db >= MAX_DB {
        1.0
    } else {
        (rms_db - MIN_DB) / (MAX_DB - MIN_DB)
    };

    let peak_normalized = if peak_db < MIN_DB {
        0.0
    } else if peak_db >= MAX_DB {
        1.0
    } else {
        (peak_db - MIN_DB) / (MAX_DB - MIN_DB)
    };

    (level, peak_normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== calculate_levels Tests ====================

    #[test]
    fn test_calculate_levels_empty_samples() {
        let (level, peak) = calculate_levels(&[]);
        assert_eq!(level, 0.0);
        assert_eq!(peak, 0.0);
    }

    #[test]
    fn test_calculate_levels_silence() {
        // Complete silence (all zeros)
        let samples = vec![0.0; 1000];
        let (level, peak) = calculate_levels(&samples);
        assert_eq!(level, 0.0);
        assert_eq!(peak, 0.0);
    }

    #[test]
    fn test_calculate_levels_max_amplitude() {
        // Maximum amplitude (all 1.0)
        let samples = vec![1.0; 1000];
        let (level, peak) = calculate_levels(&samples);
        assert!((level - 1.0).abs() < 0.01, "Level should be near 1.0, got {}", level);
        assert!((peak - 1.0).abs() < 0.01, "Peak should be near 1.0, got {}", peak);
    }

    #[test]
    fn test_calculate_levels_negative_max_amplitude() {
        // Maximum negative amplitude (all -1.0)
        let samples = vec![-1.0; 1000];
        let (level, peak) = calculate_levels(&samples);
        assert!((level - 1.0).abs() < 0.01, "Level should be near 1.0, got {}", level);
        assert!((peak - 1.0).abs() < 0.01, "Peak should be near 1.0, got {}", peak);
    }

    #[test]
    fn test_calculate_levels_mixed_amplitude() {
        // Alternating positive and negative samples
        let samples: Vec<f32> = (0..1000).map(|i| if i % 2 == 0 { 0.5 } else { -0.5 }).collect();
        let (level, peak) = calculate_levels(&samples);

        // RMS should be around 0.5, peak should be 0.5
        // In dB: 20 * log10(0.5) ≈ -6 dB
        // Normalized: (-6 - (-60)) / 60 = 54/60 = 0.9
        assert!(level > 0.8 && level < 1.0, "Level should be around 0.9, got {}", level);
        assert!(peak > 0.8 && peak < 1.0, "Peak should be around 0.9, got {}", peak);
    }

    #[test]
    fn test_calculate_levels_very_quiet() {
        // Very quiet signal (0.001 amplitude)
        // 20 * log10(0.001) = -60 dB, which is the minimum
        let samples = vec![0.001; 1000];
        let (level, peak) = calculate_levels(&samples);
        assert!(level >= 0.0 && level < 0.1, "Level should be near 0, got {}", level);
        assert!(peak >= 0.0 && peak < 0.1, "Peak should be near 0, got {}", peak);
    }

    #[test]
    fn test_calculate_levels_single_sample() {
        let (level, peak) = calculate_levels(&[0.5]);
        assert!(level > 0.0 && level < 1.0, "Level should be between 0 and 1");
        assert!(peak > 0.0 && peak < 1.0, "Peak should be between 0 and 1");
    }

    #[test]
    fn test_calculate_levels_spike_detection() {
        // Mostly quiet with one loud spike
        let mut samples = vec![0.01; 999];
        samples.push(1.0);
        let (level, peak) = calculate_levels(&samples);

        // Peak should detect the spike
        assert!((peak - 1.0).abs() < 0.01, "Peak should detect the spike, got {}", peak);
        // RMS level should be much lower than peak
        assert!(level < peak, "RMS should be lower than peak");
    }

    #[test]
    fn test_calculate_levels_returns_in_range() {
        // Test with random-ish values to ensure output is always in [0, 1]
        let samples: Vec<f32> = (0..100).map(|i| {
            let x = (i as f32 * 0.1).sin() * 0.8;
            x
        }).collect();
        let (level, peak) = calculate_levels(&samples);

        assert!(level >= 0.0 && level <= 1.0, "Level must be in [0, 1], got {}", level);
        assert!(peak >= 0.0 && peak <= 1.0, "Peak must be in [0, 1], got {}", peak);
    }

    #[test]
    fn test_calculate_levels_above_unity_clamps() {
        // If somehow we get samples > 1.0, peak should clamp to 1.0
        let samples = vec![2.0; 100]; // Above unity
        let (level, peak) = calculate_levels(&samples);
        assert_eq!(peak, 1.0, "Peak should clamp to 1.0");
        assert_eq!(level, 1.0, "Level should clamp to 1.0");
    }

    // ==================== AudioRecorder Tests ====================

    #[test]
    fn test_audio_recorder_new() {
        let recorder = AudioRecorder::new();
        // Default sample rate should be 48000
        assert_eq!(recorder.sample_rate, 48000);
        assert!(!recorder.is_recording.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn test_audio_recorder_default() {
        let recorder = AudioRecorder::default();
        assert_eq!(recorder.sample_rate, 48000);
    }

    // ==================== Resampling Tests ====================
    // Note: Full resampling tests require more setup, but we can test edge cases

    #[test]
    fn test_resample_same_rate_returns_copy() {
        let recorder = AudioRecorder {
            audio_data: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            sample_rate: 16000, // Same as WHISPER_SAMPLE_RATE
            is_recording: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            recent_samples: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        };

        let samples = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let result = recorder.resample_to_16khz(&samples).unwrap();

        assert_eq!(result.len(), samples.len());
        for (a, b) in result.iter().zip(samples.iter()) {
            assert!((a - b).abs() < 0.001, "Samples should match");
        }
    }

    #[test]
    fn test_resample_empty_input() {
        let recorder = AudioRecorder::new();
        let result = recorder.resample_to_16khz(&[]).unwrap();
        assert!(result.is_empty());
    }
}
