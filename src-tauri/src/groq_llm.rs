//! Groq LLM client for text transformation and enhancement.
//!
//! Uses the Groq API with OpenAI-compatible chat completions format.
//! Model: llama-3.3-70b-versatile (free tier, 128K context)

use crate::http_client;
use crate::rate_limit::{check_rate_limit, Service};
use reqwest::Client;
use serde::{Deserialize, Serialize};

const GROQ_CHAT_URL: &str = "https://api.groq.com/openai/v1/chat/completions";
const GROQ_MODEL: &str = "llama-3.3-70b-versatile";

/// User intent when text is selected
#[derive(Debug, Clone, PartialEq)]
pub enum UserIntent {
    /// User wants to transform/modify the selected text
    Command,
    /// User wants to replace the selection with new dictated content
    Dictation,
}

/// System prompt for intent classification
const INTENT_SYSTEM_PROMPT: &str = r#"You are an intent classifier. Determine if the user's speech is a COMMAND to modify existing text, or new CONTENT to replace the selection.

COMMAND indicators (user wants to transform the selected text):
- Imperative verbs: "make", "fix", "translate", "summarize", "shorten", "expand", "rewrite", "convert", "format"
- References to the text: "this", "it", "the text", "the selection", "that"
- Modification requests: "more formal", "less verbose", "in Spanish", "as bullets"

CONTENT indicators (user wants to dictate new text to replace selection):
- Complete sentences or phrases that stand alone
- No reference to modifying existing text
- New information being stated
- Sounds like someone composing fresh text

Examples:
- "make this shorter" → COMMAND
- "translate to French" → COMMAND
- "fix the grammar" → COMMAND
- "Hello, how are you today" → CONTENT
- "The meeting is scheduled for 3pm" → CONTENT
- "Remember to buy milk" → CONTENT
- "more professional" → COMMAND
- "I'll send the report tomorrow" → CONTENT

Respond with ONLY one word: COMMAND or CONTENT"#;

/// Strip accidental quote/code fence wrapping from LLM output
fn strip_wrapping(s: &str) -> String {
    let mut result = s.trim();

    // Strip triple quotes (Python-style)
    if result.starts_with("\"\"\"") && result.ends_with("\"\"\"") {
        result = result[3..result.len()-3].trim();
    }

    // Strip code fences with optional language
    if result.starts_with("```") {
        // Find end of first line (language specifier)
        if let Some(end_first_line) = result.find('\n') {
            let after_fence = &result[end_first_line+1..];
            if let Some(closing_idx) = after_fence.rfind("```") {
                result = after_fence[..closing_idx].trim();
            }
        }
    }

    // Strip single surrounding quotes (double or single)
    if (result.starts_with('"') && result.ends_with('"')) ||
       (result.starts_with('\'') && result.ends_with('\'')) {
        if result.len() > 2 {
            result = &result[1..result.len()-1];
        }
    }

    result.to_string()
}

/// System prompt for Command Mode transformations
const TRANSFORM_SYSTEM_PROMPT: &str = r#"You are a text transformation assistant. Transform the selected text according to the user's voice command.

Rules:
- Output ONLY the transformed text, nothing else
- Do NOT wrap output in quotes, triple quotes, code fences, or any other formatting markers
- Do not add explanations, acknowledgments, or meta-commentary
- Preserve the original meaning unless explicitly asked to change it
- Match the original formatting style (markdown, plain text, code, etc.)
- For translations, output only the translated text
- For formatting commands (bullets, numbered list), apply the formatting
- If the command is unclear, make a reasonable interpretation"#;

/// System prompt for Dictation Mode enhancement
const ENHANCE_SYSTEM_PROMPT: &str = r#"You are a speech-to-text enhancement assistant. Clean up and improve the transcription.

Instructions:
1. Remove filler words (um, uh, like, you know, etc.)
2. Fix grammar and punctuation
3. Handle course corrections - when the speaker changes their mind mid-sentence:
   - Correction signals: "no", "no no", "no wait", "actually", "scratch that", "I mean", "wait", "sorry"
   - When you see a correction signal, DISCARD everything before it and keep ONLY what comes after
   - For chained corrections like "5 pm, no 6 pm, actually 7 pm" → output ONLY "7 pm" (the final value)
   - Example: "Let's meet at 5 pm. No, no, 6 pm. Actually, you know what? Let's meet at 7 pm tomorrow." → "Let's meet at 7 pm tomorrow."
   - "delete that", "never mind" → remove the entire preceding clause
4. Preserve the speaker's FINAL intent exactly (after all corrections are applied)
5. Do NOT add information that wasn't in the original
6. PRESERVE all @-prefixed references exactly as-is (e.g., @components.json, @main.rs, @UserService) - these are intentional file/symbol tags
7. Output ONLY the enhanced text, nothing else"#;

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Debug, Deserialize)]
struct MessageContent {
    content: String,
}

pub struct GroqLlmClient {
    api_key: String,
    client: &'static Client,
}

impl GroqLlmClient {
    pub fn new(api_key: String) -> Result<Self, String> {
        // Use cached client for connection reuse
        Ok(GroqLlmClient {
            api_key,
            client: http_client::get_client()?,
        })
    }

    /// Get the API key from environment or stored key
    fn get_api_key(&self) -> Result<String, String> {
        // First check environment variable
        if let Ok(key) = std::env::var("GROQ_API_KEY") {
            if !key.is_empty() {
                return Ok(key);
            }
        }

        // Use the provided API key
        if !self.api_key.is_empty() {
            Ok(self.api_key.clone())
        } else {
            Err("No Groq API key configured. Set GROQ_API_KEY environment variable.".to_string())
        }
    }

    /// Classify user intent: is this a command to transform text, or new content?
    ///
    /// This is called when text is selected to determine if the user wants to:
    /// - Transform the selection (Command Mode) - e.g., "make this shorter"
    /// - Replace the selection with new dictation (Dictation Mode) - e.g., "Hello world"
    ///
    /// # Arguments
    /// * `transcription` - The user's spoken words
    ///
    /// # Returns
    /// UserIntent::Command or UserIntent::Dictation
    pub async fn classify_intent(&self, transcription: &str) -> Result<UserIntent, String> {
        // Check rate limit before making API call
        check_rate_limit(Service::Groq)?;

        #[cfg(debug_assertions)]
        println!("Classifying intent for: {}", transcription);

        let api_key = self.get_api_key()?;

        let request = ChatRequest {
            model: GROQ_MODEL.to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: INTENT_SYSTEM_PROMPT.to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: format!("User speech: \"{}\"", transcription),
                },
            ],
            temperature: 0.0, // Deterministic for classification
            max_tokens: 10,   // Only need one word
        };

        let result = self.send_request(&api_key, &request).await?;
        let result_upper = result.to_uppercase();

        let intent = if result_upper.contains("COMMAND") {
            UserIntent::Command
        } else {
            // Default to Dictation if unclear - safer for user experience
            UserIntent::Dictation
        };

        println!("Intent classified as: {:?}", intent);
        Ok(intent)
    }

    /// Transform selected text using a voice command (Command Mode)
    ///
    /// # Arguments
    /// * `selected_text` - The text that was selected by the user
    /// * `command` - The voice command describing the transformation
    ///
    /// # Returns
    /// The transformed text
    pub async fn transform_text(
        &self,
        selected_text: &str,
        command: &str,
    ) -> Result<String, String> {
        // Check rate limit before making API call
        check_rate_limit(Service::Groq)?;

        #[cfg(debug_assertions)]
        {
            println!("Transforming text with Groq LLM...");
            println!("Command: {}", command);
        }

        let api_key = self.get_api_key()?;

        let user_message = format!(
            "SELECTED TEXT:\n\"\"\"\n{}\n\"\"\"\n\nCOMMAND: \"{}\"",
            selected_text, command
        );

        let request = ChatRequest {
            model: GROQ_MODEL.to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: TRANSFORM_SYSTEM_PROMPT.to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: user_message,
                },
            ],
            temperature: 0.3,
            max_tokens: 4096,
        };

        self.send_request(&api_key, &request).await
    }

    /// Enhance a transcription for Dictation Mode
    ///
    /// # Arguments
    /// * `transcript` - The raw transcription from speech-to-text
    /// * `style_prompt` - Optional style guidance (e.g., "casual", "professional")
    ///
    /// # Returns
    /// The enhanced text
    pub async fn enhance_text(
        &self,
        transcript: &str,
        style_prompt: Option<&str>,
    ) -> Result<String, String> {
        // Check rate limit before making API call
        check_rate_limit(Service::Groq)?;

        println!("Enhancing text with Groq LLM...");

        let api_key = self.get_api_key()?;

        // Build system prompt with optional style guidance
        let system_prompt = match style_prompt {
            Some(style) => format!(
                "{}\n\nStyle guidance: {}",
                ENHANCE_SYSTEM_PROMPT, style
            ),
            None => ENHANCE_SYSTEM_PROMPT.to_string(),
        };

        let request = ChatRequest {
            model: GROQ_MODEL.to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                Message {
                    role: "user".to_string(),
                    content: format!("Transcription: \"{}\"", transcript),
                },
            ],
            temperature: 0.3,
            max_tokens: 4096,
        };

        self.send_request(&api_key, &request).await
    }

    /// Send a request to the Groq API
    async fn send_request(
        &self,
        api_key: &str,
        request: &ChatRequest,
    ) -> Result<String, String> {
        let response = self
            .client
            .post(GROQ_CHAT_URL)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(request)
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

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Groq response: {}", e))?;

        let result = chat_response
            .choices
            .first()
            .map(|c| c.message.content.trim().to_string())
            .ok_or("No response content from Groq")?;

        // Clean up any accidental quote/code fence wrapping from LLM
        let cleaned = strip_wrapping(&result);

        #[cfg(debug_assertions)]
        println!("Groq LLM result: {}", cleaned);

        Ok(cleaned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompts_not_empty() {
        assert!(!TRANSFORM_SYSTEM_PROMPT.is_empty());
        assert!(!ENHANCE_SYSTEM_PROMPT.is_empty());
    }
}
