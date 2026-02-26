//! Archive extraction utilities for HOG and DHF archives.
//!
//! This module provides utilities for extracting files from Descent's archive formats:
//! - **DHF**: Descent 1/2 archives (13-character filenames)
//! - **HOG2**: Descent 3 archives (36-character filenames, flags, timestamps)
//!
//! # Examples
//!
//! ## Extracting All Files from DHF Archive
//!
//! ```no_run
//! use descent_core::converters::archive::ArchiveExtractor;
//! use std::path::Path;
//!
//! let extractor = ArchiveExtractor::new();
//! extractor.extract_dhf(Path::new("descent2.hog"), Path::new("./extracted")).unwrap();
//! ```
//!
//! ## Extracting Specific File from DHF
//!
//! ```no_run
//! use descent_core::converters::archive::ArchiveExtractor;
//! use std::path::Path;
//! use std::fs;
//!
//! let extractor = ArchiveExtractor::new();
//! let file_data = extractor.extract_dhf_file(Path::new("descent2.hog"), "descent2.ham").unwrap();
//! fs::write("descent2.ham", file_data).unwrap();
//! ```

use crate::dhf::DhfArchive;
use crate::hog2::Hog2Archive;
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Archive extraction errors.
#[derive(Debug, Error)]
pub enum ArchiveError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Archive parse error: {0}")]
    Parse(String),

    #[error("File not found in archive: {0}")]
    FileNotFound(String),

    #[error("Invalid output path: {0}")]
    InvalidPath(String),
}

/// Archive extractor for HOG and DHF formats.
///
/// Provides utilities to extract files from Descent archives to disk.
pub struct ArchiveExtractor {
    /// Whether to preserve directory structure from archive.
    preserve_structure: bool,
    /// Whether to overwrite existing files.
    overwrite: bool,
}

impl ArchiveExtractor {
    /// Create a new archive extractor with default settings.
    ///
    /// Defaults:
    /// - Preserve directory structure: true
    /// - Overwrite existing files: false
    pub fn new() -> Self {
        Self {
            preserve_structure: true,
            overwrite: false,
        }
    }

    /// Set whether to preserve directory structure.
    pub fn with_preserve_structure(mut self, preserve: bool) -> Self {
        self.preserve_structure = preserve;
        self
    }

    /// Set whether to overwrite existing files.
    pub fn with_overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite = overwrite;
        self
    }

    /// Extract all files from a DHF archive to a directory.
    ///
    /// # Arguments
    ///
    /// * `archive_path` - Path to DHF/HOG archive file
    /// * `output_dir` - Directory to extract files to (will be created if missing)
    ///
    /// # Returns
    ///
    /// Number of files extracted.
    pub fn extract_dhf(
        &self,
        archive_path: &Path,
        output_dir: &Path,
    ) -> Result<usize, ArchiveError> {
        // Open archive
        let mut archive =
            DhfArchive::open(archive_path).map_err(|e| ArchiveError::Parse(e.to_string()))?;

        // Create output directory
        fs::create_dir_all(output_dir)?;

        let mut count = 0;

        // Get list of files
        let filenames: Vec<String> = archive.entries().map(|e| e.name.clone()).collect();

        // Extract each file
        for filename in filenames {
            let file_data = archive
                .read_file(&filename)
                .map_err(|e| ArchiveError::Parse(e.to_string()))?;

            let output_path = if self.preserve_structure {
                output_dir.join(&filename)
            } else {
                // Flatten path - use only filename
                let name = Path::new(&filename)
                    .file_name()
                    .ok_or_else(|| ArchiveError::InvalidPath(filename.clone()))?;
                output_dir.join(name)
            };

            // Create parent directories if needed
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Check if file exists
            if output_path.exists() && !self.overwrite {
                eprintln!("Skipping existing file: {}", output_path.display());
                continue;
            }

            // Write file
            fs::write(&output_path, file_data)?;
            count += 1;
        }

        Ok(count)
    }

    /// Extract a specific file from a DHF archive.
    ///
    /// # Arguments
    ///
    /// * `archive_path` - Path to DHF/HOG archive file
    /// * `filename` - Name of file to extract (case-insensitive)
    ///
    /// # Returns
    ///
    /// File data if found.
    pub fn extract_dhf_file(
        &self,
        archive_path: &Path,
        filename: &str,
    ) -> Result<Vec<u8>, ArchiveError> {
        let mut archive =
            DhfArchive::open(archive_path).map_err(|e| ArchiveError::Parse(e.to_string()))?;

        archive
            .read_file(filename)
            .map_err(|_e| ArchiveError::FileNotFound(filename.to_string()))
    }

    /// Extract all files from a HOG2 archive (Descent 3) to a directory.
    ///
    /// # Arguments
    ///
    /// * `archive_path` - Path to HOG2 archive file
    /// * `output_dir` - Directory to extract files to (will be created if missing)
    ///
    /// # Returns
    ///
    /// Number of files extracted.
    pub fn extract_hog2(
        &self,
        archive_path: &Path,
        output_dir: &Path,
    ) -> Result<usize, ArchiveError> {
        // Open archive
        let mut archive =
            Hog2Archive::open(archive_path).map_err(|e| ArchiveError::Parse(e.to_string()))?;

        // Create output directory
        fs::create_dir_all(output_dir)?;

        let mut count = 0;

        // Get list of files
        let filenames: Vec<String> = archive.entries().map(|e| e.name.clone()).collect();

        // Extract each file
        for filename in filenames {
            let file_data = archive
                .read_file(&filename)
                .map_err(|e| ArchiveError::Parse(e.to_string()))?;

            let output_path = if self.preserve_structure {
                output_dir.join(&filename)
            } else {
                let name = Path::new(&filename)
                    .file_name()
                    .ok_or_else(|| ArchiveError::InvalidPath(filename.clone()))?;
                output_dir.join(name)
            };

            // Create parent directories
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Check if file exists
            if output_path.exists() && !self.overwrite {
                eprintln!("Skipping existing file: {}", output_path.display());
                continue;
            }

            // Write file
            fs::write(&output_path, file_data)?;
            count += 1;
        }

        Ok(count)
    }

    /// Extract a specific file from a HOG2 archive.
    pub fn extract_hog2_file(
        &self,
        archive_path: &Path,
        filename: &str,
    ) -> Result<Vec<u8>, ArchiveError> {
        let mut archive =
            Hog2Archive::open(archive_path).map_err(|e| ArchiveError::Parse(e.to_string()))?;

        archive
            .read_file(filename)
            .map_err(|_e| ArchiveError::FileNotFound(filename.to_string()))
    }

    /// List all files in a DHF archive.
    ///
    /// Returns a vector of filenames.
    pub fn list_dhf(&self, archive_path: &Path) -> Result<Vec<String>, ArchiveError> {
        let archive =
            DhfArchive::open(archive_path).map_err(|e| ArchiveError::Parse(e.to_string()))?;

        Ok(archive.entries().map(|e| e.name.clone()).collect())
    }

    /// List all files in a HOG2 archive.
    ///
    /// Returns a vector of filenames.
    pub fn list_hog2(&self, archive_path: &Path) -> Result<Vec<String>, ArchiveError> {
        let archive =
            Hog2Archive::open(archive_path).map_err(|e| ArchiveError::Parse(e.to_string()))?;

        Ok(archive.entries().map(|e| e.name.clone()).collect())
    }
}

impl Default for ArchiveExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extractor_creation() {
        let extractor = ArchiveExtractor::new();
        assert!(extractor.preserve_structure);
        assert!(!extractor.overwrite);

        let extractor = ArchiveExtractor::new()
            .with_preserve_structure(false)
            .with_overwrite(true);
        assert!(!extractor.preserve_structure);
        assert!(extractor.overwrite);
    }
}
