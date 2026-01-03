//! Whisper API client using Groq for subscription tier users.
//! Groq provides extremely fast Whisper transcription with an OpenAI-compatible API.
//!
//! Supports two modes:
//! - **Native mode**: Strict single-language transcription in native script
//! - **Mixed mode**: Auto-detect among user's spoken languages, romanized output

use crate::http_client;
use crate::rate_limit::{check_rate_limit, Service};
use crate::signing;
use any_ascii::any_ascii;
use reqwest::{multipart, Client};
use serde::Deserialize;

const WHISPER_MODEL: &str = "whisper-large-v3-turbo";

/// Response from Groq Whisper API (simple format)
#[derive(Debug, Deserialize)]
struct GroqTranscriptionResponse {
    text: String,
}

/// Response from Groq Whisper API (verbose_json format for mixed mode)
#[derive(Debug, Deserialize)]
struct GroqVerboseResponse {
    text: String,
    language: Option<String>,
    #[serde(default)]
    segments: Vec<GroqSegment>,
}

#[derive(Debug, Deserialize)]
struct GroqSegment {
    #[allow(dead_code)]
    text: String,
    #[serde(default)]
    avg_logprob: f64,
}

pub struct WhisperApiClient {
    client: &'static Client,
}

impl WhisperApiClient {
    pub fn new() -> Result<Self, String> {
        // Use cached transcription client for connection reuse
        Ok(WhisperApiClient {
            client: http_client::get_transcription_client()?,
        })
    }

    /// Transcribe audio using Groq's Whisper API
    ///
    /// # Arguments
    /// * `audio_wav` - WAV-encoded audio bytes
    /// * `language` - Language code (e.g., "en-US", "hi", "te") or "mixed" for mixed mode
    /// * `spoken_languages` - List of languages the user speaks (for mixed mode validation)
    ///
    /// Two modes:
    /// - **Native mode** (specific language): Strict transcription in that language's native script
    /// - **Mixed mode** (language="mixed"): Auto-detect among spoken_languages, romanized output
    pub async fn transcribe(
        &self,
        audio_wav: &[u8],
        language: &str,
        spoken_languages: &[String],
    ) -> Result<String, String> {
        // Check rate limit before making API call
        check_rate_limit(Service::WhisperApi)?;

        println!(
            "Sending audio to Groq Whisper API ({} bytes)...",
            audio_wav.len()
        );

        let (api_url, api_key) = self.get_api_config();

        // Handle "auto" as mixed mode (legacy support for old stored preferences)
        let is_mixed_mode = language == "mixed" || language == "auto";
        let lang_code = language.split('-').next().unwrap_or(language);

        if is_mixed_mode {
            self.transcribe_mixed_mode(audio_wav, &api_url, api_key.as_deref(), spoken_languages)
                .await
        } else {
            self.transcribe_native_mode(audio_wav, &api_url, api_key.as_deref(), lang_code)
                .await
        }
    }

    /// Native mode: Strict single-language transcription
    /// Output is in native script (no romanization) for the specified language
    async fn transcribe_native_mode(
        &self,
        audio_wav: &[u8],
        api_url: &str,
        api_key: Option<&str>,
        lang_code: &str,
    ) -> Result<String, String> {
        println!("Native mode: strict {} transcription", lang_code);

        let file_part = multipart::Part::bytes(audio_wav.to_vec())
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| format!("Failed to create file part: {}", e))?;

        let mut form = multipart::Form::new()
            .part("file", file_part)
            .text("model", WHISPER_MODEL)
            .text("language", lang_code.to_string());

        // Add language-specific prompts for non-English to ensure native script output
        let prompt = match lang_code {
            "hi" => Some("हिंदी में ट्रांसक्राइब करें। मेरा नाम अनुराग है।"),
            "te" => Some("తెలుగులో ట్రాన్స్క్రిప్ట్ చేయండి. నా పేరు అనురాగ్."),
            "ta" => Some("தமிழில் டிரான்ஸ்கிரிப்ட் செய்யுங்கள். என் பெயர் அனுராக்."),
            "kn" => Some("ಕನ್ನಡದಲ್ಲಿ ಟ್ರಾನ್ಸ್ಕ್ರಿಪ್ಟ್ ಮಾಡಿ."),
            "ml" => Some("മലയാളത്തിൽ ട്രാൻസ്ക്രിപ്റ്റ് ചെയ്യുക."),
            "bn" => Some("বাংলায় ট্রান্সক্রিপ্ট করুন।"),
            "ja" => Some("日本語で文字起こしをしてください。"),
            "zh" => Some("请用中文转录。"),
            "ko" => Some("한국어로 전사해 주세요."),
            _ => None,
        };

        if let Some(p) = prompt {
            form = form.text("prompt", p);
        }

        let mut request = self.client.post(api_url).multipart(form);

        // Add Authorization header only for direct API (dev mode)
        // Add HMAC signature for proxy mode
        if let Some(key) = api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        } else {
            // Production mode: add HMAC signature for proxy authentication
            // Sign the audio bytes as the request payload
            let (timestamp, nonce, signature) = signing::sign_request(audio_wav);
            request = request
                .header("X-Murmur-Timestamp", timestamp)
                .header("X-Murmur-Nonce", nonce)
                .header("X-Murmur-Signature", signature);
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("Failed to send request to Groq: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Groq API error ({}): {}", status, error_text));
        }

        let result: GroqTranscriptionResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Groq response: {}", e))?;

        let transcript = result.text.trim().to_string();
        println!("Native mode transcript ({}): {}", lang_code, transcript);

        Ok(transcript)
    }

    /// Mixed mode: Auto-detect among user's spoken languages with romanized output
    /// Uses verbose_json to get detected language and confidence
    async fn transcribe_mixed_mode(
        &self,
        audio_wav: &[u8],
        api_url: &str,
        api_key: Option<&str>,
        spoken_languages: &[String],
    ) -> Result<String, String> {
        println!("Mixed mode: detecting among {:?}", spoken_languages);

        let file_part = multipart::Part::bytes(audio_wav.to_vec())
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| format!("Failed to create file part: {}", e))?;

        // Build prompt listing user's spoken languages
        let lang_names = spoken_languages
            .iter()
            .map(|code| language_code_to_name(code))
            .collect::<Vec<_>>()
            .join(", ");

        let prompt = format!(
            "This speaker uses these languages: {}. TRANSCRIBE (do NOT translate). Output the exact words spoken in the original language. Never convert one language to another.",
            lang_names
        );

        let form = multipart::Form::new()
            .part("file", file_part)
            .text("model", WHISPER_MODEL)
            .text("response_format", "verbose_json")
            .text("prompt", prompt);
        // Note: Not setting "language" parameter - let Whisper auto-detect

        let mut request = self.client.post(api_url).multipart(form);

        // Add Authorization header only for direct API (dev mode)
        // Add HMAC signature for proxy mode
        if let Some(key) = api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        } else {
            // Production mode: add HMAC signature for proxy authentication
            // Sign the audio bytes as the request payload
            let (timestamp, nonce, signature) = signing::sign_request(audio_wav);
            request = request
                .header("X-Murmur-Timestamp", timestamp)
                .header("X-Murmur-Nonce", nonce)
                .header("X-Murmur-Signature", signature);
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("Failed to send request to Groq: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Groq API error ({}): {}", status, error_text));
        }

        let result: GroqVerboseResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Groq verbose response: {}", e))?;

        let detected_lang = result.language.as_deref().unwrap_or("unknown");

        // Calculate average confidence from segments
        let avg_confidence = if !result.segments.is_empty() {
            let sum: f64 = result.segments.iter().map(|s| s.avg_logprob).sum();
            sum / result.segments.len() as f64
        } else {
            -1.0 // Unknown confidence
        };

        // Convert log probability to confidence (higher is better, 0 is perfect)
        // avg_logprob > -0.3 is generally good confidence (~74% probability)
        let is_confident = avg_confidence > -0.3;

        println!(
            "Mixed mode: detected '{}', avg_logprob={:.3}, confident={}",
            detected_lang, avg_confidence, is_confident
        );

        // Validate detected language is in user's spoken languages
        let detected_code = language_name_to_code(detected_lang);
        let is_valid_lang = spoken_languages
            .iter()
            .any(|l| l.split('-').next().unwrap_or(l) == detected_code);

        if !is_valid_lang && is_confident {
            println!(
                "Warning: Detected '{}' not in user's languages {:?}, but high confidence",
                detected_lang, spoken_languages
            );
            // Still use the transcript, user might be using a new language
        }

        let transcript = result.text.trim().to_string();
        println!("Mixed mode raw transcript: {}", transcript);

        // Always romanize in mixed mode for code-switched output (Hinglish, Tenglish, etc.)
        let romanized = romanize_transcript(&transcript);
        println!("Mixed mode romanized: {}", romanized);

        Ok(romanized)
    }

    /// Get the API URL and optional API key based on build type.
    /// - Debug builds: use direct API with GROQ_API_KEY
    /// - Release builds: always use proxy
    fn get_api_config(&self) -> (String, Option<String>) {
        let (whisper_url, _, api_key) = signing::get_api_config();
        (whisper_url.to_string(), api_key)
    }
}

/// Romanize non-Latin text to get Hinglish/Tenglish style output.
/// Converts scripts like Devanagari (मेरा) and Telugu (తెలుగు) to Latin letters (meraa, telugu).
/// Preserves existing Latin characters and punctuation.
fn romanize_transcript(text: &str) -> String {
    let mut result = String::new();

    for ch in text.chars() {
        // Check if character is already Latin/ASCII or common punctuation
        if ch.is_ascii() {
            result.push(ch);
        } else {
            // Convert non-Latin character using any_ascii
            let romanized = any_ascii(&ch.to_string());
            if !romanized.is_empty() {
                result.push_str(&romanized);
            }
        }
    }

    // Clean up any double spaces
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

use std::collections::HashMap;
use std::sync::OnceLock;

/// Language mappings - defined once, used for O(1) lookups in both directions
static LANGUAGE_CODE_TO_NAME: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
static LANGUAGE_NAME_TO_CODE: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

/// Initialize language mappings (code -> name and lowercase name -> code)
fn init_language_maps() -> (
    HashMap<&'static str, &'static str>,
    HashMap<&'static str, &'static str>,
) {
    let pairs: &[(&str, &str)] = &[
        ("en", "English"),
        ("hi", "Hindi"),
        ("te", "Telugu"),
        ("ta", "Tamil"),
        ("kn", "Kannada"),
        ("ml", "Malayalam"),
        ("bn", "Bengali"),
        ("mr", "Marathi"),
        ("gu", "Gujarati"),
        ("pa", "Punjabi"),
        ("es", "Spanish"),
        ("fr", "French"),
        ("de", "German"),
        ("it", "Italian"),
        ("pt", "Portuguese"),
        ("nl", "Dutch"),
        ("ja", "Japanese"),
        ("ko", "Korean"),
        ("zh", "Chinese"),
        ("ru", "Russian"),
        ("pl", "Polish"),
        ("tr", "Turkish"),
        ("uk", "Ukrainian"),
        ("vi", "Vietnamese"),
        ("id", "Indonesian"),
        ("th", "Thai"),
        ("sv", "Swedish"),
        ("da", "Danish"),
        ("no", "Norwegian"),
        ("fi", "Finnish"),
    ];

    let code_to_name: HashMap<_, _> = pairs.iter().copied().collect();
    let name_to_code: HashMap<_, _> = pairs
        .iter()
        .map(|(code, name)| {
            // Store lowercase name for case-insensitive lookup
            let lowercase: &'static str = match *name {
                "English" => "english",
                "Hindi" => "hindi",
                "Telugu" => "telugu",
                "Tamil" => "tamil",
                "Kannada" => "kannada",
                "Malayalam" => "malayalam",
                "Bengali" => "bengali",
                "Marathi" => "marathi",
                "Gujarati" => "gujarati",
                "Punjabi" => "punjabi",
                "Spanish" => "spanish",
                "French" => "french",
                "German" => "german",
                "Italian" => "italian",
                "Portuguese" => "portuguese",
                "Dutch" => "dutch",
                "Japanese" => "japanese",
                "Korean" => "korean",
                "Chinese" => "chinese",
                "Russian" => "russian",
                "Polish" => "polish",
                "Turkish" => "turkish",
                "Ukrainian" => "ukrainian",
                "Vietnamese" => "vietnamese",
                "Indonesian" => "indonesian",
                "Thai" => "thai",
                "Swedish" => "swedish",
                "Danish" => "danish",
                "Norwegian" => "norwegian",
                "Finnish" => "finnish",
                _ => "unknown",
            };
            (lowercase, *code)
        })
        .collect();

    (code_to_name, name_to_code)
}

/// Convert language code to full language name (for prompts) - O(1) lookup
fn language_code_to_name(code: &str) -> &'static str {
    let map = LANGUAGE_CODE_TO_NAME.get_or_init(|| init_language_maps().0);
    let base_code = code.split('-').next().unwrap_or(code);
    map.get(base_code).copied().unwrap_or("Unknown")
}

/// Convert language name (from Whisper response) to code - O(1) lookup
fn language_name_to_code(name: &str) -> &'static str {
    let map = LANGUAGE_NAME_TO_CODE.get_or_init(|| init_language_maps().1);
    // Create a lowercase version for lookup
    let lowercase = name.to_lowercase();
    map.get(lowercase.as_str()).copied().unwrap_or("unknown")
}
