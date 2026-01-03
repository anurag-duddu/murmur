//! Workspace file indexing.
//!
//! Indexes files in user-configured workspace roots for voice-based file tagging.
//! Uses the `ignore` crate to respect .gitignore patterns.

use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Maximum number of files to index per workspace.
const MAX_FILES: usize = 10_000;

/// File extensions to include in the index.
const INCLUDED_EXTENSIONS: &[&str] = &[
    // Web/Frontend
    "ts",
    "tsx",
    "js",
    "jsx",
    "mjs",
    "cjs",
    "vue",
    "svelte",
    "html",
    "css",
    "scss",
    "sass",
    "less",
    // Backend
    "rs",
    "go",
    "py",
    "rb",
    "java",
    "kt",
    "scala",
    "php",
    "c",
    "cpp",
    "cc",
    "h",
    "hpp",
    "cs",
    "swift",
    "m",
    "mm",
    // Config/Data
    "json",
    "yaml",
    "yml",
    "toml",
    "xml",
    "ini",
    "env",
    // Documentation
    "md",
    "mdx",
    "txt",
    "rst",
    // Shell/Scripts
    "sh",
    "bash",
    "zsh",
    "fish",
    "ps1",
    // Other
    "sql",
    "graphql",
    "proto",
    "dockerfile",
];

/// A file entry in the workspace index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Relative path from workspace root
    pub relative_path: String,
    /// Just the filename
    pub name: String,
    /// Lowercase filename without extension (for matching)
    pub name_normalized: String,
    /// File extension (without dot)
    pub extension: Option<String>,
}

impl FileEntry {
    /// Create a new file entry from a path.
    fn from_path(path: &Path, root: &Path) -> Option<Self> {
        let relative_path = path.strip_prefix(root).ok()?;
        let name = path.file_name()?.to_str()?.to_string();
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_string());

        // Normalize: lowercase, no extension
        let name_normalized = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        Some(FileEntry {
            relative_path: relative_path.to_string_lossy().to_string(),
            name,
            name_normalized,
            extension,
        })
    }
}

/// Index of files in a workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceIndex {
    /// Root path of the workspace
    pub root: PathBuf,
    /// Indexed files
    pub files: Vec<FileEntry>,
    /// When the index was built (as Unix timestamp for serialization)
    #[serde(skip)]
    #[allow(dead_code)] // Used internally, not serialized
    pub updated_at: Option<Instant>,
    /// Number of files that were skipped due to limits
    pub files_skipped: usize,
}

impl WorkspaceIndex {
    /// Build an index of files in the workspace.
    ///
    /// # Arguments
    /// * `root` - The root directory to index
    ///
    /// # Returns
    /// A WorkspaceIndex containing all indexed files, or an error message.
    ///
    /// # Limits
    /// - Maximum 10,000 files
    /// - Respects .gitignore
    /// - Only includes files with recognized extensions
    pub fn build(root: &Path) -> Result<Self, String> {
        if !root.exists() {
            return Err(format!("Workspace root does not exist: {}", root.display()));
        }

        if !root.is_dir() {
            return Err(format!(
                "Workspace root is not a directory: {}",
                root.display()
            ));
        }

        let mut files = Vec::new();
        let mut files_skipped = 0;

        // Use ignore crate's WalkBuilder to respect .gitignore
        let walker = WalkBuilder::new(root)
            .hidden(true) // Skip hidden files/dirs
            .git_ignore(true) // Respect .gitignore
            .git_global(true) // Respect global gitignore
            .git_exclude(true) // Respect .git/info/exclude
            .max_depth(Some(20)) // Reasonable depth limit
            .build();

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            // Skip directories
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(true) {
                continue;
            }

            let path = entry.path();

            // Check extension
            let extension = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_lowercase());

            let has_valid_extension = extension
                .as_ref()
                .map(|ext| INCLUDED_EXTENSIONS.contains(&ext.as_str()))
                .unwrap_or(false);

            if !has_valid_extension {
                continue;
            }

            // Check file limit
            if files.len() >= MAX_FILES {
                files_skipped += 1;
                continue;
            }

            // Create file entry
            if let Some(file_entry) = FileEntry::from_path(path, root) {
                files.push(file_entry);
            }
        }

        Ok(WorkspaceIndex {
            root: root.to_path_buf(),
            files,
            updated_at: Some(Instant::now()),
            files_skipped,
        })
    }

    /// Get the number of indexed files.
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Check if the index is empty.
    #[allow(dead_code)] // Used in tests
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Find files matching a search query.
    ///
    /// Returns files where the normalized name contains the query.
    #[allow(dead_code)] // Used in tests
    pub fn find_by_name(&self, query: &str) -> Vec<&FileEntry> {
        let query_lower = query.to_lowercase();
        self.files
            .iter()
            .filter(|f| f.name_normalized.contains(&query_lower))
            .collect()
    }

    /// Find files matching an exact normalized name.
    #[allow(dead_code)] // Used in tests
    pub fn find_exact(&self, name: &str) -> Option<&FileEntry> {
        let name_lower = name.to_lowercase();
        self.files.iter().find(|f| f.name_normalized == name_lower)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_workspace() -> TempDir {
        let temp_dir = TempDir::new().unwrap();

        // Create some test files
        fs::write(temp_dir.path().join("main.rs"), "fn main() {}").unwrap();
        fs::write(temp_dir.path().join("lib.rs"), "pub fn lib() {}").unwrap();
        fs::write(temp_dir.path().join("utils.ts"), "export {}").unwrap();
        fs::write(temp_dir.path().join("README.md"), "# Test").unwrap();

        // Create a subdirectory
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        fs::write(src_dir.join("app.tsx"), "export {}").unwrap();
        fs::write(src_dir.join("index.ts"), "export {}").unwrap();

        // Create a file that should be ignored (binary)
        fs::write(temp_dir.path().join("image.png"), &[0u8; 10]).unwrap();

        temp_dir
    }

    #[test]
    fn test_build_index() {
        let temp_dir = create_test_workspace();
        let index = WorkspaceIndex::build(temp_dir.path()).unwrap();

        // Should have indexed the code files
        assert!(index.file_count() >= 5);
        assert!(!index.is_empty());
    }

    #[test]
    fn test_find_by_name() {
        let temp_dir = create_test_workspace();
        let index = WorkspaceIndex::build(temp_dir.path()).unwrap();

        let results = index.find_by_name("main");
        assert!(!results.is_empty());
        assert!(results.iter().any(|f| f.name == "main.rs"));
    }

    #[test]
    fn test_find_exact() {
        let temp_dir = create_test_workspace();
        let index = WorkspaceIndex::build(temp_dir.path()).unwrap();

        let result = index.find_exact("main");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "main.rs");
    }

    #[test]
    fn test_excludes_binary_files() {
        let temp_dir = create_test_workspace();
        let index = WorkspaceIndex::build(temp_dir.path()).unwrap();

        // Should not include the .png file
        let results = index.find_by_name("image");
        assert!(results.is_empty());
    }

    #[test]
    fn test_normalized_name() {
        let temp_dir = create_test_workspace();
        let index = WorkspaceIndex::build(temp_dir.path()).unwrap();

        let result = index.find_exact("utils");
        assert!(result.is_some());
        let entry = result.unwrap();
        assert_eq!(entry.name, "utils.ts");
        assert_eq!(entry.name_normalized, "utils");
        assert_eq!(entry.extension, Some("ts".to_string()));
    }

    #[test]
    fn test_nonexistent_root() {
        let result = WorkspaceIndex::build(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_file_entry_relative_path() {
        let temp_dir = create_test_workspace();
        let index = WorkspaceIndex::build(temp_dir.path()).unwrap();

        // Find the nested file
        let results = index.find_by_name("app");
        assert!(!results.is_empty());

        let app_file = results.iter().find(|f| f.name == "app.tsx").unwrap();
        assert!(app_file.relative_path.contains("src"));
    }
}
