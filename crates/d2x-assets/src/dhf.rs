//! DHF archive file format parser (Descent 1 & 2)
//!
//! DHF files are simple archive formats used by Descent 1 and 2 to package game assets.
//!
//! ## File Format
//! ```text
//! [Optional Header: "DHF" + 0x00]
//! For each file:
//!   - filename: 13 bytes (null-terminated)
//!   - size: u32 (little-endian)
//!   - data: [size] bytes
//! ```

use crate::error::{AssetError, Result};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// DHF archive containing multiple files
pub struct DhfArchive {
    file: File,
    entries: BTreeMap<String, DhfEntry>,
}

/// Entry in a DHF archive
#[derive(Debug, Clone)]
pub struct DhfEntry {
    /// Filename (uppercase)
    pub name: String,
    /// Offset in the archive
    pub offset: u64,
    /// Size in bytes
    pub size: u32,
}

impl DhfArchive {
    /// Open a DHF archive file
    ///
    /// # Example
    /// ```ignore
    /// use d2x_assets::DhfArchive;
    ///
    /// let hog = DhfArchive::open("descent2.hog")?;
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        let entries = Self::parse_entries(&mut file)?;

        Ok(Self { file, entries })
    }

    /// Parse DHF format entries
    fn parse_entries(file: &mut File) -> Result<BTreeMap<String, DhfEntry>> {
        let mut entries = BTreeMap::new();
        let mut current_offset = 0u64;

        // Check for optional DHF signature
        let mut sig = [0u8; 3];
        if file.read_exact(&mut sig).is_ok() && &sig == b"DHF" {
            current_offset = 3;
        } else {
            file.seek(SeekFrom::Start(0))?;
        }

        loop {
            // Read filename (13 bytes)
            let mut name_bytes = [0u8; 13];
            match file.read_exact(&mut name_bytes) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            }

            // Parse null-terminated filename
            let name_len = name_bytes.iter().position(|&b| b == 0).unwrap_or(13);
            let name = String::from_utf8_lossy(&name_bytes[..name_len]).to_uppercase();

            // Read size (4 bytes, little-endian)
            let mut size_bytes = [0u8; 4];
            file.read_exact(&mut size_bytes)?;
            let size = u32::from_le_bytes(size_bytes);

            current_offset += 17; // 13 + 4

            // Store entry
            entries.insert(
                name.clone(),
                DhfEntry {
                    name,
                    offset: current_offset,
                    size,
                },
            );

            // Skip file data
            file.seek(SeekFrom::Current(size as i64))?;
            current_offset += size as u64;
        }

        Ok(entries)
    }

    /// Check if a file exists in the archive
    ///
    /// File names are case-insensitive.
    pub fn contains_file(&self, name: &str) -> bool {
        self.entries.contains_key(&name.to_uppercase())
    }

    /// Get entry information for a file
    pub fn get_entry(&self, name: &str) -> Option<&DhfEntry> {
        self.entries.get(&name.to_uppercase())
    }

    /// Read a file from the archive
    ///
    /// # Example
    /// ```ignore
    /// let data = hog.read_file("LEVEL01.RDL")?;
    /// ```
    pub fn read_file(&mut self, name: &str) -> Result<Vec<u8>> {
        let entry = self
            .get_entry(name)
            .ok_or_else(|| AssetError::FileNotFound(name.to_string()))?;

        let mut buffer = vec![0u8; entry.size as usize];
        self.file.seek(SeekFrom::Start(entry.offset))?;
        self.file.read_exact(&mut buffer)?;

        Ok(buffer)
    }

    /// Get number of files in archive
    pub fn file_count(&self) -> usize {
        self.entries.len()
    }

    /// Iterate over all entries
    pub fn entries(&self) -> impl Iterator<Item = &DhfEntry> {
        self.entries.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Create a test DHF format file
    fn create_test_dhf() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();

        // Write DHF signature
        file.write_all(b"DHF").unwrap();

        // Write first file entry
        let mut filename = [0u8; 13];
        filename[..9].copy_from_slice(b"TEST1.TXT");
        file.write_all(&filename).unwrap();
        file.write_all(&10u32.to_le_bytes()).unwrap();
        file.write_all(b"Hello Test").unwrap();

        // Write second file entry
        let mut filename = [0u8; 13];
        filename[..9].copy_from_slice(b"TEST2.TXT");
        file.write_all(&filename).unwrap();
        file.write_all(&12u32.to_le_bytes()).unwrap();
        file.write_all(b"Second File!").unwrap();

        file.flush().unwrap();
        file
    }

    #[test]
    fn test_dhf_entry_parsing() {
        let dhf_file = create_test_dhf();
        let dhf = DhfArchive::open(dhf_file.path()).unwrap();

        assert_eq!(dhf.file_count(), 2);
        assert!(dhf.contains_file("TEST1.TXT"));
        assert!(dhf.contains_file("test1.txt")); // case insensitive

        let entry = dhf.get_entry("TEST1.TXT").unwrap();
        assert_eq!(entry.name, "TEST1.TXT");
        assert_eq!(entry.size, 10);
    }

    #[test]
    fn test_dhf_file_reading() {
        let dhf_file = create_test_dhf();
        let mut dhf = DhfArchive::open(dhf_file.path()).unwrap();

        let data = dhf.read_file("TEST1.TXT").unwrap();
        assert_eq!(data, b"Hello Test");

        let data = dhf.read_file("test2.txt").unwrap(); // case insensitive
        assert_eq!(data, b"Second File!");
    }

    #[test]
    fn test_case_insensitive_lookup() {
        let dhf_file = create_test_dhf();
        let dhf = DhfArchive::open(dhf_file.path()).unwrap();

        assert!(dhf.contains_file("TEST1.TXT"));
        assert!(dhf.contains_file("test1.txt"));
        assert!(dhf.contains_file("TeSt1.TxT"));
        assert!(!dhf.contains_file("nonexistent.txt"));
    }

    #[test]
    fn test_entries_iteration() {
        let dhf_file = create_test_dhf();
        let dhf = DhfArchive::open(dhf_file.path()).unwrap();

        let entries: Vec<_> = dhf.entries().collect();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().any(|e| e.name == "TEST1.TXT"));
        assert!(entries.iter().any(|e| e.name == "TEST2.TXT"));
    }

    #[test]
    fn test_file_not_found() {
        let dhf_file = create_test_dhf();
        let mut dhf = DhfArchive::open(dhf_file.path()).unwrap();

        let result = dhf.read_file("nonexistent.txt");
        assert!(matches!(result, Err(AssetError::FileNotFound(_))));
    }
}
