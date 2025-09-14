// detector.rs
use std::fs;
use std::io;

const BINARY_CHECK_BYTES: usize = 2048;

#[derive(Debug, PartialEq)]
pub enum FileType {
    Text,
    Binary,
}

#[derive(Debug, PartialEq)]
pub enum Encoding {
    Utf8,
    ShiftJis,
    Unknown,
}

pub struct FileDetector;

impl FileDetector {
    /// ファイルがバイナリかテキストかを判定
    pub fn detect_file_type(path: &str) -> Result<FileType, io::Error> {
        let bytes = fs::read(path)?;
        let sample_size = std::cmp::min(bytes.len(), BINARY_CHECK_BYTES);
        let sample = &bytes[..sample_size];

        // null バイトがあったら即座にバイナリ
        if sample.contains(&0) {
            return Ok(FileType::Binary);
        }

        // UTF-8として有効かチェック
        if Self::is_valid_utf8(sample) {
            return Ok(FileType::Text);
        }

        // Shift-JISとして有効かチェック
        if Self::is_valid_shift_jis(sample) {
            return Ok(FileType::Text);
        }

        // 非印字文字の比率で判定
        let non_printable_ratio = Self::calculate_non_printable_ratio(sample);
        if non_printable_ratio > 0.25 {
            Ok(FileType::Binary)
        } else {
            Ok(FileType::Text)
        }
    }

    /// エンコーディングを判定
    pub fn detect_encoding(path: &str) -> Result<Encoding, io::Error> {
        let bytes = fs::read(path)?;
        let sample_size = std::cmp::min(bytes.len(), BINARY_CHECK_BYTES);
        let sample = &bytes[..sample_size];

        if Self::is_valid_utf8(sample) {
            Ok(Encoding::Utf8)
        } else if Self::is_valid_shift_jis(sample) {
            Ok(Encoding::ShiftJis)
        } else {
            Ok(Encoding::Unknown)
        }
    }

    /// UTF-8として有効かチェック
    fn is_valid_utf8(bytes: &[u8]) -> bool {
        let mut i = 0;
        while i < bytes.len() {
            let byte = bytes[i];

            if byte <= 0x7F {
                // ASCII (0xxxxxxx)
                i += 1;
            } else if (byte & 0xE0) == 0xC0 {
                // 2バイト文字 (110xxxxx 10xxxxxx)
                if i + 1 >= bytes.len() || (bytes[i + 1] & 0xC0) != 0x80 {
                    return false;
                }
                i += 2;
            } else if (byte & 0xF0) == 0xE0 {
                // 3バイト文字 (1110xxxx 10xxxxxx 10xxxxxx)
                if i + 2 >= bytes.len()
                    || (bytes[i + 1] & 0xC0) != 0x80
                    || (bytes[i + 2] & 0xC0) != 0x80 {
                    return false;
                }
                i += 3;
            } else if (byte & 0xF8) == 0xF0 {
                // 4バイト文字 (11110xxx 10xxxxxx 10xxxxxx 10xxxxxx)
                if i + 3 >= bytes.len()
                    || (bytes[i + 1] & 0xC0) != 0x80
                    || (bytes[i + 2] & 0xC0) != 0x80
                    || (bytes[i + 3] & 0xC0) != 0x80 {
                    return false;
                }
                i += 4;
            } else {
                return false;
            }
        }
        true
    }

    /// Shift-JISとして有効かチェック
    fn is_valid_shift_jis(bytes: &[u8]) -> bool {
        let mut i = 0;
        while i < bytes.len() {
            let byte = bytes[i];

            if byte <= 0x7F || (0xA1..=0xDF).contains(&byte) {
                // ASCII または 半角カタカナ
                i += 1;
            } else if (0x81..=0x9F).contains(&byte) || (0xE0..=0xEF).contains(&byte) {
                // 2バイト文字の1バイト目
                if i + 1 >= bytes.len() {
                    return false;
                }
                let second_byte = bytes[i + 1];
                if !((0x40..=0x7E).contains(&second_byte) || (0x80..=0xFC).contains(&second_byte)) {
                    return false;
                }
                i += 2;
            } else {
                return false;
            }
        }
        true
    }

    /// 非印字文字の比率を計算
    fn calculate_non_printable_ratio(bytes: &[u8]) -> f64 {
        if bytes.is_empty() {
            return 0.0;
        }

        let non_printable_count = bytes.iter()
            .filter(|&&b| !Self::is_printable_ascii(b))
            .count();

        non_printable_count as f64 / bytes.len() as f64
    }

    /// ASCII印字文字かチェック（改行、タブ、復帰文字は印字可能とみなす）
    fn is_printable_ascii(byte: u8) -> bool {
        (32..=126).contains(&byte) || byte == b'\n' || byte == b'\r' || byte == b'\t'
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf8_validation() {
        let utf8_bytes = "Hello, 世界".as_bytes();
        assert!(FileDetector::is_valid_utf8(utf8_bytes));

        let invalid_utf8 = &[0xFF, 0xFE];
        assert!(!FileDetector::is_valid_utf8(invalid_utf8));
    }

    #[test]
    fn test_binary_detection() {
        let binary_data = &[0x00, 0x01, 0x02];
        assert_eq!(0.0, FileDetector::calculate_non_printable_ratio(&[]));

        let text_data = "Hello world".as_bytes();
        assert!(FileDetector::calculate_non_printable_ratio(text_data) < 0.25);
    }
}