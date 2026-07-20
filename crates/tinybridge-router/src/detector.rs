use crate::error::Result;
use std::path::Path;

/// Detected binary format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryFormat {
    /// macOS/Apple binary (Mach-O)
    MachO,
    /// Linux binary (ELF)
    Elf,
    /// Shell script or interpreter script
    Script,
    /// Unknown or unsupported format
    Unknown,
}

impl std::fmt::Display for BinaryFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryFormat::MachO => write!(f, "mach-o"),
            BinaryFormat::Elf => write!(f, "elf"),
            BinaryFormat::Script => write!(f, "script"),
            BinaryFormat::Unknown => write!(f, "unknown"),
        }
    }
}

/// Binary format detector
pub struct BinaryDetector;

impl BinaryDetector {
    /// Detect binary format by reading magic bytes
    pub fn detect_format(path: impl AsRef<Path>) -> Result<BinaryFormat> {
        let path = path.as_ref();
        let bytes = std::fs::read(path)?;

        if bytes.len() < 4 {
            return Ok(BinaryFormat::Script);
        }

        // Check for ELF magic bytes (0x7F 0x45 0x4C 0x46)
        if bytes[0..4] == [0x7F, 0x45, 0x4C, 0x46] {
            return Ok(BinaryFormat::Elf);
        }

        // Check for Mach-O magic bytes (0xFE 0xED 0xFA 0xCE or variants)
        if bytes[0..4] == [0xFE, 0xED, 0xFA, 0xCE]
            || bytes[0..4] == [0xFE, 0xED, 0xFA, 0xCF]
            || bytes[0..4] == [0xCE, 0xFA, 0xED, 0xFE]
            || bytes[0..4] == [0xCF, 0xFA, 0xED, 0xFE]
        {
            return Ok(BinaryFormat::MachO);
        }

        // Check for shebang (#!/)
        if bytes.len() >= 2 && bytes[0] == b'#' && bytes[1] == b'!' {
            return Ok(BinaryFormat::Script);
        }

        Ok(BinaryFormat::Unknown)
    }

    /// Detect binary type by file extension
    pub fn detect_by_extension(path: impl AsRef<Path>) -> Result<BinaryFormat> {
        let path = path.as_ref();
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match extension {
            "sh" | "bash" | "zsh" | "py" | "rb" | "pl" | "js" | "ts" => Ok(BinaryFormat::Script),
            "o" | "a" | "dylib" | "so" => {
                // Library files - need to detect actual format
                Self::detect_format(path)
            }
            _ => Ok(BinaryFormat::Unknown),
        }
    }

    /// Detect architecture from binary
    pub fn detect_architecture(path: impl AsRef<Path>) -> Result<String> {
        let path = path.as_ref();
        let bytes = std::fs::read(path)?;

        if bytes.len() < 20 {
            return Ok("unknown".to_string());
        }

        // Check ELF architecture
        if bytes[0..4] == [0x7F, 0x45, 0x4C, 0x46] {
            let arch_byte = bytes[18];
            return Ok(match arch_byte {
                0x03 => "x86".to_string(),
                0x3E => "x86_64".to_string(),
                0xB7 => "arm64".to_string(),
                0x28 => "arm".to_string(),
                _ => format!("elf_{}", arch_byte),
            });
        }

        // Check Mach-O architecture
        if [0xFE, 0xED, 0xFA, 0xCE, 0xCE, 0xFA, 0xED, 0xFE]
            .iter()
            .any(|&b| bytes[0..4].starts_with(&[b; 1]))
        {
            // Mach-O CPU type at offset 4
            let cpu_type = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
            return Ok(match cpu_type {
                0x00000007 => "i386".to_string(),
                0x01000007 => "x86_64".to_string(),
                0x0000000C => "arm".to_string(),
                0x0100000C => "arm64".to_string(),
                _ => format!("macho_{}", cpu_type),
            });
        }

        Ok("unknown".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_detect_elf_format() {
        let mut file = NamedTempFile::new().unwrap();
        // ELF magic bytes
        file.write_all(&[0x7F, 0x45, 0x4C, 0x46, 0x02, 0x01, 0x01]).unwrap();
        file.flush().unwrap();

        let result = BinaryDetector::detect_format(file.path()).unwrap();
        assert_eq!(result, BinaryFormat::Elf);
    }

    #[test]
    fn test_detect_script_shebang() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"#!/bin/bash\necho hello").unwrap();
        file.flush().unwrap();

        let result = BinaryDetector::detect_format(file.path()).unwrap();
        assert_eq!(result, BinaryFormat::Script);
    }

    #[test]
    fn test_detect_by_extension() {
        assert_eq!(
            BinaryDetector::detect_by_extension("script.sh").unwrap(),
            BinaryFormat::Script
        );
        assert_eq!(
            BinaryDetector::detect_by_extension("train.py").unwrap(),
            BinaryFormat::Script
        );
    }
}
