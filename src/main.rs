mod cli;
mod config;
mod detector;
mod filter;
mod io;
mod validator;

use std::path::Path;
use std::process;

use clap::Parser;
use config::Config;
use detector::{FileDetector, FileType};
use validator::{FileValidator, ValidationError};

fn main() {
    let args = cli::Args::parse();

    let config = match Config::from_args(args) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("vitax: fatal error: {}", e);
            eprintln!("Usage: vitax <path> [paths...] [options]");
            eprintln!("Try 'vitax --help' for more information.");
            process::exit(1);
        }
    };

    for (index, path) in config.paths.iter().enumerate() {
        if index > 0 {
            println!("\n{}", "=".repeat(80));
            println!();
        }
        process_single_path(path, &config);
    }
}

/// Processes a single path (file or directory).
///
/// # Arguments
/// * `path` - The path to process
/// * `config` - Application configuration
fn process_single_path(path: &str, config: &Config) {
    let base_path = match std::fs::canonicalize(path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error resolving path '{}': {}", path, e);
            return;
        }
    };

    match io::check_path_type(path) {
        Ok(io::PathType::Directory) => {
            process_directory(path, &base_path, config);
        }
        Ok(io::PathType::File) => {
            if config.filter.should_process(path) {
                process_file(path, &base_path, true, config);
            }
        }
        Ok(io::PathType::Other) => {
            eprintln!("Unsupported path type: {}", path);
        }
        Err(e) => {
            eprintln!("Error accessing path '{}': {}", path, e);
        }
    }
}

/// Processes a directory recursively.
///
/// # Arguments
/// * `path` - Directory path to process
/// * `base_path` - Base path for relative path calculation
/// * `config` - Application configuration
fn process_directory(path: &str, base_path: &Path, config: &Config) {
    println!("{}", "=".repeat(80));
    println!("{}/", base_path.display());
    println!("{}", "=".repeat(80));

    match io::walk_directory(path, Some(config.max_depth)) {
        Ok(files) => {
            for file in files {
                if config.filter.should_process(&file) {
                    process_file(&file, base_path, false, config);
                }
            }
        }
        Err(e) => {
            eprintln!("Error walking directory '{}': {}", path, e);
        }
    }
}

/// Processes a single file.
///
/// # Arguments
/// * `path` - File path to process
/// * `base_path` - Base path for relative path calculation
/// * `is_root` - Whether this is a root file (affects display formatting)
/// * `config` - Application configuration
fn process_file(path: &str, base_path: &Path, is_root: bool, config: &Config) {
    // Validate file first
    if let Err(e) = FileValidator::quick_validate(path) {
        if config.verbose {
            print_skipped_file(path, base_path, is_root, &e);
        }
        return;
    }

    let display_path = format_display_path(path, base_path, is_root);

    match FileDetector::detect_file_type(path) {
        Ok(FileType::Binary) => {
            println!("{}", display_path);
            println!("This is a binary file\n");
        }
        Ok(FileType::Text) => {
            match io::read_file_content(path) {
                Ok(contents) => {
                    println!("{}", display_path);
                    println!("{}\n", contents);
                }
                Err(e) => {
                    if config.verbose {
                        print_read_error(path, base_path, is_root, &e);
                    }
                }
            }
        }
        Err(e) => {
            if config.verbose {
                print_detection_error(path, base_path, is_root, &e);
            }
        }
    }
}

/// Prints information about a skipped file in verbose mode.
fn print_skipped_file(path: &str, base_path: &Path, is_root: bool, error: &ValidationError) {
    let display_path = format_display_path(path, base_path, is_root);
    println!("{}", display_path);
    println!("SKIPPED: {}\n", error);
}

/// Prints information about a file read error in verbose mode.
fn print_read_error(path: &str, base_path: &Path, is_root: bool, error: &std::io::Error) {
    let display_path = format_display_path(path, base_path, is_root);
    println!("{}", display_path);
    println!("READ ERROR: {}\n", error);
}

/// Prints information about a file type detection error in verbose mode.
fn print_detection_error(path: &str, base_path: &Path, is_root: bool, error: &std::io::Error) {
    let display_path = format_display_path(path, base_path, is_root);
    println!("{}", display_path);
    println!("DETECTION ERROR: {}\n", error);
}

/// Formats the display path for a file.
fn format_display_path(path: &str, base_path: &Path, is_root: bool) -> String {
    let separator = if is_root { "=" } else { "-" };
    let line = separator.repeat(80);

    if is_root {
        format!("{}\n{}\n{}", line, path, line)
    } else {
        let file_path = match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(_) => return format!("{}\n{}\n{}", line, path, line),
        };

        let relative_path = match file_path.strip_prefix(base_path) {
            Ok(rel) => format!("./{}", rel.display()),
            Err(_) => path.to_string(),
        };

        format!("{}\n{}\n{}", line, relative_path, line)
    }
}