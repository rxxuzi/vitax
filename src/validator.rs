//! File validation and safety checking utilities.

use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug)]
pub enum ValidationError {
    FileNotFound,
    PermissionDenied,
    FileTooLarge,
    SuspiciousContent,
    IoError(io::Error),
}

impl From<io::Error> for ValidationError {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::NotFound => ValidationError::FileNotFound,
            io::ErrorKind::PermissionDenied => ValidationError::PermissionDenied,
            _ => ValidationError::IoError(error),
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::FileNotFound => write!(f, "File not found or not accessible"),
            ValidationError::PermissionDenied => write!(f, "Permission denied"),
            ValidationError::FileTooLarge => write!(f, "File is too large"),
            ValidationError::SuspiciousContent => write!(f, "File contains suspicious content"),
            ValidationError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

pub struct FileValidator;

impl FileValidator {
    /// Validates basic file path requirements.
    pub fn validate_path(path: &str) -> Result<(), ValidationError> {
        let path_obj = Path::new(path);

        if !path_obj.exists() {
            return Err(ValidationError::FileNotFound);
        }

        if !path_obj.is_file() {
            return Err(ValidationError::FileNotFound);
        }

        Ok(())
    }

    /// Checks if file size is within acceptable limits.
    ///
    /// # Arguments
    /// * `path` - File path to check
    /// * `max_size_mb` - Maximum size in MB (defaults to 10MB)
    pub fn validate_file_size(path: &str, max_size_mb: Option<u64>) -> Result<(), ValidationError> {
        let max_bytes = max_size_mb.unwrap_or(10) * 1024 * 1024;

        let metadata = fs::metadata(path)?;
        if metadata.len() > max_bytes {
            return Err(ValidationError::FileTooLarge);
        }

        Ok(())
    }

    /// Checks if file content is safe for terminal display.
    ///
    /// Examines the first 1024 bytes for null bytes and excessive control characters.
    pub fn is_safe_to_display(path: &str) -> Result<bool, ValidationError> {
        let mut buffer = vec![0u8; 1024];
        let file = fs::File::open(path)?;

        use std::io::Read;
        let mut handle = file.take(1024);
        let bytes_read = handle.read(&mut buffer)?;
        buffer.truncate(bytes_read);

        if buffer.contains(&0) {
            return Ok(false);
        }

        let control_char_count = buffer.iter()
            .filter(|&&b| b < 32 && !matches!(b, b'\n' | b'\r' | b'\t'))
            .count();

        let ratio = control_char_count as f64 / buffer.len() as f64;
        Ok(ratio < 0.1)
    }

    /// Performs comprehensive file validation.
    ///
    /// Combines path validation, size checking, and safety assessment.
    pub fn quick_validate(path: &str) -> Result<(), ValidationError> {
        Self::validate_path(path)?;
        Self::validate_file_size(path, None)?;

        if !Self::is_safe_to_display(path)? {
            return Err(ValidationError::SuspiciousContent);
        }

        Ok(())
    }
}