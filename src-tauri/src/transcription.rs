//! Unified transcription interface that routes to the appropriate backend
//! based on user's license tier and configuration.

use crate::config::TranscriptionProvider;
use crate::deepgram::DeepgramClient;
use crate::whisper_api::WhisperApiClient;
use crate::whisper_local::WhisperLocalClient;

/// Unified transcription result
pub struct TranscriptionResult {
    pub transcript: String,
    pub provider: TranscriptionProvider,
}

/// Transcribe audio using the configured provider
///
/// # Arguments
/// * `audio_wav` - WAV-encoded audio bytes (48kHz mono 16-bit PCM)
/// * `audio_samples` - Raw f32 samples at 16kHz (for Whisper providers)
/// * `provider` - Which transcription backend to use
/// * `language` - Language code (e.g., "en-US", "mixed")
/// * `spoken_languages` - User's spoken languages (for mixed mode)
/// * `deepgram_key` - Deepgram API key (for BYOK)
/// * `license_key` - User's license key (for subscription/lifetime tiers)
pub async fn transcribe(
    audio_wav: &[u8],
    audio_samples_16khz: &[f32],
    provider: &TranscriptionProvider,
    language: &str,
    spoken_languages: &[String],
    deepgram_key: Option<&str>,
    license_key: Option<&str>,
) -> Result<TranscriptionResult, String> {
    match provider {
        TranscriptionProvider::Deepgram => {
            let api_key = deepgram_key.ok_or("Deepgram API key not configured")?;
            let client = DeepgramClient::new(api_key.to_string(), Some(language.to_string()));
            let transcript = client.transcribe_audio(audio_wav.to_vec()).await?;
            Ok(TranscriptionResult {
                transcript,
                provider: TranscriptionProvider::Deepgram,
            })
        }
        TranscriptionProvider::WhisperApi => {
            let key = license_key.ok_or("License key required for Whisper API")?;
            let client = WhisperApiClient::new(key.to_string());
            let transcript = client.transcribe(audio_wav, language, spoken_languages).await?;
            Ok(TranscriptionResult {
                transcript,
                provider: TranscriptionProvider::WhisperApi,
            })
        }
        TranscriptionProvider::WhisperLocal => {
            let transcript = WhisperLocalClient::transcribe(audio_samples_16khz, language).await?;
            Ok(TranscriptionResult {
                transcript,
                provider: TranscriptionProvider::WhisperLocal,
            })
        }
    }
}

/// Check if the given provider is available for use
pub fn is_provider_available(
    provider: &TranscriptionProvider,
    deepgram_key: Option<&str>,
    license_key: Option<&str>,
    model_downloaded: bool,
) -> bool {
    match provider {
        TranscriptionProvider::Deepgram => deepgram_key.is_some(),
        TranscriptionProvider::WhisperApi => license_key.is_some(),
        TranscriptionProvider::WhisperLocal => license_key.is_some() && model_downloaded,
    }
}

/// Get the best available provider based on user's entitlements
pub fn get_best_provider(
    has_subscription: bool,
    has_lifetime: bool,
    model_downloaded: bool,
    deepgram_key: Option<&str>,
) -> Option<TranscriptionProvider> {
    // Priority: Subscription (cloud) > Lifetime (local) > BYOK (Deepgram)
    if has_subscription {
        Some(TranscriptionProvider::WhisperApi)
    } else if has_lifetime && model_downloaded {
        Some(TranscriptionProvider::WhisperLocal)
    } else if deepgram_key.is_some() {
        Some(TranscriptionProvider::Deepgram)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== is_provider_available Tests ====================

    #[test]
    fn test_deepgram_available_with_key() {
        let result = is_provider_available(
            &TranscriptionProvider::Deepgram,
            Some("dg_key_123"),
            None,
            false,
        );
        assert!(result);
    }

    #[test]
    fn test_deepgram_unavailable_without_key() {
        let result = is_provider_available(
            &TranscriptionProvider::Deepgram,
            None,
            None,
            false,
        );
        assert!(!result);
    }

    #[test]
    fn test_whisper_api_available_with_license() {
        let result = is_provider_available(
            &TranscriptionProvider::WhisperApi,
            None,
            Some("license_key"),
            false,
        );
        assert!(result);
    }

    #[test]
    fn test_whisper_api_unavailable_without_license() {
        let result = is_provider_available(
            &TranscriptionProvider::WhisperApi,
            Some("dg_key"),
            None, // No license
            false,
        );
        assert!(!result);
    }

    #[test]
    fn test_whisper_local_available_with_license_and_model() {
        let result = is_provider_available(
            &TranscriptionProvider::WhisperLocal,
            None,
            Some("license_key"),
            true, // Model downloaded
        );
        assert!(result);
    }

    #[test]
    fn test_whisper_local_unavailable_without_model() {
        let result = is_provider_available(
            &TranscriptionProvider::WhisperLocal,
            None,
            Some("license_key"),
            false, // Model not downloaded
        );
        assert!(!result);
    }

    #[test]
    fn test_whisper_local_unavailable_without_license() {
        let result = is_provider_available(
            &TranscriptionProvider::WhisperLocal,
            None,
            None, // No license
            true,
        );
        assert!(!result);
    }

    // ==================== get_best_provider Tests ====================

    #[test]
    fn test_subscription_takes_highest_priority() {
        let result = get_best_provider(
            true,  // has_subscription
            true,  // has_lifetime
            true,  // model_downloaded
            Some("dg_key"),
        );
        assert_eq!(result, Some(TranscriptionProvider::WhisperApi));
    }

    #[test]
    fn test_lifetime_with_model_second_priority() {
        let result = get_best_provider(
            false, // no subscription
            true,  // has_lifetime
            true,  // model_downloaded
            Some("dg_key"),
        );
        assert_eq!(result, Some(TranscriptionProvider::WhisperLocal));
    }

    #[test]
    fn test_lifetime_without_model_falls_to_deepgram() {
        let result = get_best_provider(
            false, // no subscription
            true,  // has_lifetime
            false, // model NOT downloaded
            Some("dg_key"),
        );
        assert_eq!(result, Some(TranscriptionProvider::Deepgram));
    }

    #[test]
    fn test_deepgram_is_fallback() {
        let result = get_best_provider(
            false, // no subscription
            false, // no lifetime
            false, // no model
            Some("dg_key"),
        );
        assert_eq!(result, Some(TranscriptionProvider::Deepgram));
    }

    #[test]
    fn test_no_provider_available() {
        let result = get_best_provider(
            false, // no subscription
            false, // no lifetime
            false, // no model
            None,  // no deepgram key
        );
        assert_eq!(result, None);
    }

    #[test]
    fn test_subscription_only() {
        let result = get_best_provider(
            true,  // has_subscription only
            false,
            false,
            None,
        );
        assert_eq!(result, Some(TranscriptionProvider::WhisperApi));
    }

    #[test]
    fn test_lifetime_only_with_model() {
        let result = get_best_provider(
            false,
            true,  // lifetime only
            true,  // with model
            None,
        );
        assert_eq!(result, Some(TranscriptionProvider::WhisperLocal));
    }

    #[test]
    fn test_lifetime_only_without_model_no_fallback() {
        let result = get_best_provider(
            false,
            true,  // lifetime only
            false, // without model
            None,  // no deepgram
        );
        // Lifetime without model and no deepgram = no provider
        assert_eq!(result, None);
    }

    // ==================== TranscriptionResult Tests ====================

    #[test]
    fn test_transcription_result_creation() {
        let result = TranscriptionResult {
            transcript: "Hello world".to_string(),
            provider: TranscriptionProvider::Deepgram,
        };
        assert_eq!(result.transcript, "Hello world");
        assert_eq!(result.provider, TranscriptionProvider::Deepgram);
    }
}
