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

pub struct FileValidator;

impl FileValidator {
    /// ファイルパスの基本的な検証
    pub fn validate_path(path: &str) -> Result<(), ValidationError> {
        let path_obj = Path::new(path);

        // ファイルが存在するかチェック
        if !path_obj.exists() {
            return Err(ValidationError::FileNotFound);
        }

        // ファイルかどうかチェック（ディレクトリではない）
        if !path_obj.is_file() {
            return Err(ValidationError::FileNotFound);
        }

        Ok(())
    }

    /// ファイルサイズが適切かチェック（デフォルト: 10MB制限）
    pub fn validate_file_size(path: &str, max_size_mb: Option<u64>) -> Result<(), ValidationError> {
        let max_bytes = max_size_mb.unwrap_or(10) * 1024 * 1024; // デフォルト10MB

        let metadata = fs::metadata(path)?;
        if metadata.len() > max_bytes {
            return Err(ValidationError::FileTooLarge);
        }

        Ok(())
    }

    /// 表示に安全そうかチェック（v0.1版：基本的なnullバイトチェック）
    pub fn is_safe_to_display(path: &str) -> Result<bool, ValidationError> {
        // 最初の1024バイトだけをチェック
        let mut buffer = vec![0u8; 1024];
        let file = fs::File::open(path)?;

        use std::io::Read;
        let mut handle = file.take(1024);
        let bytes_read = handle.read(&mut buffer)?;
        buffer.truncate(bytes_read);

        // nullバイトがあったら危険と判定
        if buffer.contains(&0) {
            return Ok(false);
        }

        // 制御文字が多すぎたら危険と判定
        let control_char_count = buffer.iter()
            .filter(|&&b| b < 32 && b != b'\n' && b != b'\r' && b != b'\t')
            .count();

        let ratio = control_char_count as f64 / buffer.len() as f64;
        Ok(ratio < 0.1) // 10%以下なら安全とみなす
    }

    /// 一括検証（よく使う検証をまとめて実行）
    pub fn quick_validate(path: &str) -> Result<(), ValidationError> {
        Self::validate_path(path)?;
        Self::validate_file_size(path, None)?;

        if !Self::is_safe_to_display(path)? {
            return Err(ValidationError::SuspiciousContent);
        }

        Ok(())
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