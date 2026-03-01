//! Video conversion module for MVE cutscene files.
//!
//! This module handles conversion of Descent's MVE (Interplay Movie) files to modern
//! MP4/H.264 format. The conversion uses FFmpeg's native MVE demuxer (`ipmovie`).
//!
//! # Feature Gate
//!
//! This module is only available when the `video` feature is enabled. This allows
//! builds without video support to avoid the FFmpeg dependency.
//!
//! # Requirements
//!
//! Requires FFmpeg libraries to be installed on the system:
//! - **Linux**: `sudo apt install libavcodec-dev libavformat-dev libavutil-dev`
//! - **macOS**: `brew install ffmpeg`
//! - **Windows**: Download FFmpeg shared libraries or use vcpkg
//!
//! # Usage
//!
//! ```rust,ignore
//! use descent_core::VideoConverter;
//!
//! let converter = VideoConverter::new();
//! converter.convert_mve("intro.mve", "output/intro.mp4")?;
//!
//! // Or batch convert an entire MVL archive
//! converter.convert_mvl("intro-l.mvl", "output/videos/")?;
//! ```

use anyhow::{Context, Result};
use std::path::Path;
use tracing::{debug, info, warn};

/// Video converter for MVE files.
pub struct VideoConverter {
    /// Quality setting (CRF value, 0-51, lower = better quality)
    pub quality: u8,
    /// Output audio bitrate in kbps
    pub audio_bitrate: u32,
    /// Target video bitrate in kbps (0 = use quality mode)
    pub video_bitrate: u32,
}

impl Default for VideoConverter {
    fn default() -> Self {
        Self {
            quality: 18,         // Visually lossless
            audio_bitrate: 128,  // 128 kbps AAC
            video_bitrate: 2000, // 2 Mbps default
        }
    }
}

impl VideoConverter {
    /// Create a new video converter with default settings.
    pub const fn new() -> Self {
        Self {
            quality: 18,
            audio_bitrate: 128,
            video_bitrate: 2000,
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

        // Open input file
        let mut input_context = ffmpeg_next::format::input(input_path)
            .with_context(|| format!("Failed to open input file: {}", input_path.display()))?;

        debug!(
            "Input format: {}, duration: {:.2}s",
            input_context.format().name(),
            input_context.duration() as f64 / f64::from(ffmpeg_next::ffi::AV_TIME_BASE)
        );

        // Find best streams
        let video_stream_index = input_context
            .streams()
            .best(ffmpeg_next::media::Type::Video)
            .context("No video stream found")?
            .index();

        let audio_stream_index = input_context
            .streams()
            .best(ffmpeg_next::media::Type::Audio)
            .map(|s| s.index());

        debug!("Video stream index: {}", video_stream_index);
        if let Some(idx) = audio_stream_index {
            debug!("Audio stream index: {}", idx);
        }

        // Get stream parameters
        let input_video_stream = input_context
            .stream(video_stream_index)
            .context("Failed to get video stream")?;

        // Create video decoder
        let video_context_decoder =
            ffmpeg_next::codec::context::Context::from_parameters(input_video_stream.parameters())?;
        let mut video_decoder = video_context_decoder
            .decoder()
            .video()
            .context("Failed to create video decoder")?;

        debug!(
            "Input video: {}x{}, format: {:?}, fps: {}/{}",
            video_decoder.width(),
            video_decoder.height(),
            video_decoder.format(),
            input_video_stream.avg_frame_rate().0,
            input_video_stream.avg_frame_rate().1
        );

        // Create audio decoder if present
        let mut audio_decoder = if let Some(audio_idx) = audio_stream_index {
            let input_audio_stream = input_context
                .stream(audio_idx)
                .context("Failed to get audio stream")?;
            let audio_context_decoder = ffmpeg_next::codec::context::Context::from_parameters(
                input_audio_stream.parameters(),
            )?;
            Some(
                audio_context_decoder
                    .decoder()
                    .audio()
                    .context("Failed to create audio decoder")?,
            )
        } else {
            None
        };

        if let Some(ref decoder) = audio_decoder {
            debug!(
                "Input audio: {} Hz, {} channels, format: {:?}",
                decoder.rate(),
                decoder.channels(),
                decoder.format()
            );
        }

        // Create output context
        let mut output_context = ffmpeg_next::format::output(&output_path)
            .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;

        // Create video encoder
        let global_header = output_context
            .format()
            .flags()
            .contains(ffmpeg_next::format::Flags::GLOBAL_HEADER);

        let mut video_encoder = self.create_video_encoder(&video_decoder, global_header)?;

        // Add video stream to output
        let mut output_video_stream =
            output_context.add_stream(ffmpeg_next::encoder::find(ffmpeg_next::codec::Id::H264))?;
        output_video_stream.set_parameters(&video_encoder);

        // Create audio encoder and add stream if audio present
        let mut audio_encoder = if let Some(ref decoder) = audio_decoder {
            let encoder = self.create_audio_encoder(decoder, global_header)?;
            let mut output_audio_stream = output_context
                .add_stream(ffmpeg_next::encoder::find(ffmpeg_next::codec::Id::AAC))?;
            output_audio_stream.set_parameters(&encoder);
            Some(encoder)
        } else {
            None
        };

        // Write output header
        output_context
            .write_header()
            .context("Failed to write output header")?;

        debug!("Starting transcoding...");

        // Save time bases before iterating (to avoid borrow checker issues)
        let video_stream_time_base = input_video_stream.time_base();
        let video_decoder_time_base = video_decoder.time_base();
        let audio_stream_time_base =
            audio_stream_index.and_then(|idx| input_context.stream(idx).map(|s| s.time_base()));
        let audio_decoder_time_base = audio_decoder.as_ref().map(|d| d.time_base());

        // Transcode loop
        let mut video_frame_count = 0u64;
        let mut audio_frame_count = 0u64;

        for (stream, mut packet) in input_context.packets() {
            if stream.index() == video_stream_index {
                // Decode video packet
                packet.rescale_ts(video_stream_time_base, video_decoder_time_base);
                video_decoder.send_packet(&packet)?;

                let mut decoded_frame = ffmpeg_next::util::frame::video::Video::empty();
                while video_decoder.receive_frame(&mut decoded_frame).is_ok() {
                    video_frame_count += 1;

                    // Encode frame
                    video_encoder.send_frame(&decoded_frame)?;
                    self.receive_and_write_video_packets(
                        &mut video_encoder,
                        &mut output_context,
                        video_stream_index,
                    )?;
                }
            } else if Some(stream.index()) == audio_stream_index {
                if let Some(ref mut decoder) = audio_decoder {
                    // Decode audio packet
                    if let Some(audio_tb) = audio_decoder_time_base {
                        packet.rescale_ts(audio_stream_time_base.unwrap(), audio_tb);
                    }
                    decoder.send_packet(&packet)?;

                    let mut decoded_frame = ffmpeg_next::util::frame::audio::Audio::empty();
                    while decoder.receive_frame(&mut decoded_frame).is_ok() {
                        audio_frame_count += 1;

                        // Encode frame
                        if let Some(ref mut encoder) = audio_encoder {
                            encoder.send_frame(&decoded_frame)?;
                            self.receive_and_write_audio_packets(
                                encoder,
                                &mut output_context,
                                audio_stream_index.unwrap(),
                            )?;
                        }
                    }
                }
            }
        }

        debug!("Flushing encoders...");

        // Flush video encoder
        video_encoder.send_eof()?;
        self.receive_and_write_video_packets(
            &mut video_encoder,
            &mut output_context,
            video_stream_index,
        )?;

        // Flush audio encoder if present
        if let Some(ref mut encoder) = audio_encoder {
            encoder.send_eof()?;
            self.receive_and_write_audio_packets(
                encoder,
                &mut output_context,
                audio_stream_index.unwrap(),
            )?;
        }

        // Write output trailer
        output_context
            .write_trailer()
            .context("Failed to write output trailer")?;

        info!(
            "Conversion complete: {} video frames, {} audio frames",
            video_frame_count, audio_frame_count
        );

        Ok(())
    }

    /// Receive encoded video packets and write to output.
    fn receive_and_write_video_packets(
        &self,
        encoder: &mut ffmpeg_next::encoder::video::Video,
        output_context: &mut ffmpeg_next::format::context::Output,
        stream_index: usize,
    ) -> Result<()> {
        let mut encoded_packet = ffmpeg_next::Packet::empty();
        while encoder.receive_packet(&mut encoded_packet).is_ok() {
            encoded_packet.set_stream(stream_index);
            encoded_packet
                .write_interleaved(output_context)
                .context("Failed to write packet")?;
        }
        Ok(())
    }

    /// Receive encoded audio packets and write to output.
    fn receive_and_write_audio_packets(
        &self,
        encoder: &mut ffmpeg_next::encoder::audio::Audio,
        output_context: &mut ffmpeg_next::format::context::Output,
        stream_index: usize,
    ) -> Result<()> {
        let mut encoded_packet = ffmpeg_next::Packet::empty();
        while encoder.receive_packet(&mut encoded_packet).is_ok() {
            encoded_packet.set_stream(stream_index);
            encoded_packet
                .write_interleaved(output_context)
                .context("Failed to write packet")?;
        }
        Ok(())
    }

    /// Create H.264 video encoder.
    fn create_video_encoder(
        &self,
        decoder: &ffmpeg_next::decoder::Video,
        global_header: bool,
    ) -> Result<ffmpeg_next::encoder::video::Video> {
        let codec = ffmpeg_next::encoder::find(ffmpeg_next::codec::Id::H264)
            .context("H.264 encoder not found")?;

        let context = ffmpeg_next::codec::context::Context::new_with_codec(codec);
        let mut encoder = context
            .encoder()
            .video()
            .context("Failed to create video encoder")?;

        // Configure video encoder
        encoder.set_width(decoder.width());
        encoder.set_height(decoder.height());
        encoder.set_format(decoder.format());
        encoder.set_time_base(decoder.time_base());
        encoder.set_bit_rate(self.video_bitrate as usize * 1000);

        if global_header {
            encoder.set_flags(ffmpeg_next::codec::Flags::GLOBAL_HEADER);
        }

        debug!(
            "Created H.264 encoder: {}x{}, bitrate: {} kbps",
            encoder.width(),
            encoder.height(),
            self.video_bitrate
        );

        Ok(encoder)
    }

    /// Create AAC audio encoder.
    fn create_audio_encoder(
        &self,
        decoder: &ffmpeg_next::decoder::Audio,
        global_header: bool,
    ) -> Result<ffmpeg_next::encoder::audio::Audio> {
        let codec = ffmpeg_next::encoder::find(ffmpeg_next::codec::Id::AAC)
            .context("AAC encoder not found")?;

        let context = ffmpeg_next::codec::context::Context::new_with_codec(codec);
        let mut encoder = context
            .encoder()
            .audio()
            .context("Failed to create audio encoder")?;

        // Configure audio encoder
        encoder.set_rate(decoder.rate() as i32);

        // Get supported audio format from codec
        let format = codec
            .audio()
            .context("Not an audio codec")?
            .formats()
            .context("No supported formats")?
            .next()
            .context("No audio format available")?;
        encoder.set_format(format);

        encoder.set_channel_layout(decoder.channel_layout());
        // Note: set_channels is not available, channel count is determined by channel_layout
        encoder.set_time_base((1, decoder.rate() as i32));
        encoder.set_bit_rate((self.audio_bitrate * 1000) as usize);

        if global_header {
            encoder.set_flags(ffmpeg_next::codec::Flags::GLOBAL_HEADER);
        }

        debug!(
            "Created AAC encoder: {} Hz, {} channels, bitrate: {} kbps",
            encoder.rate(),
            encoder.channels(),
            self.audio_bitrate
        );

        Ok(encoder)
    }

    /// Convert all MVE files from an MVL archive to MP4.
    ///
    /// # Arguments
    ///
    /// * `mvl_path` - Path to the MVL archive
    /// * `output_dir` - Directory where MP4 files should be saved
    ///
    /// # Returns
    ///
    /// Returns the number of successfully converted videos.
    pub fn convert_mvl(
        &self,
        mvl_path: impl AsRef<Path>,
        output_dir: impl AsRef<Path>,
    ) -> Result<usize> {
        let mvl_path = mvl_path.as_ref();
        let output_dir = output_dir.as_ref();

        info!("Converting MVL archive: {}", mvl_path.display());

        // Open MVL archive
        let mut mvl = crate::MvlArchive::open(mvl_path)
            .with_context(|| format!("Failed to open MVL file: {}", mvl_path.display()))?;

        let file_count = mvl.len();
        info!("Found {} videos in MVL archive", file_count);

        // Create output directory
        std::fs::create_dir_all(output_dir).with_context(|| {
            format!(
                "Failed to create output directory: {}",
                output_dir.display()
            )
        })?;

        let mut converted = 0;
        let mut index = 0;

        // Collect entry names first to avoid borrow checker issues
        let entries: Vec<_> = mvl.entries().map(|e| e.name.clone()).collect();
        let file_count = entries.len();

        // Convert each MVE file
        for filename in entries {
            index += 1;

            // Convert filename to valid path (lowercase)
            let clean_filename = filename.to_lowercase();
            let output_filename = std::path::PathBuf::from(&clean_filename).with_extension("mp4");
            let output_path = output_dir.join(&output_filename);

            info!(
                "Converting {}/{}: {} -> {}",
                index,
                file_count,
                clean_filename,
                output_path.display()
            );

            // Read MVE data from archive by name
            let mve_data = match mvl.read_file(&filename) {
                Ok(data) => data,
                Err(e) => {
                    warn!("Failed to read {} from archive: {}", filename, e);
                    continue;
                }
            };

            // Write MVE data to temporary file
            let temp_dir = std::env::temp_dir();
            let temp_mve = temp_dir.join(format!("d2x_temp_{}.mve", index));

            if let Err(e) = std::fs::write(&temp_mve, &mve_data) {
                warn!("Failed to write temporary MVE file: {}", e);
                continue;
            }

            // Convert
            match self.convert_mve(&temp_mve, &output_path) {
                Ok(()) => {
                    converted += 1;
                    info!("Successfully converted: {}", clean_filename);
                }
                Err(e) => {
                    warn!("Failed to convert {}: {:#}", clean_filename, e);
                }
            }

            // Clean up temp file
            let _ = std::fs::remove_file(&temp_mve);
        }

        info!(
            "Converted {}/{} videos from MVL archive",
            converted, file_count
        );

        Ok(converted)
    }
}
