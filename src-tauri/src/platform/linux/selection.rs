//! Linux selection stub for cross-platform compilation.
//!
//! This is a placeholder that allows the code to compile on Linux.
//! Selection functionality is macOS-only.

/// Error types for selection operations (mirrors macOS interface)
#[derive(Debug)]
pub enum SelectionError {
    /// No text is selected
    NoSelection,
    /// Accessibility permission not granted
    AccessibilityDenied,
    /// Selection is invalid
    InvalidSelection(String),
    /// Failed to get selection for other reasons
    Failed(String),
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

/// Stub: Always returns false on Linux
pub fn has_selection() -> bool {
    false
}

/// Stub: Always returns NoSelection error on Linux
pub fn get_selected_text() -> Result<String, SelectionError> {
    Err(SelectionError::Failed("Selection not supported on Linux".to_string()))
}

/// Stub: Always returns None on Linux
pub fn get_selected_text_or_none() -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_error_display() {
        let no_sel = SelectionError::NoSelection;
        assert_eq!(format!("{}", no_sel), "No text selected");

        let access_denied = SelectionError::AccessibilityDenied;
        assert_eq!(format!("{}", access_denied), "Accessibility permission denied");

        let failed = SelectionError::Failed("test error".to_string());
        assert_eq!(format!("{}", failed), "Selection error: test error");
    }

    #[test]
    fn test_has_selection_returns_false() {
        assert!(!has_selection());
    }

    #[test]
    fn test_get_selected_text_returns_error() {
        assert!(get_selected_text().is_err());
    }

    #[test]
    fn test_get_selected_text_or_none_returns_none() {
        assert!(get_selected_text_or_none().is_none());
    }
}
