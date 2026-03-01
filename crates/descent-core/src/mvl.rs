//! MVL (Movie Library) archive file format parser (Descent 1 & 2)
//!
//! MVL files are archive formats used by Descent 1 and 2 to package movie (MVE) files.
//! They are similar to HOG/DHF archives but specifically for video cutscenes.
//!
//! ## File Format
//! ```text
//! [Header]
//!   - signature: "DMVL" (4 bytes)
//!   - num_files: i32 (little-endian)
//!
//! [File Entries] (repeated num_files times)
//!   - filename: 13 bytes (null-terminated string)
//!   - size: i32 (little-endian)
//!
//! [File Data]
//!   - Concatenated file data in entry order
//! ```
//!
//! ## Example Libraries
//! - `intro-l.mvl` / `intro-h.mvl` - Intro movies (low/high resolution)
//! - `other-l.mvl` / `other-h.mvl` - Other cutscene movies
//! - `robots-l.mvl` / `robots-h.mvl` - Robot briefing movies
//! - `d2x-l.mvl` / `d2x-h.mvl` - Additional D2X movies
//!
//! ## Example
//! ```ignore
//! use descent_core::MvlArchive;
//!
//! let mut mvl = MvlArchive::open("intro-l.mvl")?;
//!
//! // List all movies
//! for entry in mvl.entries() {
//!     println!("{} - {} bytes", entry.name, entry.size);
//! }
//!
//! // Extract a movie
//! let movie_data = mvl.read_file("intro.mve")?;
//! ```

use crate::error::{AssetError, Result};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// MVL archive containing multiple MVE movie files
pub struct MvlArchive {
    file: File,
    entries: BTreeMap<String, MvlEntry>,
}

/// Entry in an MVL archive
#[derive(Debug, Clone)]
pub struct MvlEntry {
    /// Filename (uppercase)
    pub name: String,
    /// Offset in the archive
    pub offset: u64,
    /// Size in bytes
    pub size: u32,
}

impl MvlArchive {
    /// Open an MVL archive file
    ///
    /// # Example
    /// ```ignore
    /// use descent_core::MvlArchive;
    ///
    /// let mvl = MvlArchive::open("intro-l.mvl")?;
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        let entries = Self::parse_entries(&mut file)?;

        Ok(Self { file, entries })
    }

    /// Parse MVL archive entries from file header
    fn parse_entries(file: &mut File) -> Result<BTreeMap<String, MvlEntry>> {
        let mut entries = BTreeMap::new();

        // Read and verify signature (4 bytes: "DMVL")
        let mut signature = [0u8; 4];
        file.read_exact(&mut signature)?;

        if &signature != b"DMVL" {
            return Err(AssetError::InvalidFormat(
                "Invalid MVL signature (expected 'DMVL')".to_string(),
            ));
        }

        // Read number of files (i32 little-endian)
        let mut num_files_bytes = [0u8; 4];
        file.read_exact(&mut num_files_bytes)?;
        let num_files = i32::from_le_bytes(num_files_bytes);

        if !(0..=1000).contains(&num_files) {
            return Err(AssetError::InvalidFormat(format!(
                "Invalid number of files in MVL: {}",
                num_files
            )));
        }

        // Calculate starting offset for file data
        // 4 (signature) + 4 (count) + num_files * (13 + 4)
        let mut offset = 4 + 4 + (num_files as u64 * 17);

        // Read file entries
        for _ in 0..num_files {
            // Read filename (13 bytes, null-terminated)
            let mut filename = [0u8; 13];
            file.read_exact(&mut filename)?;

            // Convert to string, stopping at first null byte
            let name = filename
                .iter()
                .take_while(|&&b| b != 0)
                .map(|&b| b as char)
                .collect::<String>()
                .to_uppercase();

            // Read file size (i32 little-endian)
            let mut size_bytes = [0u8; 4];
            file.read_exact(&mut size_bytes)?;
            let size = i32::from_le_bytes(size_bytes);

            if size < 0 {
                return Err(AssetError::InvalidFormat(format!(
                    "Invalid file size for '{}': {}",
                    name, size
                )));
            }

            let entry = MvlEntry {
                name: name.clone(),
                offset,
                size: size as u32,
            };

            entries.insert(name, entry);
            offset += size as u64;
        }

        Ok(entries)
    }

    /// Get all entries in the archive
    ///
    /// # Returns
    ///
    /// Iterator over all entries
    pub fn entries(&self) -> impl Iterator<Item = &MvlEntry> {
        self.entries.values()
    }

    /// Get a specific entry by filename (case-insensitive)
    ///
    /// # Arguments
    ///
    /// * `name` - Filename to look up
    ///
    /// # Returns
    ///
    /// The entry if found, None otherwise
    pub fn get_entry(&self, name: &str) -> Option<&MvlEntry> {
        self.entries.get(&name.to_uppercase())
    }

    /// Check if a file exists in the archive (case-insensitive)
    ///
    /// # Arguments
    ///
    /// * `name` - Filename to check
    pub fn contains_file(&self, name: &str) -> bool {
        self.entries.contains_key(&name.to_uppercase())
    }

    /// Read a file from the archive by name (case-insensitive)
    ///
    /// # Arguments
    ///
    /// * `name` - Filename to read
    ///
    /// # Returns
    ///
    /// File contents as a Vec<u8>
    ///
    /// # Errors
    ///
    /// Returns an error if the file is not found or cannot be read
    pub fn read_file(&mut self, name: &str) -> Result<Vec<u8>> {
        let entry = self
            .get_entry(name)
            .ok_or_else(|| AssetError::FileNotFound(name.to_string()))?
            .clone();

        self.read_entry(&entry)
    }

    /// Read a file from the archive using an entry reference
    ///
    /// # Arguments
    ///
    /// * `entry` - Entry to read
    ///
    /// # Returns
    ///
    /// File contents as a Vec<u8>
    pub fn read_entry(&mut self, entry: &MvlEntry) -> Result<Vec<u8>> {
        let mut buffer = vec![0u8; entry.size as usize];
        self.file.seek(SeekFrom::Start(entry.offset))?;
        self.file.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    /// Get the number of files in the archive
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the archive is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// List all filenames in the archive
    ///
    /// # Returns
    ///
    /// Iterator over filenames
    pub fn filenames(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mvl_entry_debug() {
        let entry = MvlEntry {
            name: "INTRO.MVE".to_string(),
            offset: 1024,
            size: 2048,
        };
        assert_eq!(entry.name, "INTRO.MVE");
        assert_eq!(entry.offset, 1024);
        assert_eq!(entry.size, 2048);
    }
}
