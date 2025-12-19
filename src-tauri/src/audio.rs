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

    pub fn start_recording(&mut self, app_handle: AppHandle) -> Result<(), String> {
        println!("Starting audio recording...");

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
            if let Err(e) = Self::capture_audio(audio_data, is_recording, recent_samples, app_handle)
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
    ) -> Result<(), String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No input device available")?;

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
        let mut buffer = Vec::new();

        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: self.sample_rate,
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

        println!("WAV buffer size: {} bytes", buffer.len());
        Ok(buffer)
    }
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
