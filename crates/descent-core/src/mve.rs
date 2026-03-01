//! MVE (Interplay Movie) file format parser (Descent 1 & 2)
//!
//! MVE files are Interplay's full-motion video format used for cutscenes in Descent 1 and 2.
//! This module provides a parser for reading MVE file structure and extracting metadata.
//!
//! ## File Format
//! ```text
//! [Header - 26 bytes]
//!   - signature: "Interplay MVE File\x1A" (20 bytes)
//!   - constant1: 0x001A (u16 little-endian)
//!   - constant2: 0x0100 (u16 little-endian)
//!   - constant3: 0x1133 (u16 little-endian)
//!
//! [Chunks]
//!   - length: u16 (little-endian)
//!   - data: [length] bytes containing segments
//!
//! [Segments within chunks]
//!   - length: u16 (little-endian)
//!   - major_type: u8 (segment category 0-31)
//!   - minor_type: u8 (segment version)
//!   - data: [length] bytes
//! ```
//!
//! ## Segment Types
//! The MVE format uses various segment types (major type) for different purposes:
//! - 0x00: End of stream
//! - 0x01: End of chunk
//! - 0x02: Create timer
//! - 0x03: Initialize audio buffers
//! - 0x04: Start/stop audio
//! - 0x05: Initialize video buffers
//! - 0x07: Display video frame
//! - 0x08: Audio frame data
//! - 0x09: Audio frame silence
//! - 0x0A: Initialize video mode
//! - 0x0B: Create gradient
//! - 0x0C: Set palette
//! - 0x0D: Set palette compressed
//! - 0x0E: Unknown
//! - 0x0F: Set decoding map
//! - 0x11: Video data
//!
//! ## Example
//! ```ignore
//! use descent_core::MveFile;
//!
//! let mve_data = std::fs::read("intro.mve")?;
//! let mve = MveFile::parse(&mve_data)?;
//!
//! println!("MVE file with {} chunks", mve.chunk_count());
//! ```

use crate::error::{AssetError, Result};

/// MVE file header signature (20 bytes)
const MVE_SIGNATURE: &[u8] = b"Interplay MVE File\x1A\x00";

/// Expected header constants for validation
const MVE_HEADER_CONST1: u16 = 0x001A;
const MVE_HEADER_CONST2: u16 = 0x0100;
const MVE_HEADER_CONST3: u16 = 0x1133;

/// MVE file structure
#[derive(Debug)]
pub struct MveFile {
    /// Raw file data
    data: Vec<u8>,
    /// Offset to first chunk (after header)
    first_chunk_offset: usize,
}

/// MVE chunk containing segments
#[derive(Debug, Clone)]
pub struct MveChunk {
    /// Chunk length in bytes
    pub length: u16,
    /// Offset to chunk data in file
    pub offset: usize,
}

/// MVE segment within a chunk
#[derive(Debug, Clone)]
pub struct MveSegment {
    /// Segment data length
    pub length: u16,
    /// Major type (segment category)
    pub major_type: u8,
    /// Minor type (segment version)
    pub minor_type: u8,
    /// Offset to segment data
    pub data_offset: usize,
}

/// Segment type constants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MveSegmentType {
    /// End of stream
    EndOfStream = 0x00,
    /// End of chunk
    EndOfChunk = 0x01,
    /// Create timer
    CreateTimer = 0x02,
    /// Initialize audio buffers
    InitAudioBuffers = 0x03,
    /// Start/stop audio
    StartStopAudio = 0x04,
    /// Initialize video buffers
    InitVideoBuffers = 0x05,
    /// Display video frame
    DisplayVideoFrame = 0x07,
    /// Audio frame data
    AudioFrameData = 0x08,
    /// Audio frame silence
    AudioFrameSilence = 0x09,
    /// Initialize video mode
    InitVideoMode = 0x0A,
    /// Create gradient
    CreateGradient = 0x0B,
    /// Set palette
    SetPalette = 0x0C,
    /// Set palette compressed
    SetPaletteCompressed = 0x0D,
    /// Unknown
    Unknown0E = 0x0E,
    /// Set decoding map
    SetDecodingMap = 0x0F,
    /// Video data
    VideoData = 0x11,
}

impl TryFrom<u8> for MveSegmentType {
    type Error = AssetError;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0x00 => Ok(Self::EndOfStream),
            0x01 => Ok(Self::EndOfChunk),
            0x02 => Ok(Self::CreateTimer),
            0x03 => Ok(Self::InitAudioBuffers),
            0x04 => Ok(Self::StartStopAudio),
            0x05 => Ok(Self::InitVideoBuffers),
            0x07 => Ok(Self::DisplayVideoFrame),
            0x08 => Ok(Self::AudioFrameData),
            0x09 => Ok(Self::AudioFrameSilence),
            0x0A => Ok(Self::InitVideoMode),
            0x0B => Ok(Self::CreateGradient),
            0x0C => Ok(Self::SetPalette),
            0x0D => Ok(Self::SetPaletteCompressed),
            0x0E => Ok(Self::Unknown0E),
            0x0F => Ok(Self::SetDecodingMap),
            0x11 => Ok(Self::VideoData),
            _ => Err(AssetError::InvalidFormat(format!(
                "Unknown MVE segment type: 0x{:02X}",
                value
            ))),
        }
    }
}

impl From<MveSegmentType> for u8 {
    fn from(value: MveSegmentType) -> Self {
        value as u8
    }
}

impl MveFile {
    /// Parse an MVE file from raw data
    ///
    /// # Arguments
    ///
    /// * `data` - Raw MVE file data
    ///
    /// # Returns
    ///
    /// Parsed MVE file structure
    ///
    /// # Errors
    ///
    /// Returns an error if the file header is invalid
    ///
    /// # Example
    /// ```ignore
    /// use descent_core::MveFile;
    ///
    /// let mve_data = std::fs::read("intro.mve")?;
    /// let mve = MveFile::parse(&mve_data)?;
    /// ```
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 26 {
            return Err(AssetError::InvalidFormat(
                "MVE file too small for header".to_string(),
            ));
        }

        // Verify signature
        if &data[0..20] != MVE_SIGNATURE {
            return Err(AssetError::InvalidFormat(
                "Invalid MVE signature".to_string(),
            ));
        }

        // Verify header constants
        let const1 = u16::from_le_bytes([data[20], data[21]]);
        let const2 = u16::from_le_bytes([data[22], data[23]]);
        let const3 = u16::from_le_bytes([data[24], data[25]]);

        if const1 != MVE_HEADER_CONST1 {
            return Err(AssetError::InvalidFormat(format!(
                "Invalid MVE header constant 1: expected 0x{:04X}, got 0x{:04X}",
                MVE_HEADER_CONST1, const1
            )));
        }

        if const2 != MVE_HEADER_CONST2 {
            return Err(AssetError::InvalidFormat(format!(
                "Invalid MVE header constant 2: expected 0x{:04X}, got 0x{:04X}",
                MVE_HEADER_CONST2, const2
            )));
        }

        if const3 != MVE_HEADER_CONST3 {
            return Err(AssetError::InvalidFormat(format!(
                "Invalid MVE header constant 3: expected 0x{:04X}, got 0x{:04X}",
                MVE_HEADER_CONST3, const3
            )));
        }

        Ok(Self {
            data: data.to_vec(),
            first_chunk_offset: 26,
        })
    }

    /// Get the raw file data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the file size in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Count the number of chunks in the file
    ///
    /// This iterates through all chunks to count them.
    pub fn chunk_count(&self) -> usize {
        self.chunks().count()
    }

    /// Iterate over all chunks in the file
    ///
    /// # Returns
    ///
    /// Iterator over chunks
    pub fn chunks(&self) -> MveChunkIterator<'_> {
        MveChunkIterator {
            data: &self.data,
            offset: self.first_chunk_offset,
        }
    }

    /// Get a specific chunk by index
    ///
    /// # Arguments
    ///
    /// * `index` - Chunk index (0-based)
    ///
    /// # Returns
    ///
    /// The chunk if found, None if index is out of bounds
    pub fn get_chunk(&self, index: usize) -> Option<MveChunk> {
        self.chunks().nth(index)
    }

    /// Read segment data from a segment
    ///
    /// # Arguments
    ///
    /// * `segment` - Segment to read
    ///
    /// # Returns
    ///
    /// Segment data as a byte slice
    pub fn get_segment_data(&self, segment: &MveSegment) -> &[u8] {
        let start = segment.data_offset;
        let end = start + segment.length as usize;
        &self.data[start..end]
    }

    /// Iterate over segments in a specific chunk
    ///
    /// # Arguments
    ///
    /// * `chunk` - Chunk to iterate
    ///
    /// # Returns
    ///
    /// Iterator over segments in the chunk
    pub fn chunk_segments(&self, chunk: &MveChunk) -> MveSegmentIterator<'_> {
        MveSegmentIterator {
            data: &self.data,
            offset: chunk.offset,
            end_offset: chunk.offset + chunk.length as usize,
        }
    }
}

/// Iterator over MVE chunks
pub struct MveChunkIterator<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Iterator for MveChunkIterator<'a> {
    type Item = MveChunk;

    fn next(&mut self) -> Option<Self::Item> {
        // Need at least 4 bytes for chunk header (length + type)
        if self.offset + 4 > self.data.len() {
            return None;
        }

        // Read chunk length (u16 little-endian)
        let length = u16::from_le_bytes([self.data[self.offset], self.data[self.offset + 1]]);

        // Skip length field (2 bytes) to get to chunk data
        let chunk_data_offset = self.offset + 2;

        // Verify we have enough data for the chunk
        if chunk_data_offset + length as usize > self.data.len() {
            return None;
        }

        let chunk = MveChunk {
            length,
            offset: chunk_data_offset,
        };

        // Move to next chunk (skip length field + chunk data + 2 bytes for chunk type)
        self.offset = chunk_data_offset + length as usize;

        // Skip the chunk type field (2 bytes) if present
        if self.offset + 2 <= self.data.len() {
            self.offset += 2;
        }

        Some(chunk)
    }
}

/// Iterator over MVE segments within a chunk
pub struct MveSegmentIterator<'a> {
    data: &'a [u8],
    offset: usize,
    end_offset: usize,
}

impl<'a> Iterator for MveSegmentIterator<'a> {
    type Item = MveSegment;

    fn next(&mut self) -> Option<Self::Item> {
        // Need at least 4 bytes for segment header
        if self.offset + 4 > self.end_offset || self.offset + 4 > self.data.len() {
            return None;
        }

        // Read segment length (u16 little-endian)
        let length = u16::from_le_bytes([self.data[self.offset], self.data[self.offset + 1]]);

        // Read major and minor types
        let major_type = self.data[self.offset + 2];
        let minor_type = self.data[self.offset + 3];

        // Data starts after the 4-byte header
        let data_offset = self.offset + 4;

        // Check for end of chunk marker
        if major_type == 0x01 {
            return None;
        }

        // Verify we have enough data for the segment
        if data_offset + length as usize > self.end_offset
            || data_offset + length as usize > self.data.len()
        {
            return None;
        }

        let segment = MveSegment {
            length,
            major_type,
            minor_type,
            data_offset,
        };

        // Move to next segment
        self.offset = data_offset + length as usize;

        Some(segment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mve_signature() {
        assert_eq!(MVE_SIGNATURE.len(), 20);
        assert_eq!(&MVE_SIGNATURE[0..18], b"Interplay MVE File");
        assert_eq!(MVE_SIGNATURE[18], 0x1A);
    }

    #[test]
    fn test_segment_type_conversion() {
        use std::convert::TryFrom;

        assert_eq!(
            MveSegmentType::try_from(0x00).ok(),
            Some(MveSegmentType::EndOfStream)
        );
        assert_eq!(
            MveSegmentType::try_from(0x11).ok(),
            Some(MveSegmentType::VideoData)
        );
        assert!(MveSegmentType::try_from(0xFF).is_err());

        // Test From trait for u8
        let segment_type = MveSegmentType::VideoData;
        let value: u8 = segment_type.into();
        assert_eq!(value, 0x11);
    }

    #[test]
    fn test_invalid_mve_too_small() {
        let data = vec![0u8; 10];
        let result = MveFile::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_mve_bad_signature() {
        let mut data = vec![0u8; 26];
        data[0..20].copy_from_slice(b"Invalid Signature!!!");
        let result = MveFile::parse(&data);
        assert!(result.is_err());
    }
}
