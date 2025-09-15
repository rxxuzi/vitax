//! File system operations and directory traversal utilities.

use std::fs;
use std::io;
use std::path::Path;

/// Reads the entire contents of a file into a string.
pub fn read_file_content(filename: &str) -> Result<String, io::Error> {
    fs::read_to_string(filename)
}

/// Returns a sorted list of directory entries.
///
/// Directories are listed first, followed by files, both sorted alphabetically.
pub fn read_directory_entries(dir_path: &str) -> Result<Vec<DirectoryEntry>, io::Error> {
    let mut entries = Vec::new();
    let dir = fs::read_dir(dir_path)?;

    for entry in dir {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;

        let entry_info = DirectoryEntry {
            name: path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("???")
                .to_string(),
            path: path.to_string_lossy().to_string(),
            is_directory: metadata.is_dir(),
            is_file: metadata.is_file(),
            size: if metadata.is_file() { Some(metadata.len()) } else { None },
        };

        entries.push(entry_info);
    }

    entries.sort_by(|a, b| {
        match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });

    Ok(entries)
}

/// Determines the type of path (file, directory, or other).
pub fn check_path_type(path: &str) -> Result<PathType, io::Error> {
    let path_obj = Path::new(path);

    if !path_obj.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Path does not exist"));
    }

    if path_obj.is_dir() {
        Ok(PathType::Directory)
    } else if path_obj.is_file() {
        Ok(PathType::File)
    } else {
        Ok(PathType::Other)
    }
}

/// Recursively walks a directory and returns all file paths.
///
/// # Arguments
/// * `dir_path` - The directory to traverse
/// * `max_depth` - Maximum recursion depth (None for unlimited)
pub fn walk_directory(dir_path: &str, max_depth: Option<usize>) -> Result<Vec<String>, io::Error> {
    let mut files = Vec::new();
    walk_directory_recursive(dir_path, max_depth.unwrap_or(100), 0, &mut files)?;
    Ok(files)
}

fn walk_directory_recursive(
    current_path: &str,
    max_depth: usize,
    current_depth: usize,
    files: &mut Vec<String>
) -> Result<(), io::Error> {
    if current_depth >= max_depth {
        return Ok(());
    }

    let entries = read_directory_entries(current_path)?;

    for entry in entries {
        if entry.is_file {
            files.push(entry.path.clone());
        } else if entry.is_directory {
            walk_directory_recursive(&entry.path, max_depth, current_depth + 1, files)?;
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub is_file: bool,
    pub size: Option<u64>,
}

#[derive(Debug, PartialEq)]
pub enum PathType {
    File,
    Directory,
    Other,
}