//! File tagging by voice.
//!
//! Matches spoken filenames against the workspace index and converts them to @filename syntax:
//! - "auth check dot ts" → "@authCheck.ts"
//! - "the main file" → "@main.rs"

use super::file_index::{FileEntry, WorkspaceIndex};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use regex::Regex;
use std::sync::LazyLock;

/// Minimum fuzzy match score to consider a match valid.
const MIN_MATCH_SCORE: i64 = 50;

/// Maximum number of words to consider as a potential filename.
const MAX_FILENAME_WORDS: usize = 5;

/// Pattern for spoken filename with extension: "[word(s)] dot [extension]"
/// Uses word boundary and limited word count to avoid matching too much context.
/// Matches only 1-2 words immediately before "dot extension".
static DOT_EXTENSION_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    // Match 1-2 words (typical filename length) before "dot extension"
    // Using word boundaries to ensure we capture just the filename
    Regex::new(r"(?i)\b(\w+(?:\s+\w+)?)\s+dot\s+(ts|tsx|js|jsx|cjs|mjs|vue|svelte|rs|py|go|rb|java|html|css|scss|sass|less|json|yaml|yml|md|txt|toml|sh|sql|graphql)\b").unwrap()
});

/// Pattern for already-formatted filenames: "filename.ext"
/// Matches filenames that already have a period and extension.
/// Captures optional prefix character to check if already tagged.
static LITERAL_FILENAME_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    // Match optional prefix char + filename with extension
    // Group 1 = prefix (may be empty, @, or /), Group 2 = filename
    // Allows dots in filename like "tailwind.config.cjs" or "vite.config.ts"
    Regex::new(r"(^|[^@/\w])([\w.-]+\.(?:ts|tsx|js|jsx|cjs|mjs|vue|svelte|rs|py|go|rb|java|html|css|scss|sass|less|json|yaml|yml|md|txt|toml|sh|sql|graphql))\b").unwrap()
});

/// Pattern for common file references: "the [name] file"
static THE_FILE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bthe\s+(\w+)\s+file\b").unwrap()
});

/// Pattern for explicit file reference: "file [name]"
static FILE_PREFIX_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bfile\s+(\w+(?:\s+dot\s+\w+)?)\b").unwrap()
});

/// Fuzzy matcher for filename matching.
static FUZZY_MATCHER: LazyLock<SkimMatcherV2> = LazyLock::new(SkimMatcherV2::default);

/// Pattern to clean up punctuation attached to @-tagged filenames.
/// Matches @filename.ext followed by punctuation, adds space before punctuation.
static PUNCTUATION_CLEANUP_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    // Match @filename.ext immediately followed by sentence-ending punctuation
    // Group 1 = the @filename.ext, Group 2 = the punctuation
    Regex::new(r"(@[\w.-]+\.(?:ts|tsx|js|jsx|cjs|mjs|vue|svelte|rs|py|go|rb|java|html|css|scss|sass|less|json|yaml|yml|md|txt|toml|sh|sql|graphql))([?!.,;:])").unwrap()
});

/// Apply file tagging to text.
///
/// IMPORTANT: Only tags files that ACTUALLY EXIST in the workspace index.
/// If no workspace index is provided, no tagging is performed.
/// This ensures we don't create @ references to non-existent files.
///
/// # Arguments
/// * `text` - The text to process
/// * `index` - Workspace file index (required for tagging to work)
///
/// # Examples
/// ```ignore
/// let result = apply_file_tagging("Fix the bug in auth check dot ts", Some(&index));
/// // If authCheck.ts exists in index: "Fix the bug in @authCheck.ts"
/// // If authCheck.ts does NOT exist: "Fix the bug in auth check dot ts" (unchanged)
///
/// let result = apply_file_tagging("Open components.json", None);
/// // No index = no tagging: "Open components.json" (unchanged)
/// ```
pub fn apply_file_tagging(text: &str, index: Option<&WorkspaceIndex>) -> String {
    // If no workspace index, skip all tagging - we can't verify files exist
    let idx = match index {
        Some(idx) => idx,
        None => {
            println!("[FILE_TAGGER] No workspace index - skipping file tagging");
            return text.to_string();
        }
    };

    let mut result = text.to_string();

    // 1. Match literal filenames like "components.json" → "@components.json"
    // Only if the file exists in the index
    result = apply_literal_filename_pattern(&result, idx);

    // 2. Match "[words] dot [extension]" pattern
    result = apply_dot_extension_pattern(&result, idx);

    // 3. Match "the [name] file" pattern
    result = apply_the_file_pattern(&result, idx);

    // 4. Match "file [name]" pattern
    result = apply_file_prefix_pattern(&result, idx);

    result
}

/// Clean up punctuation attached to @-tagged filenames.
///
/// The LLM may add punctuation directly after @filename.ext (e.g., "@components.json?").
/// This function separates the punctuation with a space so the @ reference stays valid.
///
/// # Examples
/// ```ignore
/// cleanup_tagged_punctuation("Open @components.json?") → "Open @components.json ?"
/// cleanup_tagged_punctuation("Check @main.rs, please") → "Check @main.rs , please"
/// ```
pub fn cleanup_tagged_punctuation(text: &str) -> String {
    let result = PUNCTUATION_CLEANUP_PATTERN
        .replace_all(text, "$1 $2")
        .to_string();
    if result != text {
        println!(
            "[FILE_TAGGER] Cleaned up punctuation: '{}' -> '{}'",
            text, result
        );
    }
    result
}

/// Apply the literal filename pattern (e.g., "components.json" → "@components.json").
/// This handles cases where the transcription already outputs proper filenames.
/// ONLY tags files that exist in the workspace index.
fn apply_literal_filename_pattern(text: &str, index: &WorkspaceIndex) -> String {
    let mut result = text.to_string();

    // Find all potential filename matches
    for captures in LITERAL_FILENAME_PATTERN.captures_iter(text) {
        let filename = captures.get(2).unwrap().as_str();

        // Check if this file exists in the workspace index
        let file_exists = index.files.iter().any(|f| f.name == filename);

        if file_exists {
            // Replace only this occurrence
            let full_match = captures.get(0).unwrap().as_str();
            let prefix = captures.get(1).unwrap().as_str();
            let replacement = format!("{}@{}", prefix, filename);
            result = result.replacen(full_match, &replacement, 1);
            println!("[FILE_TAGGER] Tagged verified file: {} -> @{}", filename, filename);
        } else {
            println!("[FILE_TAGGER] File '{}' not in workspace - not tagging", filename);
        }
    }

    result
}

/// Apply the "[words] dot [extension]" pattern.
/// Only tags if a matching file is found in the index - no guessing.
fn apply_dot_extension_pattern(text: &str, index: &WorkspaceIndex) -> String {
    // Collect all replacements first to avoid borrow issues
    let mut replacements: Vec<(String, String)> = Vec::new();

    for captures in DOT_EXTENSION_PATTERN.captures_iter(text) {
        let full_match = captures.get(0).unwrap().as_str().to_string();
        let words = captures.get(1).unwrap().as_str();
        let extension = captures.get(2).unwrap().as_str();

        // Normalize the spoken words (remove spaces, try different casings)
        let normalized = normalize_spoken_filename(words);

        // Try to find a matching file - ONLY tag if we find one
        if let Some(file) = find_best_match(&normalized, Some(extension), index) {
            let replacement = format!("@{}", file.name);
            println!("[FILE_TAGGER] Matched spoken '{}' to @{}", words, file.name);
            replacements.push((full_match, replacement));
        } else {
            // No match found - leave the text unchanged, don't guess
            println!("[FILE_TAGGER] No match for '{} dot {}' - leaving unchanged", words, extension);
        }
    }

    // Apply all replacements
    let mut result = text.to_string();
    for (from, to) in replacements {
        result = result.replace(&from, &to);
    }
    result
}

/// Apply the "the [name] file" pattern.
fn apply_the_file_pattern(text: &str, index: &WorkspaceIndex) -> String {
    let mut result = text.to_string();

    while let Some(captures) = THE_FILE_PATTERN.captures(&result) {
        let full_match = captures.get(0).unwrap();
        let name = captures.get(1).unwrap().as_str();

        // Try to find a matching file
        if let Some(file) = find_best_match(name, None, index) {
            let replacement = format!("@{}", file.name);
            result = result.replace(full_match.as_str(), &replacement);
        }
        // If no match, leave as-is (don't guess)
        else {
            break;
        }
    }

    result
}

/// Apply the "file [name]" pattern.
fn apply_file_prefix_pattern(text: &str, index: &WorkspaceIndex) -> String {
    let mut result = text.to_string();

    while let Some(captures) = FILE_PREFIX_PATTERN.captures(&result) {
        let full_match = captures.get(0).unwrap();
        let name = captures.get(1).unwrap().as_str();

        // Handle "file name dot extension"
        let (base_name, extension) = if name.to_lowercase().contains(" dot ") {
            let parts: Vec<&str> = name.splitn(2, |c: char| {
                c.to_lowercase().to_string() == "dot"
                    || name.to_lowercase().contains(" dot ")
            }).collect();
            if parts.len() == 2 {
                (parts[0].trim(), Some(parts[1].trim()))
            } else {
                (name, None)
            }
        } else {
            (name, None)
        };

        // Try to find a matching file
        if let Some(file) = find_best_match(base_name, extension, index) {
            let replacement = format!("@{}", file.name);
            result = result.replace(full_match.as_str(), &replacement);
        }
        // If no match, leave as-is
        else {
            break;
        }
    }

    result
}

/// Normalize a spoken filename to a likely code filename.
///
/// "auth check" → "authCheck" (camelCase)
/// "user service" → "userService"
fn normalize_spoken_filename(spoken: &str) -> String {
    let words: Vec<&str> = spoken.split_whitespace().collect();
    if words.is_empty() {
        return String::new();
    }

    // Convert to camelCase (most common for filenames)
    words
        .iter()
        .enumerate()
        .map(|(i, w)| {
            if i == 0 {
                w.to_lowercase()
            } else {
                let mut chars = w.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => c.to_uppercase().chain(chars.map(|c| c.to_ascii_lowercase())).collect(),
                }
            }
        })
        .collect()
}

/// Find the best matching file in the index.
fn find_best_match<'a>(
    name: &str,
    extension: Option<&str>,
    index: &'a WorkspaceIndex,
) -> Option<&'a FileEntry> {
    let name_lower = name.to_lowercase().replace(' ', "");

    // First, try exact match
    let candidates: Vec<&FileEntry> = if let Some(ext) = extension {
        let ext_lower = ext.to_lowercase();
        index
            .files
            .iter()
            .filter(|f| {
                f.extension
                    .as_ref()
                    .map(|e| e.to_lowercase() == ext_lower)
                    .unwrap_or(false)
            })
            .collect()
    } else {
        index.files.iter().collect()
    };

    // Exact match on normalized name
    if let Some(file) = candidates
        .iter()
        .find(|f| f.name_normalized == name_lower)
    {
        return Some(file);
    }

    // Fuzzy match
    let mut best_match: Option<(&FileEntry, i64)> = None;

    for file in candidates {
        // Score against normalized name
        if let Some(score) = FUZZY_MATCHER.fuzzy_match(&file.name_normalized, &name_lower) {
            if score >= MIN_MATCH_SCORE {
                if best_match.is_none() || score > best_match.unwrap().1 {
                    best_match = Some((file, score));
                }
            }
        }

        // Also try matching against the full name (without extension)
        if let Some(score) = FUZZY_MATCHER.fuzzy_match(&file.name, &name_lower) {
            if score >= MIN_MATCH_SCORE {
                if best_match.is_none() || score > best_match.unwrap().1 {
                    best_match = Some((file, score));
                }
            }
        }
    }

    best_match.map(|(file, _)| file)
}

/// Match a spoken phrase to a file in the index.
///
/// Returns the filename if a good match is found.
pub fn match_spoken_to_file(spoken: &str, index: &WorkspaceIndex) -> Option<String> {
    let normalized = normalize_spoken_filename(spoken);
    find_best_match(&normalized, None, index).map(|f| f.name.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_index() -> WorkspaceIndex {
        WorkspaceIndex {
            root: PathBuf::from("/test"),
            files: vec![
                FileEntry {
                    relative_path: "src/authCheck.ts".to_string(),
                    name: "authCheck.ts".to_string(),
                    name_normalized: "authcheck".to_string(),
                    extension: Some("ts".to_string()),
                },
                FileEntry {
                    relative_path: "src/userService.ts".to_string(),
                    name: "userService.ts".to_string(),
                    name_normalized: "userservice".to_string(),
                    extension: Some("ts".to_string()),
                },
                FileEntry {
                    relative_path: "main.rs".to_string(),
                    name: "main.rs".to_string(),
                    name_normalized: "main".to_string(),
                    extension: Some("rs".to_string()),
                },
                FileEntry {
                    relative_path: "lib.rs".to_string(),
                    name: "lib.rs".to_string(),
                    name_normalized: "lib".to_string(),
                    extension: Some("rs".to_string()),
                },
                FileEntry {
                    relative_path: "README.md".to_string(),
                    name: "README.md".to_string(),
                    name_normalized: "readme".to_string(),
                    extension: Some("md".to_string()),
                },
            ],
            updated_at: None,
            files_skipped: 0,
        }
    }

    #[test]
    fn test_dot_extension_pattern() {
        let index = create_test_index();
        let result = apply_file_tagging("Fix the bug in auth check dot ts", Some(&index));
        assert_eq!(result, "Fix the bug in @authCheck.ts");
    }

    #[test]
    fn test_the_file_pattern() {
        let index = create_test_index();
        let result = apply_file_tagging("Check the main file", Some(&index));
        assert_eq!(result, "Check @main.rs");
    }

    #[test]
    fn test_no_match_preserves_text() {
        let index = create_test_index();
        let result = apply_file_tagging("Hello world", Some(&index));
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_normalize_spoken_filename() {
        assert_eq!(normalize_spoken_filename("auth check"), "authCheck");
        assert_eq!(normalize_spoken_filename("user service"), "userService");
        assert_eq!(normalize_spoken_filename("main"), "main");
    }

    #[test]
    fn test_match_spoken_to_file() {
        let index = create_test_index();
        let result = match_spoken_to_file("auth check", &index);
        assert_eq!(result, Some("authCheck.ts".to_string()));
    }

    #[test]
    fn test_match_spoken_to_file_no_match() {
        let index = create_test_index();
        let result = match_spoken_to_file("nonexistent", &index);
        assert!(result.is_none());
    }

    #[test]
    fn test_fuzzy_match() {
        let index = create_test_index();
        // "auth" should fuzzy match "authCheck"
        let result = match_spoken_to_file("auth", &index);
        assert!(result.is_some());
    }

    #[test]
    fn test_case_insensitive() {
        let index = create_test_index();
        let result = apply_file_tagging("Fix the bug in AUTH CHECK DOT TS", Some(&index));
        assert_eq!(result, "Fix the bug in @authCheck.ts");
    }

    #[test]
    fn test_unknown_file_not_tagged() {
        let index = create_test_index();
        // When no file matches, DON'T tag - leave text unchanged
        let result = apply_file_tagging("edit the unknown dot ts", Some(&index));
        assert!(!result.contains("@"), "Unknown files should NOT be tagged");

        let result2 = apply_file_tagging("open mystery dot ts", Some(&index));
        assert!(!result2.contains("@"), "Unknown files should NOT be tagged");
    }

    #[test]
    fn test_literal_filename_only_tags_existing() {
        // Without an index, nothing should be tagged
        let result = apply_file_tagging("Open components.json", None);
        assert_eq!(result, "Open components.json", "Without index, no tagging");

        // With an index, only existing files get tagged
        let index = create_test_index();
        // main.rs exists in index
        let result = apply_file_tagging("Check main.rs for errors", Some(&index));
        assert_eq!(result, "Check @main.rs for errors");

        // nonexistent.ts does NOT exist in index - should NOT be tagged
        let result = apply_file_tagging("Open nonexistent.ts", Some(&index));
        assert_eq!(result, "Open nonexistent.ts", "Non-existent file should not be tagged");
    }

    #[test]
    fn test_literal_filename_multiple_existing() {
        let index = create_test_index();
        // main.rs and lib.rs both exist in index
        let result = apply_file_tagging("Check main.rs and lib.rs for errors", Some(&index));
        assert_eq!(result, "Check @main.rs and @lib.rs for errors");
    }

    #[test]
    fn test_literal_filename_not_double_tagged() {
        // Already tagged files should not get double-tagged
        let index = create_test_index();
        let result = apply_file_tagging("Open @main.rs", Some(&index));
        assert_eq!(result, "Open @main.rs");
    }

    #[test]
    fn test_literal_filename_in_path_not_tagged() {
        // Filenames in paths (after /) should not get @ prefix
        let index = create_test_index();
        let result = apply_file_tagging("Edit /src/main.rs", Some(&index));
        assert_eq!(result, "Edit /src/main.rs");
    }

    #[test]
    fn test_no_index_means_no_tagging() {
        // No workspace index = no tagging at all
        let result = apply_file_tagging("Check main.rs and lib.rs", None);
        assert_eq!(result, "Check main.rs and lib.rs");

        let result = apply_file_tagging("Fix auth check dot ts", None);
        assert_eq!(result, "Fix auth check dot ts");
    }

    #[test]
    fn test_cleanup_question_mark() {
        let result = cleanup_tagged_punctuation("Can you open @components.json?");
        assert_eq!(result, "Can you open @components.json ?");
    }

    #[test]
    fn test_cleanup_period() {
        let result = cleanup_tagged_punctuation("Please open @main.rs.");
        assert_eq!(result, "Please open @main.rs .");
    }

    #[test]
    fn test_cleanup_comma() {
        let result = cleanup_tagged_punctuation("Check @lib.rs, then run tests");
        assert_eq!(result, "Check @lib.rs , then run tests");
    }

    #[test]
    fn test_cleanup_no_punctuation_unchanged() {
        let result = cleanup_tagged_punctuation("Open @components.json and @main.rs");
        assert_eq!(result, "Open @components.json and @main.rs");
    }

    #[test]
    fn test_cleanup_multiple_files() {
        let result = cleanup_tagged_punctuation("Check @main.rs, @lib.rs, and @mod.rs.");
        assert_eq!(result, "Check @main.rs , @lib.rs , and @mod.rs .");
    }
}
