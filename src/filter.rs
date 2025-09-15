//! File filtering logic.
//!
//! This module provides filtering capabilities for files and directories
//! based on extensions, ignore patterns, and hidden file visibility.

use std::path::Path;
use glob::Pattern;

/// Manages file filtering based on various criteria.
#[derive(Debug, Clone)]
pub struct FileFilter {
    /// File extensions to include (empty = all)
    extensions: Vec<String>,
    /// Glob patterns to ignore
    ignore_patterns: Vec<String>,
    /// Whether to show hidden files
    show_hidden: bool,
}

impl FileFilter {
    /// Creates a new FileFilter with the specified criteria.
    pub fn new(extensions: Vec<String>, ignore_patterns: Vec<String>, show_hidden: bool) -> Self {
        Self {
            extensions,
            ignore_patterns,
            show_hidden,
        }
    }

    /// Determines if a path should be processed.
    ///
    /// # Arguments
    /// * `path` - The file or directory path to check
    ///
    /// Returns `true` if the path passes all filters, `false` otherwise.
    pub fn should_process(&self, path: &str) -> bool {
        if self.should_ignore(path) {
            return false;
        }

        if Path::new(path).is_dir() {
            return true;
        }

        self.matches_extension(path)
    }

    /// Checks if a file matches the extension filter.
    fn matches_extension(&self, path: &str) -> bool {
        if self.extensions.is_empty() {
            return true;
        }

        Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| {
                let ext_lower = ext.to_lowercase();
                self.extensions.iter().any(|e| e.to_lowercase() == ext_lower)
            })
            .unwrap_or(false)
    }

    /// Checks if a path should be ignored based on patterns and hidden file rules.
    fn should_ignore(&self, path: &str) -> bool {
        let path_obj = Path::new(path);

        for component in path_obj.components() {
            if let std::path::Component::Normal(name) = component {
                let name_str = name.to_string_lossy();

                if self.matches_ignore_pattern(&name_str) {
                    return true;
                }

                if !self.show_hidden && name_str.starts_with('.') {
                    return true;
                }
            }
        }

        let basename = path_obj
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path);

        self.matches_ignore_pattern(path) || self.matches_ignore_pattern(basename)
    }

    /// Tests if text matches any ignore pattern.
    fn matches_ignore_pattern(&self, text: &str) -> bool {
        self.ignore_patterns.iter().any(|pattern| {
            Pattern::new(pattern)
                .map(|glob| glob.matches(text))
                .unwrap_or(false)
        })
    }

    /// Returns a human-readable description of active filters.
    pub fn describe(&self) -> String {
        let mut parts = Vec::new();

        if !self.extensions.is_empty() {
            parts.push(format!("extensions: {}", self.extensions.join(", ")));
        }

        if !self.ignore_patterns.is_empty() {
            parts.push(format!("ignoring: {}", self.ignore_patterns.join(", ")));
        }

        if !self.show_hidden {
            parts.push("hiding hidden files".to_string());
        }

        if parts.is_empty() {
            "no filters applied".to_string()
        } else {
            format!("Filters: {}", parts.join("; "))
        }
    }

    /// Returns true if any filters are active.
    pub fn has_filters(&self) -> bool {
        !self.extensions.is_empty() || !self.ignore_patterns.is_empty() || !self.show_hidden
    }

    /// Returns the list of active extension filters.
    pub fn extensions(&self) -> &[String] {
        &self.extensions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_filtering() {
        let filter = FileFilter::new(
            vec!["rs".to_string(), "toml".to_string()],
            vec![],
            false,
        );

        assert!(filter.matches_extension("main.rs"));
        assert!(filter.matches_extension("Cargo.toml"));
        assert!(filter.matches_extension("Main.RS"));
        assert!(!filter.matches_extension("README.md"));
        assert!(!filter.matches_extension("no_extension"));
    }

    #[test]
    fn test_ignore_patterns() {
        let filter = FileFilter::new(
            vec![],
            vec!["*.tmp".to_string(), "target".to_string()],
            false,
        );

        assert!(filter.should_ignore("file.tmp"));
        assert!(filter.should_ignore("target/debug/build"));
        assert!(filter.should_ignore(".hidden_file"));
        assert!(!filter.should_ignore("main.rs"));
    }

    #[test]
    fn test_combined_filters() {
        let filter = FileFilter::new(
            vec!["rs".to_string()],
            vec!["*_test.rs".to_string()],
            false,
        );

        assert!(filter.should_process("main.rs"));
        assert!(!filter.should_process("main_test.rs"));
        assert!(!filter.should_process("README.md"));
        assert!(!filter.should_process(".hidden.rs"));
    }

    #[test]
    fn test_show_hidden() {
        let filter_hide = FileFilter::new(vec![], vec![], false);
        let filter_show = FileFilter::new(vec![], vec![], true);

        assert!(filter_hide.should_ignore(".gitignore"));
        assert!(!filter_show.should_ignore(".gitignore"));
    }
}