//! Sound and music file format parsers for Descent 1 and 2.
//!
//! This module handles two sound-related formats:
//! - **SNDs**: Digitized sound effects stored in PIG files (raw PCM audio)
//! - **HMP**: Music files using HMI MIDI format (Human Machine Interfaces MIDI variant)
//!
//! # Sound Effects (SNDs)
//!
//! Sound effects are stored in PIG files with a simple header structure:
//! - 8-byte name (null-padded)
//! - 4-byte total length
//! - 4-byte data length
//! - 4-byte offset (within PIG file)
//!
//! The audio data is raw PCM, typically:
//! - 8-bit unsigned samples
//! - 11025 Hz or 22050 Hz sample rate
//! - Mono
//!
//! # Music (HMP)
//!
//! HMP files are a variant of MIDI used by Descent, created by Human Machine Interfaces.
//! Key differences from standard MIDI:
//! - Custom file signature: "HMIMIDIP"
//! - Different variable-length encoding (HMI vs standard MIDI)
//! - Up to 32 tracks
//! - Embedded tempo and timing information
//!
//! # Example
//!
//! ```no_run
//! use descent_core::sound::{SoundHeader, HmpFile};
//! use std::fs;
//!
//! // Parse sound header from PIG file
//! let pig_data = fs::read("descent2.pig").unwrap();
//! let offset = 0x1000; // Example offset
//! let header = SoundHeader::parse(&pig_data[offset..]).unwrap();
//! println!("Sound: {}, {} bytes", header.name, header.data_length);
//!
//! // Parse HMP music file
//! let hmp_data = fs::read("game01.hmp").unwrap();
//! let hmp = HmpFile::parse(&hmp_data).unwrap();
//! println!("HMP: {} tracks, division={}", hmp.num_tracks, hmp.midi_division);
//! ```

use crate::error::{AssetError, Result};
use crate::io::ReadExt;
use crate::validation::{validate_min, validate_range};
use std::io::Cursor;

/// Maximum number of tracks in an HMP file.
pub const HMP_MAX_TRACKS: usize = 32;

/// Sound effect header from PIG file.
///
/// This structure appears in PIG files before the actual audio data.
/// The name is used to reference the sound effect from HAM game data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundHeader {
    /// Sound name (up to 8 characters, null-padded).
    pub name: String,
    /// Total length including header (bytes).
    pub length: u32,
    /// Length of audio data only (bytes).
    pub data_length: u32,
    /// Offset within PIG file (bytes).
    pub offset: u32,
}

impl SoundHeader {
    /// Parse a sound header from binary data.
    ///
    /// # Format
    ///
    /// ```text
    /// Offset  Size  Type    Description
    /// ------  ----  ------  -----------
    /// 0x00    8     char[]  Sound name (null-padded)
    /// 0x08    4     u32     Total length
    /// 0x0C    4     u32     Data length
    /// 0x10    4     u32     Offset in PIG file
    /// ```
    ///
    /// Total size: 20 bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 20 {
            return Err(AssetError::InvalidFormat(
                "Sound header too short (need 20 bytes)".to_string(),
            ));
        }

        // Read 8-byte name
        let name_bytes = &data[0..8];
        let name = String::from_utf8_lossy(name_bytes)
            .trim_end_matches('\0')
            .to_string();

        // Read lengths and offset (little-endian)
        let length = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let data_length = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        let offset = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);

        Ok(Self {
            name,
            length,
            data_length,
            offset,
        })
    }

    /// Size of the sound header in bytes.
    pub const fn header_size() -> usize {
        20
    }
}

/// Raw PCM audio data for a sound effect.
///
/// Descent sound effects are typically:
/// - 8-bit unsigned PCM
/// - 11025 Hz or 22050 Hz sample rate
/// - Mono
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundData {
    /// Sound header information.
    pub header: SoundHeader,
    /// Raw PCM audio samples (8-bit unsigned).
    pub samples: Vec<u8>,
}

impl SoundData {
    /// Parse sound data from PIG file.
    ///
    /// The data should start at the sound's offset within the PIG file.
    pub fn parse(data: &[u8], header: SoundHeader) -> Result<Self> {
        let data_len = header.data_length as usize;
        if data.len() < data_len {
            return Err(AssetError::InvalidFormat(format!(
                "Sound data too short: expected {} bytes, got {}",
                data_len,
                data.len()
            )));
        }

        let samples = data[..data_len].to_vec();

        Ok(Self { header, samples })
    }

    /// Get the sample rate (typically 11025 or 22050 Hz).
    ///
    /// This is not stored in the file and must be determined from context
    /// (HAM file) or assumed based on game version.
    pub fn sample_rate_hint(&self) -> u32 {
        // Default to 11025 Hz for D1/D2
        // The actual sample rate is specified in the HAM file
        11025
    }

    /// Convert 8-bit unsigned samples to f32 for audio playback.
    ///
    /// Range: [-1.0, 1.0]
    pub fn to_f32_samples(&self) -> Vec<f32> {
        self.samples
            .iter()
            .map(|&sample| {
                // Convert u8 (0-255) to f32 (-1.0 to 1.0)
                ((sample as f32) - 128.0) / 128.0
            })
            .collect()
    }
}

/// HMP music track data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HmpTrack {
    /// Track data (MIDI events with HMI variable-length encoding).
    pub data: Vec<u8>,
}

/// HMP (HMI MIDI) music file.
///
/// HMP is a variant of MIDI used by Descent, created by Human Machine Interfaces.
/// It differs from standard MIDI in:
/// - File signature: "HMIMIDIP"
/// - Variable-length encoding (HMI format, not standard MIDI)
/// - Track structure and header layout
///
/// # File Structure
///
/// ```text
/// Offset   Size  Description
/// -------  ----  -----------
/// 0x00     8     Signature: "HMIMIDIP"
/// 0x30     4     Number of tracks
/// 0x38     4     MIDI division (timing)
/// 0x308    ...   Track data (multiple tracks)
/// ```
///
/// Each track consists of:
/// - 4-byte offset skip
/// - 4-byte length
/// - 4-byte offset skip
/// - n bytes of MIDI data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HmpFile {
    /// Number of tracks (1-32).
    pub num_tracks: u32,
    /// MIDI division for timing (ticks per quarter note).
    pub midi_division: u32,
    /// Tempo in BPM (beats per minute), default 120.
    pub tempo: u32,
    /// Track data.
    pub tracks: Vec<HmpTrack>,
}

impl HmpFile {
    /// Parse an HMP file from binary data.
    ///
    /// # Format
    ///
    /// The HMP format has a complex header structure:
    /// - Offset 0x00: "HMIMIDIP" signature (8 bytes)
    /// - Offset 0x30: number of tracks (4 bytes, little-endian)
    /// - Offset 0x38: MIDI division (4 bytes, little-endian)
    /// - Offset 0x308: track data begins
    ///
    /// Each track:
    /// - 4 bytes: skip (seek ahead)
    /// - 4 bytes: track length (includes 12 header bytes)
    /// - 4 bytes: skip (seek ahead)
    /// - n bytes: MIDI data (length = track_length - 12)
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);

        // Read signature (8 bytes)
        let signature = cursor.read_bytes(8)?;
        if signature != b"HMIMIDIP" {
            return Err(AssetError::InvalidFormat(format!(
                "Invalid HMP signature: expected 'HMIMIDIP', got '{}'",
                String::from_utf8_lossy(&signature)
            )));
        }

        // Seek to track count at offset 0x30
        cursor.skip_bytes(0x30 - 8)?;
        let num_tracks = cursor.read_u32_le()?;
        validate_range(num_tracks, 1, HMP_MAX_TRACKS as u32, "HMP track count")?;

        // Seek to MIDI division at offset 0x38
        cursor.skip_bytes(0x38 - 0x34)?;
        let midi_division = cursor.read_u32_le()?;

        // Seek to track data at offset 0x308
        cursor.skip_bytes(0x308 - 0x3C)?;

        let mut tracks = Vec::with_capacity(num_tracks as usize);

        for _ in 0..num_tracks {
            // Skip 4 bytes
            cursor.skip_bytes(4)?;

            // Read track length (includes 12-byte overhead)
            let track_length = cursor.read_u32_le()?;
            validate_min(track_length, 12, "HMP track length")?;

            // Actual data length (minus header overhead)
            let data_length = (track_length - 12) as usize;

            // Skip 4 bytes
            cursor.skip_bytes(4)?;

            // Read track data
            let track_data = cursor.read_bytes(data_length)?;

            tracks.push(HmpTrack { data: track_data });
        }

        Ok(Self {
            num_tracks,
            midi_division,
            tempo: 120, // Default tempo
            tracks,
        })
    }

    /// Convert HMP to standard MIDI format (for playback).
    ///
    /// This requires converting HMI variable-length encoding to standard MIDI
    /// variable-length encoding and adjusting track headers.
    ///
    /// Note: Full conversion is complex and typically done at playback time.
    /// This method is a placeholder for future implementation.
    pub fn to_midi(&self) -> Result<Vec<u8>> {
        // TODO: Implement HMP to MIDI conversion
        // This requires:
        // 1. Convert HMI variable-length encoding to standard MIDI
        // 2. Create MIDI file header (MThd chunk)
        // 3. Create MIDI track headers (MTrk chunks)
        // 4. Add tempo events
        Err(AssetError::Other(
            "HMP to MIDI conversion not yet implemented".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_header_parse() {
        let mut data = vec![0u8; 20];
        // Name: "LASER1" (null-padded to 8 bytes)
        data[0..6].copy_from_slice(b"LASER1");
        // length = 1000
        data[8..12].copy_from_slice(&1000u32.to_le_bytes());
        // data_length = 980
        data[12..16].copy_from_slice(&980u32.to_le_bytes());
        // offset = 5000
        data[16..20].copy_from_slice(&5000u32.to_le_bytes());

        let header = SoundHeader::parse(&data).unwrap();
        assert_eq!(header.name, "LASER1");
        assert_eq!(header.length, 1000);
        assert_eq!(header.data_length, 980);
        assert_eq!(header.offset, 5000);
    }

    #[test]
    fn test_sound_header_too_short() {
        let data = vec![0u8; 19]; // Too short
        assert!(SoundHeader::parse(&data).is_err());
    }

    #[test]
    fn test_sound_header_name_trimming() {
        let mut data = vec![0u8; 20];
        // Name with null bytes
        data[0..8].copy_from_slice(b"SND\0\0\0\0\0");

        let header = SoundHeader::parse(&data).unwrap();
        assert_eq!(header.name, "SND");
    }

    #[test]
    fn test_sound_data_parse() {
        let header = SoundHeader {
            name: "TEST".to_string(),
            length: 1020,
            data_length: 1000,
            offset: 0,
        };

        let samples = vec![128u8; 1000]; // Silence (128 = 0 in 8-bit unsigned)
        let sound = SoundData::parse(&samples, header.clone()).unwrap();

        assert_eq!(sound.header.name, "TEST");
        assert_eq!(sound.samples.len(), 1000);
        assert_eq!(sound.samples[0], 128);
    }

    #[test]
    fn test_sound_data_too_short() {
        let header = SoundHeader {
            name: "TEST".to_string(),
            length: 1020,
            data_length: 1000,
            offset: 0,
        };

        let samples = vec![128u8; 500]; // Too short
        assert!(SoundData::parse(&samples, header).is_err());
    }

    #[test]
    fn test_sound_to_f32() {
        let header = SoundHeader {
            name: "TEST".to_string(),
            length: 23,
            data_length: 3,
            offset: 0,
        };

        let samples = vec![0u8, 128u8, 255u8]; // Min, center, max
        let sound = SoundData::parse(&samples, header).unwrap();
        let f32_samples = sound.to_f32_samples();

        assert_eq!(f32_samples.len(), 3);
        assert!((f32_samples[0] - -1.0).abs() < 0.01); // 0 -> -1.0
        assert!((f32_samples[1] - 0.0).abs() < 0.01); // 128 -> 0.0
        assert!((f32_samples[2] - 0.9921875).abs() < 0.01); // 255 -> ~1.0
    }

    #[test]
    fn test_hmp_parse_invalid_signature() {
        let mut data = vec![0u8; 1024];
        data[0..8].copy_from_slice(b"BADMAGIC");

        assert!(HmpFile::parse(&data).is_err());
    }

    #[test]
    fn test_hmp_parse_valid() {
        let mut data = vec![0u8; 0x400]; // Large enough for header + tracks

        // Signature
        data[0..8].copy_from_slice(b"HMIMIDIP");

        // Number of tracks at 0x30
        data[0x30..0x34].copy_from_slice(&2u32.to_le_bytes());

        // MIDI division at 0x38
        data[0x38..0x3C].copy_from_slice(&480u32.to_le_bytes());

        // Track 1 at 0x308
        data[0x308..0x30C].copy_from_slice(&0u32.to_le_bytes()); // skip
        data[0x30C..0x310].copy_from_slice(&50u32.to_le_bytes()); // length (50 bytes total, 38 data)
        data[0x310..0x314].copy_from_slice(&0u32.to_le_bytes()); // skip
        // 38 bytes of track data (50 - 12 = 38)

        // Track 2 starts after track 1 data (0x314 + 38 = 0x33A)
        let track2_start = 0x314 + 38;
        data[track2_start..track2_start + 4].copy_from_slice(&0u32.to_le_bytes()); // skip
        data[track2_start + 4..track2_start + 8].copy_from_slice(&30u32.to_le_bytes()); // length
        data[track2_start + 8..track2_start + 12].copy_from_slice(&0u32.to_le_bytes()); // skip

        let hmp = HmpFile::parse(&data).unwrap();
        assert_eq!(hmp.num_tracks, 2);
        assert_eq!(hmp.midi_division, 480);
        assert_eq!(hmp.tempo, 120);
        assert_eq!(hmp.tracks.len(), 2);
        assert_eq!(hmp.tracks[0].data.len(), 38);
        assert_eq!(hmp.tracks[1].data.len(), 18);
    }

    #[test]
    fn test_hmp_invalid_track_count() {
        let mut data = vec![0u8; 0x100];
        data[0..8].copy_from_slice(b"HMIMIDIP");
        // Invalid track count (0)
        data[0x30..0x34].copy_from_slice(&0u32.to_le_bytes());

        assert!(HmpFile::parse(&data).is_err());

        // Invalid track count (too many)
        data[0x30..0x34].copy_from_slice(&100u32.to_le_bytes());
        assert!(HmpFile::parse(&data).is_err());
    }

    #[test]
    fn test_hmp_track_length_validation() {
        let mut data = vec![0u8; 0x400];
        data[0..8].copy_from_slice(b"HMIMIDIP");
        data[0x30..0x34].copy_from_slice(&1u32.to_le_bytes()); // 1 track
        data[0x38..0x3C].copy_from_slice(&480u32.to_le_bytes());

        // Invalid track length (< 12)
        data[0x30C..0x310].copy_from_slice(&5u32.to_le_bytes());

        assert!(HmpFile::parse(&data).is_err());
    }
}
