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
    pub fn detect_file_type(path: &str) -> Result<FileType, io::Error> {
        let bytes = fs::read(path)?;
        let sample_size = std::cmp::min(bytes.len(), BINARY_CHECK_BYTES);
        let sample = &bytes[..sample_size];

        if sample.contains(&0) {
            return Ok(FileType::Binary);
        }

        if Self::is_valid_utf8(sample) || Self::is_valid_shift_jis(sample) {
            return Ok(FileType::Text);
        }

        if Self::calculate_non_printable_ratio(sample) > 0.25 {
            Ok(FileType::Binary)
        } else {
            Ok(FileType::Text)
        }
    }

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

    fn is_valid_utf8(bytes: &[u8]) -> bool {
        let mut i = 0;
        while i < bytes.len() {
            let byte = bytes[i];

            if byte <= 0x7F {
                i += 1;
            } else if (byte & 0xE0) == 0xC0 {
                if i + 1 >= bytes.len() || (bytes[i + 1] & 0xC0) != 0x80 {
                    return false;
                }
                i += 2;
            } else if (byte & 0xF0) == 0xE0 {
                if i + 2 >= bytes.len()
                    || (bytes[i + 1] & 0xC0) != 0x80
                    || (bytes[i + 2] & 0xC0) != 0x80 {
                    return false;
                }
                i += 3;
            } else if (byte & 0xF8) == 0xF0 {
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

    fn is_valid_shift_jis(bytes: &[u8]) -> bool {
        let mut i = 0;
        while i < bytes.len() {
            let byte = bytes[i];

            if byte <= 0x7F || (0xA1..=0xDF).contains(&byte) {
                i += 1;
            } else if (0x81..=0x9F).contains(&byte) || (0xE0..=0xEF).contains(&byte) {
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

    fn calculate_non_printable_ratio(bytes: &[u8]) -> f64 {
        if bytes.is_empty() {
            return 0.0;
        }

        let non_printable_count = bytes.iter()
            .filter(|&&b| !Self::is_printable_ascii(b))
            .count();

        non_printable_count as f64 / bytes.len() as f64
    }

    fn is_printable_ascii(byte: u8) -> bool {
        (32..=126).contains(&byte) || matches!(byte, b'\n' | b'\r' | b'\t')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf8_validation() {
        let utf8_bytes = "Hello, world".as_bytes();
        assert!(FileDetector::is_valid_utf8(utf8_bytes));

        let invalid_utf8 = &[0xFF, 0xFE];
        assert!(!FileDetector::is_valid_utf8(invalid_utf8));
    }

    #[test]
    fn test_binary_detection() {
        let text_data = "Hello world".as_bytes();
        assert!(FileDetector::calculate_non_printable_ratio(text_data) < 0.25);
    }
}