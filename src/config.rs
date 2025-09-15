//! Application configuration and validation.
//!
//! This module manages the application's configuration,
//! validates user inputs, and creates the necessary components.

use crate::cli::Args;
use crate::filter::FileFilter;
use std::path::Path;

/// Application configuration built from CLI arguments.
#[derive(Debug)]
pub struct Config {
    /// Paths to process
    pub paths: Vec<String>,
    /// Maximum recursion depth
    pub max_depth: usize,
    /// File filter instance
    pub filter: FileFilter,
}

impl Config {
    /// Creates a new configuration from CLI arguments.
    ///
    /// # Arguments
    /// * `args` - Parsed command line arguments
    ///
    /// Returns `Ok(Config)` if validation passes, `Err(ConfigError)` otherwise.
    pub fn from_args(args: Args) -> Result<Self, ConfigError> {
        Self::validate(&args)?;

        let filter = FileFilter::new(
            args.extensions,
            args.ignore,
            args.show_hidden,
        );

        Ok(Self {
            paths: args.paths,
            max_depth: args.max_depth,
            filter,
        })
    }

    /// Validates CLI arguments for correctness.
    fn validate(args: &Args) -> Result<(), ConfigError> {
        if args.paths.is_empty() {
            return Err(ConfigError::NoInputFiles);
        }

        if args.max_depth == 0 {
            return Err(ConfigError::InvalidDepth);
        }

        for path in &args.paths {
            if !Path::new(path).exists() {
                return Err(ConfigError::PathNotFound(path.clone()));
            }
        }

        for ext in &args.extensions {
            if ext.is_empty() {
                return Err(ConfigError::EmptyExtension);
            }

            if ext.contains('.') {
                return Err(ConfigError::InvalidExtension(
                    ext.clone(),
                    "should not contain dots (use 'rs' not '.rs')".to_string()
                ));
            }

            if ext.contains('*') {
                return Err(ConfigError::InvalidExtension(
                    ext.clone(),
                    "should not contain wildcards".to_string()
                ));
            }
        }

        Ok(())
    }

    /// Returns true if any filters are active.
    pub fn has_filters(&self) -> bool {
        self.filter.has_filters()
    }

    /// Returns a description of active filters.
    pub fn describe_filters(&self) -> String {
        self.filter.describe()
    }
}

/// Configuration validation errors.
#[derive(Debug)]
pub enum ConfigError {
    /// No input files specified
    NoInputFiles,
    /// Invalid depth value
    InvalidDepth,
    /// Path not found
    PathNotFound(String),
    /// Empty extension specified
    EmptyExtension,
    /// Invalid extension format
    InvalidExtension(String, String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::NoInputFiles => write!(f, "no input files"),
            ConfigError::InvalidDepth => write!(f, "depth must be at least 1"),
            ConfigError::PathNotFound(path) => write!(f, "path not found: {}", path),
            ConfigError::EmptyExtension => write!(f, "empty extension is not allowed"),
            ConfigError::InvalidExtension(ext, reason) => {
                write!(f, "invalid extension '{}': {}", ext, reason)
            }
        }
    }
}

impl std::error::Error for ConfigError {}