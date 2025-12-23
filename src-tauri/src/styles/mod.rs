//! Context-aware styles for dictation enhancement.
//!
//! This module provides automatic style selection based on the active application.
//! Styles adjust the tone and formatting of dictated text to match context.

pub mod builtin;
pub mod detection;
pub mod mapping;

use serde::{Deserialize, Serialize};

/// A style definition that controls how dictation is enhanced.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Style {
    /// Unique identifier (e.g., "casual", "professional")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of the style
    pub description: String,
    /// LLM prompt modifier that describes the style
    pub prompt_modifier: String,
    /// Automatic formatting features
    #[serde(default)]
    pub auto_features: AutoFeatures,
}

/// Automatic formatting features applied after LLM enhancement.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AutoFeatures {
    /// Start with lowercase (for casual messaging)
    #[serde(default)]
    pub lowercase_start: bool,
    /// Allow contractions (don't vs do not)
    #[serde(default)]
    pub allow_contractions: bool,
    /// Remove trailing period for single sentences
    #[serde(default)]
    pub remove_periods_single_sentence: bool,
}

/// Information about the currently active application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveApp {
    /// Bundle identifier (e.g., "com.apple.mail")
    pub bundle_id: String,
    /// Localized app name (e.g., "Mail")
    pub name: String,
}

impl Style {
    /// Get the style's prompt modifier for the LLM.
    pub fn get_prompt_modifier(&self) -> &str {
        &self.prompt_modifier
    }
}

/// Get the currently active application.
pub fn get_active_app() -> Option<ActiveApp> {
    detection::get_active_app()
}

/// Get the appropriate style for an application.
pub fn get_style_for_app(app: &ActiveApp) -> Style {
    mapping::get_style_for_app(app)
}

/// Get the style for the currently active application.
pub fn get_current_style() -> Style {
    match get_active_app() {
        Some(app) => get_style_for_app(&app),
        None => builtin::get_default_style(),
    }
}

/// Get all available built-in styles.
pub fn get_all_styles() -> Vec<Style> {
    builtin::get_all_styles()
}

/// Get the default style (used when app is not recognized).
pub fn get_default_style() -> Style {
    builtin::get_default_style()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_prompt_modifier() {
        let style = Style {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test style".to_string(),
            prompt_modifier: "Be concise".to_string(),
            auto_features: AutoFeatures::default(),
        };
        assert_eq!(style.get_prompt_modifier(), "Be concise");
    }
}
