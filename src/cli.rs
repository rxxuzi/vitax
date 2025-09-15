//! Command line interface definitions and argument parsing.

use clap::Parser;

#[derive(Parser)]
#[command(name = "vitax")]
#[command(about = "A safe directory analysis tool")]
#[command(version)]
pub struct Args {
    /// Input paths to analyze
    pub paths: Vec<String>,

    /// Maximum recursion depth
    #[arg(short = 'd', long = "depth", default_value = "10")]
    pub max_depth: usize,

    /// Patterns to ignore (can be used multiple times)
    /// Examples: -I node_modules -I "*.json" -I .git
    #[arg(short = 'I', long = "ignore")]
    pub ignore: Vec<String>,

    /// Show hidden files and directories
    #[arg(short = 'a', long = "all")]
    pub show_hidden: bool,
}

impl Args {
    /// Validates the parsed arguments and returns errors if invalid.
    pub fn validate(&self) -> Result<(), String> {
        if self.paths.is_empty() {
            return Err("no input files".to_string());
        }

        if self.max_depth == 0 {
            return Err("depth must be at least 1".to_string());
        }

        Ok(())
    }

    /// Checks if a given path should be ignored based on ignore patterns.
    pub fn should_ignore(&self, path: &str) -> bool {
        let path_obj = std::path::Path::new(path);

        // Check each component of the path
        for component in path_obj.components() {
            if let std::path::Component::Normal(name) = component {
                let name_str = name.to_string_lossy();

                for pattern in &self.ignore {
                    if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
                        if glob_pattern.matches(&name_str) {
                            return true;
                        }
                    }
                }

                // Hide hidden files by default unless --all is specified
                if !self.show_hidden && name_str.starts_with('.') {
                    return true;
                }
            }
        }

        // Also check the full path and basename for patterns like "*.c"
        let basename = path_basename(path);
        for pattern in &self.ignore {
            if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
                if glob_pattern.matches(path) || glob_pattern.matches(basename) {
                    return true;
                }
            }
        }

        false
    }
}

fn path_basename(path: &str) -> &str {
    std::path::Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path)
}