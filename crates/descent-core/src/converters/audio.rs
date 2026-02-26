//! Audio format converters for Descent sound effects and music.
//!
//! This module provides converters for:
//! - **PCM → WAV**: Convert raw 8-bit PCM sound effects to WAV format
//! - **HMP → MIDI**: Convert Descent HMP music to standard MIDI (via `HmpFile::to_midi()`)
//!
//! # Examples
//!
//! ## Converting Sound Effects to WAV
//!
//! ```no_run
//! use descent_core::converters::audio::AudioConverter;
//! use std::fs;
//!
//! // Raw 8-bit PCM data from PIG file
//! let pcm_data: Vec<u8> = vec![128, 130, 135, 140, /* ... */];
//! let sample_rate = 22050; // Hz
//!
//! let converter = AudioConverter::new();
//! let wav_data = converter.pcm_to_wav(&pcm_data, sample_rate).unwrap();
//! fs::write("sound.wav", wav_data).unwrap();
//! ```
//!
//! ## Converting HMP Music to MIDI
//!
//! ```no_run
//! use descent_core::sound::HmpFile;
//! use std::fs;
//!
//! let hmp_data = fs::read("game01.hmp").unwrap();
//! let hmp = HmpFile::parse(&hmp_data).unwrap();
//!
//! // Use built-in conversion
//! let midi_data = hmp.to_midi().unwrap();
//! fs::write("game01.mid", midi_data).unwrap();
//! ```

use std::io::{Cursor, Write};
use thiserror::Error;

/// Audio conversion errors.
#[derive(Debug, Error)]
pub enum AudioConvertError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid sample rate: {0} Hz (must be > 0)")]
    InvalidSampleRate(u32),

    #[error("PCM data is empty")]
    EmptyData,

    #[error("PCM data too large: {0} bytes (max 4GB)")]
    DataTooLarge(usize),
}

/// Audio format converter.
///
/// Handles conversion of Descent's audio formats to modern standards.
pub struct AudioConverter {
    /// Target bit depth for WAV output (8 or 16).
    bit_depth: u16,
}

impl AudioConverter {
    /// Create a new audio converter with default settings.
    ///
    /// Default: 16-bit WAV output (converts 8-bit PCM to 16-bit)
    pub fn new() -> Self {
        Self { bit_depth: 16 }
    }

    /// Create a converter with custom bit depth.
    ///
    /// # Arguments
    ///
    /// * `bit_depth` - Output bit depth (8 or 16)
    pub fn with_bit_depth(bit_depth: u16) -> Self {
        assert!(
            bit_depth == 8 || bit_depth == 16,
            "Bit depth must be 8 or 16"
        );
        Self { bit_depth }
    }

    /// Convert 8-bit unsigned PCM to WAV format.
    ///
    /// Descent sound effects are stored as raw 8-bit unsigned PCM data.
    /// This converts them to standard WAV format with proper headers.
    ///
    /// # Arguments
    ///
    /// * `pcm_data` - Raw 8-bit unsigned PCM samples (0-255, 128=silence)
    /// * `sample_rate` - Sample rate in Hz (typically 11025 or 22050)
    ///
    /// # Returns
    ///
    /// WAV file data ready to be written to disk.
    ///
    /// # Format Details
    ///
    /// - Input: 8-bit unsigned PCM (0-255, 128 = center/silence)
    /// - Output: 16-bit signed PCM (-32768 to 32767, 0 = silence) or 8-bit unsigned
    /// - Channels: Mono (1 channel)
    /// - Byte order: Little-endian
    ///
    /// # Example
    ///
    /// ```no_run
    /// use descent_core::converters::audio::AudioConverter;
    ///
    /// let pcm_data: Vec<u8> = vec![128, 130, 135, 140, 135, 130, 128];
    /// let converter = AudioConverter::new();
    /// let wav_data = converter.pcm_to_wav(&pcm_data, 22050).unwrap();
    /// std::fs::write("sound.wav", wav_data).unwrap();
    /// ```
    pub fn pcm_to_wav(
        &self,
        pcm_data: &[u8],
        sample_rate: u32,
    ) -> Result<Vec<u8>, AudioConvertError> {
        // Validate inputs
        if sample_rate == 0 {
            return Err(AudioConvertError::InvalidSampleRate(sample_rate));
        }

        if pcm_data.is_empty() {
            return Err(AudioConvertError::EmptyData);
        }

        if pcm_data.len() > u32::MAX as usize {
            return Err(AudioConvertError::DataTooLarge(pcm_data.len()));
        }

        // Convert PCM data based on bit depth
        let audio_data = if self.bit_depth == 16 {
            self.convert_8bit_to_16bit(pcm_data)
        } else {
            pcm_data.to_vec()
        };

        // Calculate sizes
        let num_channels: u16 = 1; // Mono
        let bits_per_sample = self.bit_depth;
        let byte_rate = sample_rate * u32::from(num_channels) * u32::from(bits_per_sample / 8);
        let block_align = num_channels * (bits_per_sample / 8);
        let data_size = audio_data.len() as u32;
        let file_size = 36 + data_size; // 44 byte header - 8 byte RIFF header

        // Build WAV file
        let mut wav = Cursor::new(Vec::new());

        // RIFF header
        wav.write_all(b"RIFF")?;
        wav.write_all(&file_size.to_le_bytes())?;
        wav.write_all(b"WAVE")?;

        // fmt chunk
        wav.write_all(b"fmt ")?;
        wav.write_all(&16u32.to_le_bytes())?; // Chunk size
        wav.write_all(&1u16.to_le_bytes())?; // Audio format (1 = PCM)
        wav.write_all(&num_channels.to_le_bytes())?;
        wav.write_all(&sample_rate.to_le_bytes())?;
        wav.write_all(&byte_rate.to_le_bytes())?;
        wav.write_all(&block_align.to_le_bytes())?;
        wav.write_all(&bits_per_sample.to_le_bytes())?;

        // data chunk
        wav.write_all(b"data")?;
        wav.write_all(&data_size.to_le_bytes())?;
        wav.write_all(&audio_data)?;

        Ok(wav.into_inner())
    }

    /// Convert 8-bit unsigned PCM to 16-bit signed PCM.
    ///
    /// Descent uses 8-bit unsigned format (0-255, 128=silence).
    /// This converts to 16-bit signed format (-32768 to 32767, 0=silence).
    fn convert_8bit_to_16bit(&self, pcm_8bit: &[u8]) -> Vec<u8> {
        let mut pcm_16bit = Vec::with_capacity(pcm_8bit.len() * 2);

        for &sample in pcm_8bit {
            // Convert: unsigned 8-bit (0-255) → signed 16-bit (-32768 to 32767)
            // Formula: (sample - 128) * 256
            let sample_16 = (i16::from(sample) - 128) * 256;
            pcm_16bit.extend_from_slice(&sample_16.to_le_bytes());
        }

        pcm_16bit
    }

    /// Get the current bit depth setting.
    pub fn bit_depth(&self) -> u16 {
        self.bit_depth
    }
}

impl Default for AudioConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pcm_to_wav_16bit() {
        let converter = AudioConverter::new();

        // Create test PCM data (silent, then louder)
        let pcm_data = vec![128, 128, 130, 135, 140, 135, 130, 128, 128];
        let sample_rate = 22050;

        let wav_data = converter.pcm_to_wav(&pcm_data, sample_rate).unwrap();

        // Verify WAV header
        assert_eq!(&wav_data[0..4], b"RIFF");
        assert_eq!(&wav_data[8..12], b"WAVE");
        assert_eq!(&wav_data[12..16], b"fmt ");

        // Verify format parameters
        let num_channels = u16::from_le_bytes([wav_data[22], wav_data[23]]);
        let sample_rate_in_file =
            u32::from_le_bytes([wav_data[24], wav_data[25], wav_data[26], wav_data[27]]);
        let bits_per_sample = u16::from_le_bytes([wav_data[34], wav_data[35]]);

        assert_eq!(num_channels, 1); // Mono
        assert_eq!(sample_rate_in_file, 22050);
        assert_eq!(bits_per_sample, 16);

        // Verify data chunk
        assert_eq!(&wav_data[36..40], b"data");
        let data_size =
            u32::from_le_bytes([wav_data[40], wav_data[41], wav_data[42], wav_data[43]]);
        assert_eq!(data_size, 18); // 9 samples * 2 bytes
    }

    #[test]
    fn test_pcm_to_wav_8bit() {
        let converter = AudioConverter::with_bit_depth(8);

        let pcm_data = vec![128, 130, 135, 140];
        let sample_rate = 11025;

        let wav_data = converter.pcm_to_wav(&pcm_data, sample_rate).unwrap();

        // Verify 8-bit format
        let bits_per_sample = u16::from_le_bytes([wav_data[34], wav_data[35]]);
        assert_eq!(bits_per_sample, 8);

        // Verify data size (should match input size for 8-bit)
        let data_size =
            u32::from_le_bytes([wav_data[40], wav_data[41], wav_data[42], wav_data[43]]);
        assert_eq!(data_size, 4);
    }

    #[test]
    fn test_8bit_to_16bit_conversion() {
        let converter = AudioConverter::new();

        // Test silence (128 → 0)
        let silence = vec![128];
        let converted = converter.convert_8bit_to_16bit(&silence);
        let sample = i16::from_le_bytes([converted[0], converted[1]]);
        assert_eq!(sample, 0);

        // Test max positive (255 → 32512)
        let max = vec![255];
        let converted = converter.convert_8bit_to_16bit(&max);
        let sample = i16::from_le_bytes([converted[0], converted[1]]);
        assert_eq!(sample, 32512);

        // Test max negative (0 → -32768)
        let min = vec![0];
        let converted = converter.convert_8bit_to_16bit(&min);
        let sample = i16::from_le_bytes([converted[0], converted[1]]);
        assert_eq!(sample, -32768);
    }

    #[test]
    fn test_invalid_sample_rate() {
        let converter = AudioConverter::new();
        let pcm_data = vec![128, 130, 135];

        let result = converter.pcm_to_wav(&pcm_data, 0);
        assert!(matches!(
            result,
            Err(AudioConvertError::InvalidSampleRate(0))
        ));
    }

    #[test]
    fn test_empty_data() {
        let converter = AudioConverter::new();
        let pcm_data = vec![];

        let result = converter.pcm_to_wav(&pcm_data, 22050);
        assert!(matches!(result, Err(AudioConvertError::EmptyData)));
    }
}
