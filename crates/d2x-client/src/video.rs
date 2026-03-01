//! Video conversion module for MVE cutscene files.
//!
//! This module handles conversion of Descent's MVE (Interplay Movie) files to modern
//! MP4/H.264 format for playback in Bevy. The conversion uses FFmpeg's native MVE
//! demuxer, statically linked via the `ffmpeg-next` crate.
//!
//! # Feature Gate
//!
//! This module is only available when the `cutscenes` feature is enabled. This allows
//! builds without video support to avoid the large FFmpeg dependency.
//!
//! # Usage
//!
//! ```rust,ignore
//! use d2x_client::video::VideoConverter;
//!
//! let converter = VideoConverter::new();
//! converter.convert_mve("intro.mve", "output/intro.mp4")?;
//! ```
//!
//! # Implementation Status
//!
//! **Current**: Placeholder implementation that validates FFmpeg integration.
//! **TODO**: Implement full transcoding pipeline (decode MVE → encode H.264/AAC).
//!
//! The full implementation requires:
//! 1. Opening input MVE file with FFmpeg's `ipmovie` demuxer
//! 2. Decoding video frames (8-bit indexed color) and audio (PCM)
//! 3. Encoding to H.264 (CRF 18) and AAC (128kbps)
//! 4. Muxing to MP4 container
//! 5. Proper frame timing and A/V sync

#[cfg(feature = "cutscenes")]
use anyhow::{Context, Result};
#[cfg(feature = "cutscenes")]
use std::path::Path;
#[cfg(feature = "cutscenes")]
use tracing::{info, warn};

/// Video converter for MVE files.
#[cfg(feature = "cutscenes")]
pub struct VideoConverter {
    /// Quality setting (CRF value, 0-51, lower = better quality)
    pub quality: u8,
    /// Output audio bitrate in kbps
    pub audio_bitrate: u32,
}

#[cfg(feature = "cutscenes")]
impl Default for VideoConverter {
    fn default() -> Self {
        Self {
            quality: 18,        // Visually lossless
            audio_bitrate: 128, // 128 kbps AAC
        }
    }
}

#[cfg(feature = "cutscenes")]
impl VideoConverter {
    /// Create a new video converter with default settings.
    pub const fn new() -> Self {
        Self {
            quality: 18,
            audio_bitrate: 128,
        }
    }

    /// Convert an MVE file to MP4/H.264.
    ///
    /// # Arguments
    ///
    /// * `input_path` - Path to the source MVE file
    /// * `output_path` - Path where the MP4 file should be saved
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Input file doesn't exist or can't be read
    /// - FFmpeg fails to decode the MVE file
    /// - Output directory doesn't exist or isn't writable
    /// - Encoding fails
    ///
    /// # Implementation Note
    ///
    /// This is currently a placeholder that verifies FFmpeg integration.
    /// Full transcoding implementation is pending.
    pub fn convert_mve(
        &self,
        input_path: impl AsRef<Path>,
        output_path: impl AsRef<Path>,
    ) -> Result<()> {
        let input_path = input_path.as_ref();
        let output_path = output_path.as_ref();

        info!(
            "Converting MVE: {} -> {}",
            input_path.display(),
            output_path.display()
        );

        // Ensure output directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create output directory: {}", parent.display())
            })?;
        }

        // Verify FFmpeg can open the file
        let input_context = ffmpeg_next::format::input(input_path)
            .with_context(|| format!("Failed to open input file: {}", input_path.display()))?;

        let mut converted = 0;

        // Convert each MVE file
        for i in 0..mvl.file_count() {
            let (filename, mve_data) = mvl.get_file(i)?;

            // Convert filename to valid path (remove null bytes, convert to lowercase)
            let clean_filename = filename.trim_end_matches('\0').to_lowercase();
            let output_filename = std::path::PathBuf::from(&clean_filename).with_extension("mp4");
            let output_path = output_dir.join(&output_filename);

            info!(
                "Converting {}/{}: {} -> {}",
                i + 1,
                mvl.file_count(),
                clean_filename,
                output_path.display()
            );

            // Write MVE data to temporary file
            let temp_dir = std::env::temp_dir();
            let temp_mve = temp_dir.join(format!("d2x_temp_{}.mve", i));

            std::fs::write(&temp_mve, mve_data).context("Failed to write temporary MVE file")?;

            // Convert
            match self.convert_mve(&temp_mve, &output_path) {
                Ok(()) => {
                    converted += 1;
                    info!("Successfully processed: {}", clean_filename);
                }
                Err(e) => {
                    warn!("Failed to convert {}: {}", clean_filename, e);
                }
            }

            // Clean up temp file
            let _ = std::fs::remove_file(&temp_mve);
        }

        info!(
            "Processed {}/{} videos from MVL archive",
            converted,
            mvl.file_count()
        );

        Ok(converted)
    }
}

#[cfg(not(feature = "cutscenes"))]
/// Stub implementation when cutscenes feature is disabled.
pub struct VideoConverter;

#[cfg(not(feature = "cutscenes"))]
impl VideoConverter {
    pub const fn new() -> Self {
        Self
    }
}
