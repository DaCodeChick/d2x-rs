//! HOG2 archive file format parser (Descent 3)
//!
//! HOG2 files are the enhanced archive format used by Descent 3 to package game assets.
//!
//! ## File Format
//! ```text
//! Header:
//!   - signature: 4 bytes ("HOG2")
//!   - file_count: u32 (little-endian)
//!   - data_offset: u32 (little-endian) - offset to start of file data
//! For each file:
//!   - filename: 36 bytes (null-terminated)
//!   - flags: u32 (little-endian)
//!   - size: u32 (little-endian)
//!   - timestamp: u32 (little-endian)
//! [Padding to data_offset]
//! File data sections
//! ```

use crate::error::{AssetError, Result};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// HOG2 archive containing multiple files
pub struct Hog2Archive {
    file: File,
    entries: BTreeMap<String, Hog2Entry>,
}

/// Entry in a HOG2 archive
#[derive(Debug, Clone)]
pub struct Hog2Entry {
    /// Filename (uppercase)
    pub name: String,
    /// Offset in the archive
    pub offset: u64,
    /// Size in bytes
    pub size: u32,
    /// File flags
    pub flags: u32,
    /// Unix timestamp
    pub timestamp: u32,
}

impl Hog2Archive {
    /// Open a HOG2 archive file
    ///
    /// # Example
    /// ```ignore
    /// use d2x_assets::Hog2Archive;
    ///
    /// let hog = Hog2Archive::open("d3.hog")?;
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        let entries = Self::parse_entries(&mut file)?;

        Ok(Self { file, entries })
    }

    /// Parse HOG2 format entries
    fn parse_entries(file: &mut File) -> Result<BTreeMap<String, Hog2Entry>> {
        file.seek(SeekFrom::Start(0))?;

        // Read HOG2 header
        let mut sig = [0u8; 4];
        file.read_exact(&mut sig)?;
        if &sig != b"HOG2" {
            return Err(AssetError::InvalidFormat(
                "Expected HOG2 signature".to_string(),
            ));
        }

        let mut buf = [0u8; 4];
        file.read_exact(&mut buf)?;
        let file_count = u32::from_le_bytes(buf);

        file.read_exact(&mut buf)?;
        let data_offset = u32::from_le_bytes(buf) as u64;

        // Read file entries
        let mut entries = BTreeMap::new();
        let mut current_data_offset = data_offset;

        for _ in 0..file_count {
            // Read filename (36 bytes)
            let mut name_bytes = [0u8; 36];
            file.read_exact(&mut name_bytes)?;

            // Parse null-terminated filename
            let name_len = name_bytes.iter().position(|&b| b == 0).unwrap_or(36);
            let name = String::from_utf8_lossy(&name_bytes[..name_len]).to_uppercase();

            // Read flags
            file.read_exact(&mut buf)?;
            let flags = u32::from_le_bytes(buf);

            // Read size
            file.read_exact(&mut buf)?;
            let size = u32::from_le_bytes(buf);

            // Read timestamp
            file.read_exact(&mut buf)?;
            let timestamp = u32::from_le_bytes(buf);

            // Store entry
            entries.insert(
                name.clone(),
                Hog2Entry {
                    name,
                    offset: current_data_offset,
                    size,
                    flags,
                    timestamp,
                },
            );

            current_data_offset += size as u64;
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
    pub fn get_entry(&self, name: &str) -> Option<&Hog2Entry> {
        self.entries.get(&name.to_uppercase())
    }

    /// Read a file from the archive
    ///
    /// # Example
    /// ```ignore
    /// let data = hog.read_file("level1.d3l")?;
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
    pub fn entries(&self) -> impl Iterator<Item = &Hog2Entry> {
        self.entries.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Create a test HOG2 format file
    fn create_test_hog2() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();

        // Write HOG2 header
        file.write_all(b"HOG2").unwrap();
        file.write_all(&2u32.to_le_bytes()).unwrap(); // file count
        let data_offset: u32 = 12 + 2 * 48; // header + 2 entries
        file.write_all(&data_offset.to_le_bytes()).unwrap();

        // Write first file entry
        let mut filename = [0u8; 36];
        filename[..9].copy_from_slice(b"TEST1.TXT");
        file.write_all(&filename).unwrap();
        file.write_all(&0u32.to_le_bytes()).unwrap(); // flags
        file.write_all(&10u32.to_le_bytes()).unwrap(); // size
        file.write_all(&1234567890u32.to_le_bytes()).unwrap(); // timestamp

        // Write second file entry
        let mut filename = [0u8; 36];
        filename[..9].copy_from_slice(b"TEST2.TXT");
        file.write_all(&filename).unwrap();
        file.write_all(&0u32.to_le_bytes()).unwrap(); // flags
        file.write_all(&12u32.to_le_bytes()).unwrap(); // size
        file.write_all(&1234567891u32.to_le_bytes()).unwrap(); // timestamp

        // Write file data
        file.write_all(b"Hello Test").unwrap();
        file.write_all(b"Second File!").unwrap();

        file.flush().unwrap();
        file
    }

    #[test]
    fn test_hog2_entry_parsing() {
        let hog2_file = create_test_hog2();
        let hog = Hog2Archive::open(hog2_file.path()).unwrap();

        assert_eq!(hog.file_count(), 2);
        assert!(hog.contains_file("TEST1.TXT"));
        assert!(hog.contains_file("test2.txt")); // case insensitive

        let entry = hog.get_entry("TEST1.TXT").unwrap();
        assert_eq!(entry.name, "TEST1.TXT");
        assert_eq!(entry.size, 10);
        assert_eq!(entry.flags, 0);
        assert_eq!(entry.timestamp, 1234567890);

        let entry = hog.get_entry("TEST2.TXT").unwrap();
        assert_eq!(entry.timestamp, 1234567891);
    }

    #[test]
    fn test_hog2_file_reading() {
        let hog2_file = create_test_hog2();
        let mut hog = Hog2Archive::open(hog2_file.path()).unwrap();

        let data = hog.read_file("TEST1.TXT").unwrap();
        assert_eq!(data, b"Hello Test");

        let data = hog.read_file("test2.txt").unwrap(); // case insensitive
        assert_eq!(data, b"Second File!");
    }

    #[test]
    fn test_case_insensitive_lookup() {
        let hog2_file = create_test_hog2();
        let hog = Hog2Archive::open(hog2_file.path()).unwrap();

        assert!(hog.contains_file("TEST1.TXT"));
        assert!(hog.contains_file("test1.txt"));
        assert!(hog.contains_file("TeSt1.TxT"));
        assert!(!hog.contains_file("nonexistent.txt"));
    }

    #[test]
    fn test_entries_iteration() {
        let hog2_file = create_test_hog2();
        let hog = Hog2Archive::open(hog2_file.path()).unwrap();

        let entries: Vec<_> = hog.entries().collect();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().any(|e| e.name == "TEST1.TXT"));
        assert!(entries.iter().any(|e| e.name == "TEST2.TXT"));
    }

    #[test]
    fn test_file_not_found() {
        let hog2_file = create_test_hog2();
        let mut hog = Hog2Archive::open(hog2_file.path()).unwrap();

        let result = hog.read_file("nonexistent.txt");
        assert!(matches!(result, Err(AssetError::FileNotFound(_))));
    }

    #[test]
    fn test_invalid_signature() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"INVALID_SIG").unwrap();
        file.flush().unwrap();

        let result = Hog2Archive::open(file.path());
        assert!(matches!(result, Err(AssetError::InvalidFormat(_))));
    }
}
