use std::io::Read;
use std::path::Path;

use crate::error::{Result, TunnelError};

/// Detected binary format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryFormat {
    /// ELF binary (Linux)
    Elf,
    /// Mach-O binary (macOS)
    MachO,
    /// Script with shebang (bash, python, etc.)
    Script,
    /// Unknown or no magic bytes
    Unknown,
}

impl BinaryFormat {
    /// Detect binary format from a file path
    pub fn detect_from_file<P: AsRef<Path>>(path: P) -> Result<BinaryFormat> {
        let mut file = std::fs::File::open(path.as_ref())
            .map_err(|e| TunnelError::IoError(e))?;

        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)
            .map_err(|e| TunnelError::IoError(e))?;

        Ok(Self::from_magic_bytes(&magic))
    }

    /// Detect binary format from magic bytes
    pub fn from_magic_bytes(bytes: &[u8]) -> BinaryFormat {
        if bytes.len() < 2 {
            return BinaryFormat::Unknown;
        }

        // ELF: 0x7F 'E' 'L' 'F'
        if bytes.len() >= 4 && bytes[0] == 0x7F && bytes[1] == b'E' && bytes[2] == b'L' && bytes[3] == b'F' {
            return BinaryFormat::Elf;
        }

        // Mach-O (big-endian): 0xFE ED FA [CE|CF]
        if bytes.len() >= 4 && bytes[0] == 0xFE && bytes[1] == 0xED && bytes[2] == 0xFA {
            if bytes[3] == 0xCE || bytes[3] == 0xCF {
                return BinaryFormat::MachO;
            }
        }

        // Mach-O (little-endian): 0xFE ED FA [CE|CF] reversed
        if bytes.len() >= 4 && bytes[0] == 0xCE && bytes[1] == 0xFA && bytes[2] == 0xED && bytes[3] == 0xFE {
            return BinaryFormat::MachO;
        }

        if bytes.len() >= 4 && bytes[0] == 0xCF && bytes[1] == 0xFA && bytes[2] == 0xED && bytes[3] == 0xFE {
            return BinaryFormat::MachO;
        }

        // Shebang: #!
        if bytes[0] == b'#' && bytes[1] == b'!' {
            return BinaryFormat::Script;
        }

        BinaryFormat::Unknown
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            BinaryFormat::Elf => "ELF",
            BinaryFormat::MachO => "Mach-O",
            BinaryFormat::Script => "Script",
            BinaryFormat::Unknown => "Unknown",
        }
    }

    /// Does this format require Linux execution?
    pub fn requires_linux(&self) -> bool {
        matches!(self, BinaryFormat::Elf)
    }

    /// Does this format run on macOS?
    pub fn runs_on_macos(&self) -> bool {
        matches!(self, BinaryFormat::MachO | BinaryFormat::Script)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elf_detection() {
        let elf_magic = [0x7F, b'E', b'L', b'F'];
        assert_eq!(BinaryFormat::from_magic_bytes(&elf_magic), BinaryFormat::Elf);
    }

    #[test]
    fn test_macho_detection() {
        // Big-endian Mach-O 32-bit
        let macho_be_32 = [0xFE, 0xED, 0xFA, 0xCE];
        assert_eq!(BinaryFormat::from_magic_bytes(&macho_be_32), BinaryFormat::MachO);

        // Big-endian Mach-O 64-bit
        let macho_be_64 = [0xFE, 0xED, 0xFA, 0xCF];
        assert_eq!(BinaryFormat::from_magic_bytes(&macho_be_64), BinaryFormat::MachO);

        // Little-endian Mach-O 32-bit
        let macho_le_32 = [0xCE, 0xFA, 0xED, 0xFE];
        assert_eq!(BinaryFormat::from_magic_bytes(&macho_le_32), BinaryFormat::MachO);

        // Little-endian Mach-O 64-bit
        let macho_le_64 = [0xCF, 0xFA, 0xED, 0xFE];
        assert_eq!(BinaryFormat::from_magic_bytes(&macho_le_64), BinaryFormat::MachO);
    }

    #[test]
    fn test_shebang_detection() {
        let shebang = [b'#', b'!', b'/', b'b'];
        assert_eq!(BinaryFormat::from_magic_bytes(&shebang), BinaryFormat::Script);
    }

    #[test]
    fn test_unknown_detection() {
        let unknown = [0xFF, 0xFF, 0xFF, 0xFF];
        assert_eq!(BinaryFormat::from_magic_bytes(&unknown), BinaryFormat::Unknown);
    }

    #[test]
    fn test_requires_linux() {
        assert!(BinaryFormat::Elf.requires_linux());
        assert!(!BinaryFormat::MachO.requires_linux());
        assert!(!BinaryFormat::Script.requires_linux());
        assert!(!BinaryFormat::Unknown.requires_linux());
    }

    #[test]
    fn test_runs_on_macos() {
        assert!(!BinaryFormat::Elf.runs_on_macos());
        assert!(BinaryFormat::MachO.runs_on_macos());
        assert!(BinaryFormat::Script.runs_on_macos());
        assert!(!BinaryFormat::Unknown.runs_on_macos());
    }
}
