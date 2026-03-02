//! Error types for asset parsing

use std::io;

/// Result type alias for asset operations
pub type Result<T> = std::result::Result<T, AssetError>;

/// Errors that can occur during asset parsing
#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    /// Invalid HOG file signature
    #[error("Invalid HOG signature")]
    InvalidHogSignature,

    /// Invalid file format
    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    /// File not found in HOG archive
    #[error("File not found in archive: {0}")]
    FileNotFound(String),

    /// Asset not found
    #[error("Asset not found: {0}")]
    NotFound(String),

    /// Invalid PIG file format
    #[error("Invalid PIG file format: {0}")]
    InvalidPigFormat(String),

    /// Invalid HAM file format
    #[error("Invalid HAM file format: {0}")]
    InvalidHamFormat(String),

    /// Invalid level file format
    #[error("Invalid level file format: {0}")]
    InvalidLevelFormat(String),

    /// Unsupported file version
    #[error("Unsupported version: {version}, expected {expected}")]
    UnsupportedVersion { version: u32, expected: u32 },

    /// Unsupported file format
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    /// Corrupt data at specific offset
    #[error("Corrupt data at offset {offset:#x}")]
    CorruptData { offset: usize },

    /// Decompression failed
    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// UTF-8 error
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    /// Binary parsing error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Other error
    #[error("{0}")]
    Other(String),
}
