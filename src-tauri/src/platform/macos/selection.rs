//! macOS selection detection and manipulation via Accessibility API.
//!
//! This module provides functions to:
//! - Check if any text is currently selected
//! - Get the currently selected text
//! - Replace the selected text with new content
//!
//! Uses the macOS Accessibility API exclusively. No clipboard fallback for reading.

use get_selected_text::get_selected_text as get_selected_text_impl;
use regex::Regex;
use std::sync::LazyLock;

/// Static UUID pattern regex - compiled once and reused.
/// SAFETY: unwrap() is safe here because the regex is a compile-time constant
/// that has been validated during development. LazyLock ensures it's only
/// compiled once on first use.
static UUID_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}").unwrap()
});

/// Error types for selection operations
#[derive(Debug)]
pub enum SelectionError {
    /// No text is selected
    NoSelection,
    /// Accessibility permission not granted
    AccessibilityDenied,
    /// Selection is invalid (e.g., VS Code webview path)
    InvalidSelection(String),
    /// Failed to get selection for other reasons
    Failed(String),
}

/// Check if the selected text looks like valid user-selected content.
///
/// Some apps (especially VS Code) return internal identifiers instead of
/// actual selected text via the Accessibility API. We detect and reject these.
fn is_valid_selection(text: &str) -> bool {
    // Reject VS Code webview panel identifiers
    if text.contains("webview-panel/") || text.contains("webview-") {
        #[cfg(debug_assertions)]
        println!("[SELECTION] Rejecting VS Code webview identifier");
        return false;
    }

    // Reject paths that look like internal URIs
    if text.starts_with("vscode-") || text.starts_with("file://") {
        #[cfg(debug_assertions)]
        println!("[SELECTION] Rejecting internal URI");
        return false;
    }

    // Reject UUID-like strings (common in accessibility tree paths)
    // Pattern: contains multiple UUID segments like "8ab98c93-a228-4d81-bcbe-8c175dddd3fa"
    if UUID_PATTERN.is_match(text) && text.len() < 100 {
        // Short text that's mostly a UUID is likely an internal identifier
        #[cfg(debug_assertions)]
        println!("[SELECTION] Rejecting UUID-like selection");
        return false;
    }

    true
}

impl std::fmt::Display for SelectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectionError::NoSelection => write!(f, "No text selected"),
            SelectionError::AccessibilityDenied => write!(f, "Accessibility permission denied"),
            SelectionError::InvalidSelection(msg) => write!(f, "Invalid selection: {}", msg),
            SelectionError::Failed(msg) => write!(f, "Selection error: {}", msg),
        }
    }
}

impl std::error::Error for SelectionError {}

/// Get the currently selected text from the frontmost application.
///
/// Uses the macOS Accessibility API exclusively. Does NOT use clipboard.
///
/// # Returns
/// - `Ok(String)` - The selected text
/// - `Err(SelectionError::NoSelection)` - No text is selected
/// - `Err(SelectionError::AccessibilityDenied)` - Permission not granted
/// - `Err(SelectionError::Failed)` - Other error
pub fn get_selected_text() -> Result<String, SelectionError> {
    match get_selected_text_impl() {
        Ok(text) => {
            if text.is_empty() {
                Err(SelectionError::NoSelection)
            } else if !is_valid_selection(&text) {
                // Reject bogus selections (like VS Code webview paths)
                Err(SelectionError::InvalidSelection(text))
            } else {
                // Note: Not logging selected text content to avoid leaking sensitive data
                #[cfg(debug_assertions)]
                println!("[SELECTION] Got {} chars", text.len());
                Ok(text)
            }
        }
        Err(e) => {
            let error_msg = format!("{:?}", e);

            // Check for common error patterns
            if error_msg.contains("accessibility")
                || error_msg.contains("permission")
                || error_msg.contains("not trusted")
            {
                Err(SelectionError::AccessibilityDenied)
            } else if error_msg.contains("no selection")
                || error_msg.contains("empty")
                || error_msg.contains("null")
            {
                Err(SelectionError::NoSelection)
            } else {
                Err(SelectionError::Failed(error_msg))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_error_display() {
        let no_sel = SelectionError::NoSelection;
        assert_eq!(format!("{}", no_sel), "No text selected");

        let access_denied = SelectionError::AccessibilityDenied;
        assert_eq!(
            format!("{}", access_denied),
            "Accessibility permission denied"
        );

        let failed = SelectionError::Failed("test error".to_string());
        assert_eq!(format!("{}", failed), "Selection error: test error");
    }
}
