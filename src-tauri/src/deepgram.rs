use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeepgramResponse {
    pub results: DeepgramResults,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeepgramResults {
    pub channels: Vec<DeepgramChannel>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeepgramChannel {
    pub alternatives: Vec<DeepgramAlternative>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeepgramAlternative {
    pub transcript: String,
    pub confidence: f64,
}

pub struct DeepgramClient {
    api_key: String,
    language: String,
    client: Client,
}

impl DeepgramClient {
    pub fn new(api_key: String, language: Option<String>) -> Self {
        DeepgramClient {
            api_key,
            language: language.unwrap_or_else(|| "en-US".to_string()),
            client: Client::new(),
        }
    }

    pub async fn transcribe_audio(&self, audio_data: Vec<u8>) -> Result<String, String> {
        let url = "https://api.deepgram.com/v1/listen";

        // Build query parameters based on language setting
        // For "auto" or "multi" mode, use detect_language=true for automatic language detection
        // Note: Auto-detect works better with longer audio samples (5+ seconds)
        // Nova-3 supports true multilingual transcription with "multi" language code
        // Supported in multi mode: English, Spanish, French, German, Hindi, Russian, Portuguese, Japanese, Italian, Dutch
        let params: Vec<(&str, &str)> = if self.language == "auto" || self.language == "multi" {
            println!("Sending audio to Deepgram Nova-3 for transcription (multilingual mode)...");
            vec![
                ("model", "nova-3"),
                ("smart_format", "true"),
                ("punctuate", "true"),
                ("language", "multi"),
            ]
        } else {
            println!("Sending audio to Deepgram Nova-3 for transcription (language: {})...", self.language);
            vec![
                ("model", "nova-3"),
                ("smart_format", "true"),
                ("punctuate", "true"),
                ("language", self.language.as_str()),
            ]
        };

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Token {}", self.api_key))
            .header("Content-Type", "audio/wav")
            .query(&params)
            .body(audio_data)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to Deepgram: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!(
                "Deepgram API error ({}): {}",
                status, error_text
            ));
        }

        let deepgram_response: DeepgramResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Deepgram response: {}", e))?;

        // Extract transcript from response
        let transcript = deepgram_response
            .results
            .channels
            .get(0)
            .and_then(|channel| channel.alternatives.get(0))
            .map(|alt| alt.transcript.clone())
            .ok_or("No transcript found in Deepgram response")?;

        println!("Deepgram transcript: {}", transcript);

        Ok(transcript)
    }
}