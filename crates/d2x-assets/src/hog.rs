//! HOG archive file format parser
//!
//! HOG files are simple archive formats used by Descent to package game assets.
//!
//! ## Corresponds to D2X-XL Code
//! - `include/hogfile.h`: CHogFile class
//! - `io/hogfile.cpp`: CHogFile implementation
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

/// HOG archive containing multiple files
pub struct HogArchive {
    file: File,
    entries: BTreeMap<String, HogEntry>,
}

/// Entry in a HOG archive
#[derive(Debug, Clone)]
pub struct HogEntry {
    /// Filename (uppercase)
    pub name: String,
    /// Offset in the archive
    pub offset: u64,
    /// Size in bytes
    pub size: u32,
}

impl HogArchive {
    /// Open a HOG archive file
    ///
    /// # Example
    /// ```ignore
    /// use d2x_assets::HogArchive;
    ///
    /// let hog = HogArchive::open("descent2.hog")?;
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        let entries = Self::parse_entries(&mut file)?;

        Ok(Self { file, entries })
    }

    /// Parse HOG file entries
    fn parse_entries(file: &mut File) -> Result<BTreeMap<String, HogEntry>> {
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
                HogEntry {
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
    pub fn get_entry(&self, name: &str) -> Option<&HogEntry> {
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
    pub fn entries(&self) -> impl Iterator<Item = &HogEntry> {
        self.entries.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_parsing() {
        // Test will require actual descent2.hog file
        // This is a placeholder
    }

    #[test]
    fn test_case_insensitive_lookup() {
        // Verify uppercase/lowercase names work
    }
}
