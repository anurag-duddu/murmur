use crate::http_client;
use crate::rate_limit::{check_rate_limit, Service};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
}

#[derive(Debug, Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Debug, Deserialize)]
struct ClaudeContent {
    text: String,
}

pub struct ClaudeClient {
    api_key: String,
    model: String,
    client: &'static Client,
}

impl ClaudeClient {
    pub fn new(api_key: String, model: Option<String>) -> Result<Self, String> {
        // Use cached client for connection reuse
        Ok(ClaudeClient {
            api_key,
            model: model.unwrap_or_else(|| "claude-3-5-sonnet-20241022".to_string()),
            client: http_client::get_client()?,
        })
    }

    pub async fn enhance_text(
        &self,
        transcript: &str,
        app_context: Option<&str>,
    ) -> Result<String, String> {
        // Check rate limit before making API call
        check_rate_limit(Service::Claude)?;

        println!("Enhancing text with Claude...");

        let url = "https://api.anthropic.com/v1/messages";

        // Build the enhancement prompt
        let context_info = app_context.unwrap_or("general text");
        let prompt = format!(
            r#"You are a text enhancement assistant. Your task is to clean up and improve the following voice transcription.

Context: The user is writing in {context}

Raw transcription: "{transcript}"

Instructions:
1. Remove any remaining filler words (um, uh, like, you know, etc.)
2. Fix grammar and improve clarity
3. Handle course corrections - if the user says "no wait", "actually", "scratch that", "I mean", or similar phrases, only keep what comes after
4. Format appropriately for the context:
   - Email/Professional: Formal tone, proper punctuation, paragraphs
   - Messaging/Chat: Casual, conversational, shorter sentences
   - Code/Technical: Preserve technical terms exactly, code-like formatting
   - Notes/Documents: Clear, structured, proper paragraphs
5. Preserve the user's intent and meaning exactly
6. Do NOT add information that wasn't in the original transcription
7. Output ONLY the enhanced text, nothing else - no explanations, no meta-commentary

Enhanced text:"#,
            context = context_info,
            transcript = transcript
        );

        let request_body = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 1024,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: prompt,
            }],
        };

        let response = self
            .client
            .post(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to Claude: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Claude API error ({}): {}", status, error_text));
        }

        let claude_response: ClaudeResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Claude response: {}", e))?;

        let enhanced_text = claude_response
            .content
            .get(0)
            .map(|content| content.text.trim().to_string())
            .ok_or("No enhanced text found in Claude response")?;

        #[cfg(debug_assertions)]
        println!("Claude enhanced text: {}", enhanced_text);

        Ok(enhanced_text)
    }
}