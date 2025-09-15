mod io;
mod detector;
mod validator;
mod cli;

use std::process;
use std::path::Path;
use clap::Parser;
use detector::{FileDetector, FileType};
use validator::FileValidator;
use cli::Args;

fn main() {
    let args = Args::parse();

    if let Err(e) = args.validate() {
        eprintln!("vitax: fatal error: {}", e);
        eprintln!("Usage: vitax <path> [paths...] [options]");
        process::exit(1);
    }

    for (index, path) in args.paths.iter().enumerate() {
        if index > 0 {
            println!("\n{}", "=".repeat(80));
            println!();
        }
        process_single_path(path, &args);
    }
}

fn process_single_path(path: &str, args: &Args) {
    let base_path = match std::fs::canonicalize(path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error resolving path '{}': {}", path, e);
            return;
        }
    };

    match io::check_path_type(path) {
        Ok(io::PathType::Directory) => process_directory(path, &base_path, args),
        Ok(io::PathType::File) => {
            if !args.should_ignore(path) {
                process_file(path, &base_path, true);
            }
        },
        Ok(io::PathType::Other) => eprintln!("Unsupported path type: {}", path),
        Err(e) => eprintln!("Error accessing path '{}': {}", path, e),
    }
}

fn process_directory(path: &str, base_path: &Path, args: &Args) {
    println!("================================================================================");
    println!("{}/", base_path.display());
    println!("================================================================================");

    match io::walk_directory(path, Some(args.max_depth)) {
        Ok(files) => {
            for file in files {
                if !args.should_ignore(&file) {
                    process_file(&file, base_path, false);
                }
            }
        }
        Err(e) => eprintln!("Error walking directory '{}': {}", path, e),
    }
}

fn process_file(path: &str, base_path: &Path, is_root: bool) {
    if let Err(e) = FileValidator::quick_validate(path) {
        eprintln!("Skipping file '{}': {}", path, e);
        return;
    }

    let display_path = if is_root {
        format!("================================================================================\n{}\n================================================================================", path)
    } else {
        let file_path = match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(_) => {
                eprintln!("Error resolving path: {}", path);
                return;
            }
        };

        let relative_path = match file_path.strip_prefix(base_path) {
            Ok(rel) => format!("./{}", rel.display()),
            Err(_) => path.to_string(),
        };

        format!("--------------------------------------------------------------------------------\n{}\n--------------------------------------------------------------------------------", relative_path)
    };

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
                Err(e) => eprintln!("Error reading file '{}': {}", path, e),
            }
        }
        Err(e) => eprintln!("Error detecting file type '{}': {}", path, e),
    }
}