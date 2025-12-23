//! CLI syntax pattern recognition.
//!
//! Converts spoken CLI patterns to their correct syntax:
//! - "dash dash verbose" → "--verbose"
//! - "pipe" → "|"
//! - "and and" → "&&"

use regex::Regex;
use std::sync::LazyLock;

/// CLI syntax patterns: (spoken pattern, correct form)
const CLI_PATTERNS: &[(&str, &str)] = &[
    // Double dash options (must come before single dash)
    ("dash dash", "--"),
    ("double dash", "--"),
    ("hyphen hyphen", "--"),

    // Single dash options (handled by DASH_LETTER_PATTERN for "dash [letter]")
    // Removed "dash " and "hyphen " with trailing spaces - they don't match with \b word boundaries
    // and are better handled by the specialized DASH_LETTER_PATTERN regex

    // Operators
    ("pipe", "|"),
    ("and and", "&&"),
    ("ampersand ampersand", "&&"),
    ("or or", "||"),
    ("greater than", ">"),
    ("less than", "<"),
    ("greater than greater than", ">>"),
    ("append", ">>"),
    ("redirect", ">"),

    // Special characters
    ("backtick", "`"),
    ("back tick", "`"),
    ("tilde", "~"),
    ("at sign", "@"),
    ("at symbol", "@"),
    ("hash", "#"),
    ("pound", "#"),
    ("dollar sign", "$"),
    ("dollar", "$"),
    ("percent", "%"),
    ("caret", "^"),
    ("asterisk", "*"),
    ("star", "*"),
    ("ampersand", "&"),
    ("semicolon", ";"),
    ("colon", ":"),

    // Paths and slashes - LONGER PATTERNS FIRST
    ("forward slash", "/"),
    ("backslash", "\\"),
    ("back slash", "\\"),
    ("slash", "/"),

    // Quotes
    ("double quote", "\""),
    ("single quote", "'"),
    ("quote", "\""),

    // Brackets
    ("open paren", "("),
    ("close paren", ")"),
    ("open bracket", "["),
    ("close bracket", "]"),
    ("open brace", "{"),
    ("close brace", "}"),
    ("open curly", "{"),
    ("close curly", "}"),
    ("open angle", "<"),
    ("close angle", ">"),

    // Common CLI words
    ("sudo", "sudo"),
    ("npm run", "npm run"),
    ("npm install", "npm install"),
    ("yarn add", "yarn add"),
    ("cargo run", "cargo run"),
    ("cargo build", "cargo build"),
    ("cargo test", "cargo test"),
    ("python", "python"),
    ("python3", "python3"),
    ("pip install", "pip install"),
];

/// Word-to-symbol mappings for CLI options.
/// These are applied after the main pattern replacement.
const CLI_OPTION_SHORTCUTS: &[(&str, &str)] = &[
    // Common single-letter options
    ("verbose", "v"),
    ("help", "h"),
    ("version", "V"),
    ("recursive", "r"),
    ("force", "f"),
    ("all", "a"),
    ("long", "l"),
    ("quiet", "q"),
    ("silent", "s"),
    ("interactive", "i"),
    ("yes", "y"),
    ("no", "n"),
];

/// Compiled regex patterns for CLI replacement.
static CLI_REGEX_PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    CLI_PATTERNS
        .iter()
        .filter_map(|(pattern, replacement)| {
            // Case-insensitive matching with word boundaries
            // Use \b at start and end to prevent partial word matches
            let escaped = regex::escape(pattern);
            let regex_pattern = format!(r"(?i)\b{}\b", escaped);
            Regex::new(&regex_pattern)
                .ok()
                .map(|re| (re, *replacement))
        })
        .collect()
});

/// Pattern for "dash [letter]" → "-[letter]"
static DASH_LETTER_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bdash\s+([a-z])\b").unwrap()
});

/// Pattern for "dash dash [word]" → "--[word]"
static DASH_DASH_WORD_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bdash\s+dash\s+(\w+)").unwrap()
});

/// Pattern for "double dash [word]" → "--[word]"
static DOUBLE_DASH_WORD_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bdouble\s+dash\s+(\w+)").unwrap()
});

/// Apply CLI syntax patterns to text.
///
/// Converts spoken CLI patterns to their correct syntax.
///
/// # Examples
/// ```ignore
/// let result = apply_cli_patterns("git commit dash m fix typo");
/// assert_eq!(result, "git commit -m fix typo");
/// ```
pub fn apply_cli_patterns(text: &str) -> String {
    let mut result = text.to_string();

    // Apply "dash dash [word]" → "--[word]" first (more specific)
    result = DASH_DASH_WORD_PATTERN
        .replace_all(&result, "--$1")
        .to_string();

    // Apply "double dash [word]" → "--[word]"
    result = DOUBLE_DASH_WORD_PATTERN
        .replace_all(&result, "--$1")
        .to_string();

    // Apply "dash [letter]" → "-[letter]"
    result = DASH_LETTER_PATTERN
        .replace_all(&result, "-$1")
        .to_string();

    // Apply static patterns
    for (regex, replacement) in CLI_REGEX_PATTERNS.iter() {
        result = regex.replace_all(&result, *replacement).to_string();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_dash_letter() {
        assert_eq!(apply_cli_patterns("ls dash l"), "ls -l");
        assert_eq!(apply_cli_patterns("git commit dash m"), "git commit -m");
    }

    #[test]
    fn test_double_dash_word() {
        assert_eq!(apply_cli_patterns("npm dash dash version"), "npm --version");
        assert_eq!(apply_cli_patterns("cargo dash dash release"), "cargo --release");
    }

    #[test]
    fn test_double_dash_word_alt() {
        assert_eq!(apply_cli_patterns("npm double dash version"), "npm --version");
    }

    #[test]
    fn test_pipe() {
        assert_eq!(apply_cli_patterns("ls pipe grep foo"), "ls | grep foo");
    }

    #[test]
    fn test_and_and() {
        assert_eq!(apply_cli_patterns("mkdir foo and and cd foo"), "mkdir foo && cd foo");
    }

    #[test]
    fn test_redirect() {
        assert_eq!(apply_cli_patterns("echo hello greater than file.txt"), "echo hello > file.txt");
    }

    #[test]
    fn test_append() {
        assert_eq!(apply_cli_patterns("echo hello append file.txt"), "echo hello >> file.txt");
    }

    #[test]
    fn test_tilde() {
        assert_eq!(apply_cli_patterns("cd tilde"), "cd ~");
    }

    #[test]
    fn test_backtick() {
        assert_eq!(apply_cli_patterns("run backtick command backtick"), "run ` command `");
    }

    #[test]
    fn test_slashes() {
        assert_eq!(apply_cli_patterns("path forward slash to forward slash file"), "path / to / file");
        assert_eq!(apply_cli_patterns("backslash n"), "\\ n");
    }

    #[test]
    fn test_complex_command() {
        let result = apply_cli_patterns("git commit dash m fix typo and and git push");
        assert_eq!(result, "git commit -m fix typo && git push");
    }

    #[test]
    fn test_npm_commands() {
        assert_eq!(apply_cli_patterns("npm run dev"), "npm run dev");
        assert_eq!(apply_cli_patterns("npm install lodash"), "npm install lodash");
    }

    #[test]
    fn test_preserves_non_matches() {
        assert_eq!(apply_cli_patterns("hello world"), "hello world");
    }

    #[test]
    fn test_case_insensitive() {
        assert_eq!(apply_cli_patterns("DASH l"), "-l");
        assert_eq!(apply_cli_patterns("Pipe"), "|");
    }
}
