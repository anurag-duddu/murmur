//! Built-in style definitions.
//!
//! These styles are always available and cover common use cases.

use super::{AutoFeatures, Style};

/// Get all built-in styles.
pub fn get_all_styles() -> Vec<Style> {
    vec![
        casual(),
        professional(),
        neutral(),
        technical(),
        creative(),
    ]
}

/// Get the default style (neutral).
pub fn get_default_style() -> Style {
    neutral()
}

/// Get a built-in style by ID.
pub fn get_style_by_id(id: &str) -> Option<Style> {
    match id {
        "casual" => Some(casual()),
        "professional" => Some(professional()),
        "neutral" => Some(neutral()),
        "technical" => Some(technical()),
        "creative" => Some(creative()),
        _ => None,
    }
}

/// Casual style for messaging apps (Slack, Discord, Messages).
/// Formatting: lowercase start, skip trailing periods, relaxed punctuation.
pub fn casual() -> Style {
    Style {
        id: "casual".to_string(),
        name: "Casual".to_string(),
        description: "Relaxed formatting for messaging apps".to_string(),
        prompt_modifier: r#"Formatting style: Casual (messaging apps)
- Start sentences with lowercase (unless proper noun)
- Skip the trailing period on single sentences
- Keep contractions as-is
- DO NOT rewrite sentences or change the meaning
- Only fix obvious errors, remove filler words, handle corrections"#.to_string(),
        auto_features: AutoFeatures {
            lowercase_start: true,
            allow_contractions: true,
            remove_periods_single_sentence: true,
        },
    }
}

/// Professional style for email and business communication.
/// Formatting: proper capitalization, full punctuation.
pub fn professional() -> Style {
    Style {
        id: "professional".to_string(),
        name: "Professional".to_string(),
        description: "Proper formatting for business communication".to_string(),
        prompt_modifier: r#"Formatting style: Professional (email, business)
- Proper sentence capitalization
- Include all punctuation (periods, commas)
- Keep contractions (modern business writing)
- DO NOT rewrite sentences or change the meaning
- Only fix obvious errors, remove filler words, handle corrections"#.to_string(),
        auto_features: AutoFeatures {
            lowercase_start: false,
            allow_contractions: true,
            remove_periods_single_sentence: false,
        },
    }
}

/// Neutral style - minimal transformation, used as default.
pub fn neutral() -> Style {
    Style {
        id: "neutral".to_string(),
        name: "Neutral".to_string(),
        description: "Clean formatting with minimal changes".to_string(),
        prompt_modifier: r#"Formatting style: Neutral (default)
- Standard capitalization and punctuation
- Preserve the speaker's exact wording
- DO NOT rewrite sentences or change the meaning
- Only fix obvious errors, remove filler words, handle corrections"#.to_string(),
        auto_features: AutoFeatures {
            lowercase_start: false,
            allow_contractions: true,
            remove_periods_single_sentence: false,
        },
    }
}

/// Technical style for terminals and code editors.
/// Preserves technical terminology exactly as spoken.
pub fn technical() -> Style {
    Style {
        id: "technical".to_string(),
        name: "Technical".to_string(),
        description: "Preserves technical terms for development".to_string(),
        prompt_modifier: r#"Formatting style: Technical (code editors, terminals)
- Preserve technical terms exactly: API, JSON, npm, git, CLI, etc.
- Preserve casing styles: camelCase, snake_case, kebab-case, PascalCase
- Don't "correct" technical terms that look like typos
- DO NOT rewrite sentences or change the meaning
- Only fix obvious errors, remove filler words, handle corrections"#.to_string(),
        auto_features: AutoFeatures {
            lowercase_start: false,
            allow_contractions: true,
            remove_periods_single_sentence: false,
        },
    }
}

/// Creative style for writing apps.
/// Standard formatting, preserves natural voice.
pub fn creative() -> Style {
    Style {
        id: "creative".to_string(),
        name: "Creative".to_string(),
        description: "Standard formatting for writing apps".to_string(),
        prompt_modifier: r#"Formatting style: Creative (writing apps)
- Standard capitalization and punctuation
- Preserve the speaker's natural voice and word choices
- Allow varied sentence lengths
- DO NOT rewrite sentences or change the meaning
- Only fix obvious errors, remove filler words, handle corrections"#.to_string(),
        auto_features: AutoFeatures {
            lowercase_start: false,
            allow_contractions: true,
            remove_periods_single_sentence: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_styles_have_unique_ids() {
        let styles = get_all_styles();
        let mut ids: Vec<&str> = styles.iter().map(|s| s.id.as_str()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), styles.len(), "Style IDs must be unique");
    }

    #[test]
    fn test_get_style_by_id() {
        assert!(get_style_by_id("casual").is_some());
        assert!(get_style_by_id("professional").is_some());
        assert!(get_style_by_id("neutral").is_some());
        assert!(get_style_by_id("technical").is_some());
        assert!(get_style_by_id("creative").is_some());
        assert!(get_style_by_id("nonexistent").is_none());
    }

    #[test]
    fn test_default_style_is_neutral() {
        let default = get_default_style();
        assert_eq!(default.id, "neutral");
    }
}
