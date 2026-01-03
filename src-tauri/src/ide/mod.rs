//! IDE Integrations module.
//!
//! Provides developer-specific features when dictating in code editors:
//! - Programming dictionary (API, JSON, SQL, etc.)
//! - CLI syntax patterns (dash, pipe, &&, etc.)
//! - Variable case recognition (camelCase, snake_case, etc.)
//! - File tagging by voice (@filename syntax)

pub mod cli_syntax;
pub mod dictionary;
pub mod file_index;
pub mod file_tagger;
pub mod variable;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// IDE context captured when recording starts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDEContext {
    /// Whether the active app is a supported IDE/terminal
    pub is_ide: bool,
    /// The bundle ID of the active app
    pub bundle_id: String,
    /// Detected programming language (if any)
    pub language: Option<String>,
}

impl Default for IDEContext {
    fn default() -> Self {
        IDEContext {
            is_ide: false,
            bundle_id: String::new(),
            language: None,
        }
    }
}

/// Settings for IDE integrations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDESettings {
    /// Enable file tagging by voice (@filename syntax)
    pub file_tagging_enabled: bool,
    /// Enable variable case recognition (camelCase, snake_case, etc.)
    pub variable_recognition_enabled: bool,
    /// Enable CLI syntax patterns (dash, pipe, etc.)
    pub cli_syntax_enabled: bool,
    /// Enable programming dictionary (API, JSON, etc.)
    pub dictionary_enabled: bool,
    /// Default variable case style
    pub default_case_style: variable::CaseStyle,
    /// User-configured workspace roots for file indexing
    pub workspace_roots: Vec<PathBuf>,
}

impl Default for IDESettings {
    fn default() -> Self {
        IDESettings {
            file_tagging_enabled: true,
            variable_recognition_enabled: true,
            cli_syntax_enabled: true,
            dictionary_enabled: true,
            default_case_style: variable::CaseStyle::CamelCase,
            workspace_roots: Vec::new(),
        }
    }
}

/// Supported IDE bundle IDs.
/// These are used to detect when the user is in a code editor.
const IDE_BUNDLE_IDS: &[&str] = &[
    // Terminals
    "com.apple.Terminal",
    "com.googlecode.iterm2",
    "dev.warp.Warp-Stable",
    "co.zeit.hyper",
    "net.kovidgoyal.kitty",
    "io.alacritty",
    // Code Editors
    "com.microsoft.VSCode",
    "com.microsoft.VSCodeInsiders",
    "com.todesktop.230313mzl4w4u92", // Cursor
    "com.codeium.windsurf",
    "com.sublimetext.4",
    "com.sublimetext.3",
    "com.panic.Nova",
    "com.barebones.bbedit",
    // IDEs
    "com.apple.dt.Xcode",
    "com.jetbrains.intellij",
    "com.jetbrains.intellij.ce",
    "com.jetbrains.pycharm",
    "com.jetbrains.pycharm.ce",
    "com.jetbrains.WebStorm",
    "com.jetbrains.PhpStorm",
    "com.jetbrains.CLion",
    "com.jetbrains.GoLand",
    "com.jetbrains.RubyMine",
    "com.jetbrains.rider",
    "com.google.android.studio",
    // Vim/Emacs
    "org.gnu.Emacs",
    "org.vim.MacVim",
    "com.qvacua.VimR",
];

/// Keywords found in text input service bundle IDs that indicate an IDE.
/// Example: "com.apple.TextInputUI.xpc.CursorUIViewService" contains "Cursor"
const IDE_TEXT_INPUT_KEYWORDS: &[&str] = &[
    "Cursor",    // Cursor editor
    "VSCode",    // VS Code
    "Code",      // VS Code variants
    "Xcode",     // Xcode
    "IntelliJ",  // JetBrains IntelliJ
    "PyCharm",   // JetBrains PyCharm
    "WebStorm",  // JetBrains WebStorm
    "GoLand",    // JetBrains GoLand
    "CLion",     // JetBrains CLion
    "Rider",     // JetBrains Rider
    "RubyMine",  // JetBrains RubyMine
    "PhpStorm",  // JetBrains PhpStorm
    "Android",   // Android Studio
    "Sublime",   // Sublime Text
    "Nova",      // Panic Nova
    "BBEdit",    // BBEdit
    "Emacs",     // Emacs
    "Vim",       // Vim/MacVim/VimR
    "Terminal",  // Terminal
    "iTerm",     // iTerm2
    "Warp",      // Warp terminal
    "Hyper",     // Hyper terminal
    "kitty",     // kitty terminal
    "Alacritty", // Alacritty terminal
    "Windsurf",  // Codeium Windsurf
];

/// Check if a bundle ID is a supported IDE/terminal.
///
/// Checks both exact bundle ID matches and text input service patterns.
/// When focused in an IDE's text field, macOS may report a text input service
/// bundle ID like "com.apple.TextInputUI.xpc.CursorUIViewService" instead of
/// the actual IDE bundle ID.
pub fn is_ide(bundle_id: &str) -> bool {
    // First, check exact matches
    if IDE_BUNDLE_IDS.contains(&bundle_id) {
        return true;
    }

    // Check if this is a text input service for a known IDE
    // These have patterns like "com.apple.TextInputUI.xpc.{IDE}UIViewService"
    if bundle_id.contains("TextInputUI") || bundle_id.contains("InputMethod") {
        for keyword in IDE_TEXT_INPUT_KEYWORDS {
            if bundle_id.contains(keyword) {
                println!(
                    "[IDE] Detected IDE via text input service: {} (matched '{}')",
                    bundle_id, keyword
                );
                return true;
            }
        }
    }

    false
}

/// Get IDE context for the active application.
pub fn get_ide_context(bundle_id: &str) -> IDEContext {
    IDEContext {
        is_ide: is_ide(bundle_id),
        bundle_id: bundle_id.to_string(),
        language: None, // Language detection can be added later
    }
}

/// Apply all IDE-specific transformations to text.
///
/// This is the main entry point for IDE processing.
/// Transformations are applied in order:
/// 1. Programming dictionary (API, JSON, etc.)
/// 2. CLI syntax patterns (dash, pipe, etc.)
/// 3. Variable case recognition (camelCase triggers)
/// 4. File tagging (if workspace is indexed)
///
/// # Arguments
/// * `text` - The raw transcription text
/// * `context` - IDE context for the active app
/// * `settings` - User's IDE settings
/// * `workspace_index` - Optional workspace file index for file tagging
pub fn apply_ide_transformations(
    text: &str,
    context: &IDEContext,
    settings: &IDESettings,
    workspace_index: Option<&file_index::WorkspaceIndex>,
) -> String {
    println!("[IDE] apply_ide_transformations called with: {:?}", text);
    println!(
        "[IDE] context.is_ide={}, bundle_id={}",
        context.is_ide, context.bundle_id
    );

    // Only apply transformations if in an IDE
    if !context.is_ide {
        println!("[IDE] Not in IDE, skipping transformations");
        return text.to_string();
    }

    let mut result = text.to_string();

    // 1. Programming dictionary
    if settings.dictionary_enabled {
        result = dictionary::apply_dictionary(&result);
    }

    // 2. CLI syntax patterns
    if settings.cli_syntax_enabled {
        result = cli_syntax::apply_cli_patterns(&result);
    }

    // 3. Variable case recognition
    if settings.variable_recognition_enabled {
        result = variable::apply_variable_patterns(&result, settings.default_case_style);
    }

    // 4. File tagging
    // Some patterns (literal filenames) work without an index
    // Others (fuzzy matching) require a workspace index
    if settings.file_tagging_enabled {
        result = file_tagger::apply_file_tagging(&result, workspace_index);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ide_vscode() {
        assert!(is_ide("com.microsoft.VSCode"));
    }

    #[test]
    fn test_is_ide_cursor() {
        assert!(is_ide("com.todesktop.230313mzl4w4u92"));
    }

    #[test]
    fn test_is_ide_terminal() {
        assert!(is_ide("com.apple.Terminal"));
    }

    #[test]
    fn test_is_not_ide() {
        assert!(!is_ide("com.apple.Notes"));
        assert!(!is_ide("com.tinyspeck.slackmacgap"));
    }

    #[test]
    fn test_get_ide_context() {
        let context = get_ide_context("com.microsoft.VSCode");
        assert!(context.is_ide);
        assert_eq!(context.bundle_id, "com.microsoft.VSCode");
    }

    #[test]
    fn test_default_settings() {
        let settings = IDESettings::default();
        assert!(settings.file_tagging_enabled);
        assert!(settings.dictionary_enabled);
        assert!(settings.cli_syntax_enabled);
        assert!(settings.variable_recognition_enabled);
    }

    #[test]
    fn test_is_ide_cursor_text_input_service() {
        // When Cursor's text input is active, macOS reports this bundle ID
        assert!(is_ide("com.apple.TextInputUI.xpc.CursorUIViewService"));
    }

    #[test]
    fn test_is_ide_vscode_text_input_service() {
        // VSCode text input service pattern
        assert!(is_ide("com.apple.TextInputUI.xpc.VSCodeUIViewService"));
    }

    #[test]
    fn test_text_input_non_ide_not_matched() {
        // Generic text input service should not match
        assert!(!is_ide("com.apple.TextInputUI.xpc.GenericUIViewService"));
    }
}
