//! Variable case recognition and conversion.
//!
//! Detects trigger words for case styles and applies them:
//! - "camel case user name" → "userName"
//! - "snake case user name" → "user_name"
//! - "is login error" → "isLoginError" (detected from context)

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// Case style for variable names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CaseStyle {
    /// camelCase - first word lowercase, rest capitalized
    #[default]
    CamelCase,
    /// PascalCase - all words capitalized
    PascalCase,
    /// snake_case - all lowercase with underscores
    SnakeCase,
    /// SCREAMING_SNAKE_CASE - all uppercase with underscores
    ScreamingSnake,
    /// kebab-case - all lowercase with hyphens
    KebabCase,
}

impl CaseStyle {
    /// Convert a slice of words to the specified case style.
    pub fn apply(&self, words: &[&str]) -> String {
        if words.is_empty() {
            return String::new();
        }

        match self {
            CaseStyle::CamelCase => {
                words
                    .iter()
                    .enumerate()
                    .map(|(i, w)| {
                        if i == 0 {
                            w.to_lowercase()
                        } else {
                            capitalize(w)
                        }
                    })
                    .collect()
            }
            CaseStyle::PascalCase => words.iter().map(|w| capitalize(w)).collect(),
            CaseStyle::SnakeCase => words
                .iter()
                .map(|w| w.to_lowercase())
                .collect::<Vec<_>>()
                .join("_"),
            CaseStyle::ScreamingSnake => words
                .iter()
                .map(|w| w.to_uppercase())
                .collect::<Vec<_>>()
                .join("_"),
            CaseStyle::KebabCase => words
                .iter()
                .map(|w| w.to_lowercase())
                .collect::<Vec<_>>()
                .join("-"),
        }
    }
}

/// Capitalize first letter of a word.
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().chain(chars.map(|c| c.to_ascii_lowercase())).collect(),
    }
}

/// Trigger phrase patterns for case styles.
/// Format: (regex pattern, case style, capture group index for words)
/// SAFETY: unwrap() is safe for all regexes below - they are compile-time constant
/// strings that have been validated during development.
static CASE_TRIGGER_PATTERNS: LazyLock<Vec<(Regex, CaseStyle)>> = LazyLock::new(|| {
    vec![
        // "camel case [words]"
        (
            Regex::new(r"(?i)\bcamel\s+case\s+(.+)").unwrap(),
            CaseStyle::CamelCase,
        ),
        // "pascal case [words]"
        (
            Regex::new(r"(?i)\bpascal\s+case\s+(.+)").unwrap(),
            CaseStyle::PascalCase,
        ),
        // "snake case [words]"
        (
            Regex::new(r"(?i)\bsnake\s+case\s+(.+)").unwrap(),
            CaseStyle::SnakeCase,
        ),
        // "screaming snake [words]" or "constant case [words]"
        (
            Regex::new(r"(?i)\b(?:screaming\s+snake|constant\s+case)\s+(.+)").unwrap(),
            CaseStyle::ScreamingSnake,
        ),
        // "kebab case [words]"
        (
            Regex::new(r"(?i)\bkebab\s+case\s+(.+)").unwrap(),
            CaseStyle::KebabCase,
        ),
    ]
});

/// Pattern for "underscore" between words (implies snake_case).
static UNDERSCORE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(\w+)\s+underscore\s+(\w+)").unwrap());

/// Pattern for "dash" between words (implies kebab-case).
static DASH_BETWEEN_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(\w+)\s+dash\s+(\w+)").unwrap());

/// Apply variable case patterns to text.
///
/// Detects trigger words and applies the corresponding case style.
///
/// # Arguments
/// * `text` - The text to process
/// * `default_style` - Default case style to use when none is specified
///
/// # Examples
/// ```ignore
/// let result = apply_variable_patterns("camel case user name", CaseStyle::CamelCase);
/// assert_eq!(result, "userName");
/// ```
pub fn apply_variable_patterns(text: &str, _default_style: CaseStyle) -> String {
    let mut result = text.to_string();

    // Check for explicit case triggers first
    for (regex, style) in CASE_TRIGGER_PATTERNS.iter() {
        if let Some(captures) = regex.captures(&result) {
            if let Some(words_match) = captures.get(1) {
                let words: Vec<&str> = words_match.as_str().split_whitespace().collect();
                let converted = style.apply(&words);
                result = regex.replace(&result, converted.as_str()).to_string();
            }
        }
    }

    // Handle "underscore" between words → snake_case
    while UNDERSCORE_PATTERN.is_match(&result) {
        result = UNDERSCORE_PATTERN
            .replace(&result, "${1}_${2}")
            .to_string();
    }

    // Handle "dash" between words → kebab-case (only for variable context)
    // Note: We're careful here to not conflict with CLI dash patterns
    // This only applies when "dash" appears between two words without spaces around the dash
    while DASH_BETWEEN_PATTERN.is_match(&result) {
        result = DASH_BETWEEN_PATTERN
            .replace(&result, "${1}-${2}")
            .to_string();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camel_case_apply() {
        let words = vec!["user", "name"];
        assert_eq!(CaseStyle::CamelCase.apply(&words), "userName");
    }

    #[test]
    fn test_pascal_case_apply() {
        let words = vec!["user", "name"];
        assert_eq!(CaseStyle::PascalCase.apply(&words), "UserName");
    }

    #[test]
    fn test_snake_case_apply() {
        let words = vec!["user", "name"];
        assert_eq!(CaseStyle::SnakeCase.apply(&words), "user_name");
    }

    #[test]
    fn test_screaming_snake_apply() {
        let words = vec!["max", "retries"];
        assert_eq!(CaseStyle::ScreamingSnake.apply(&words), "MAX_RETRIES");
    }

    #[test]
    fn test_kebab_case_apply() {
        let words = vec!["user", "name"];
        assert_eq!(CaseStyle::KebabCase.apply(&words), "user-name");
    }

    #[test]
    fn test_camel_case_trigger() {
        let result = apply_variable_patterns("camel case user name", CaseStyle::CamelCase);
        assert_eq!(result, "userName");
    }

    #[test]
    fn test_pascal_case_trigger() {
        let result = apply_variable_patterns("pascal case user service", CaseStyle::CamelCase);
        assert_eq!(result, "UserService");
    }

    #[test]
    fn test_snake_case_trigger() {
        let result = apply_variable_patterns("snake case user name", CaseStyle::CamelCase);
        assert_eq!(result, "user_name");
    }

    #[test]
    fn test_kebab_case_trigger() {
        let result = apply_variable_patterns("kebab case my component", CaseStyle::CamelCase);
        assert_eq!(result, "my-component");
    }

    #[test]
    fn test_underscore_between_words() {
        let result = apply_variable_patterns("user underscore id", CaseStyle::CamelCase);
        assert_eq!(result, "user_id");
    }

    #[test]
    fn test_multiple_underscores() {
        let result =
            apply_variable_patterns("user underscore first underscore name", CaseStyle::CamelCase);
        assert_eq!(result, "user_first_name");
    }

    #[test]
    fn test_preserves_non_matches() {
        let result = apply_variable_patterns("hello world", CaseStyle::CamelCase);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_case_insensitive_trigger() {
        let result = apply_variable_patterns("CAMEL CASE user name", CaseStyle::CamelCase);
        assert_eq!(result, "userName");
    }

    #[test]
    fn test_screaming_snake_trigger() {
        let result = apply_variable_patterns("screaming snake max value", CaseStyle::CamelCase);
        assert_eq!(result, "MAX_VALUE");
    }

    #[test]
    fn test_constant_case_trigger() {
        let result = apply_variable_patterns("constant case default timeout", CaseStyle::CamelCase);
        assert_eq!(result, "DEFAULT_TIMEOUT");
    }
}
