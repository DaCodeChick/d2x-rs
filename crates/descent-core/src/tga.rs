//! TGA (Targa) image format parser.
//!
//! TGA is a raster graphics file format created by Truevision Inc. (now part of Avid Technology).
//! It's commonly used in D2X-XL for high-resolution textures because it supports:
//! - 8, 16, 24, and 32-bit color depths
//! - RLE compression
//! - Alpha channels
//! - Simple format that's easy to parse
//!
//! # Format Overview
//!
//! TGA files consist of:
//! 1. Header (18 bytes) - image metadata
//! 2. Image ID (variable length, usually 0)
//! 3. Color map data (if present)
//! 4. Image data (raw or RLE compressed)
//! 5. Optional footer (26 bytes for TGA 2.0)
//!
//! # Example
//!
//! ```no_run
//! use descent_core::tga::TgaImage;
//!
//! let data = std::fs::read("texture.tga").unwrap();
//! let tga = TgaImage::parse(&data).unwrap();
//!
//! println!("{}x{} image, {} bits per pixel",
//!     tga.width(), tga.height(), tga.bits_per_pixel());
//!
//! // Get RGBA data
//! let rgba = tga.to_rgba().unwrap();
//! ```

use crate::error::{AssetError, Result};
use std::io::{Cursor, Read};

/// TGA image data.
#[derive(Debug, Clone)]
pub struct TgaImage {
    header: TgaHeader,
    pixels: Vec<u8>,
}

#[derive(Debug, Clone)]
struct TgaHeader {
    id_length: u8,
    color_map_type: u8,
    image_type: u8,
    color_map_origin: u16,
    color_map_length: u16,
    color_map_depth: u8,
    x_origin: u16,
    y_origin: u16,
    width: u16,
    height: u16,
    bits_per_pixel: u8,
    image_descriptor: u8,
}

/// TGA image types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum ImageType {
    NoImage = 0,
    UncompressedColorMapped = 1,
    UncompressedRgb = 2,
    UncompressedBlackAndWhite = 3,
    RleColorMapped = 9,
    RleRgb = 10,
    RleBlackAndWhite = 11,
}

impl TgaImage {
    /// Parse a TGA image from binary data.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw TGA file data
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File format is invalid
    /// - Header is malformed
    /// - Image data is corrupted
    /// - Unsupported image type
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let header = Self::parse_header(&mut cursor)?;

        // Skip image ID
        if header.id_length > 0 {
            let mut id = vec![0u8; header.id_length as usize];
            cursor
                .read_exact(&mut id)
                .map_err(|e| AssetError::ParseError(format!("Failed to read image ID: {}", e)))?;
        }

        // Skip color map (not commonly used for D2X-XL textures)
        if header.color_map_type == 1 {
            let color_map_size =
                (header.color_map_length as usize) * ((header.color_map_depth as usize + 7) / 8);
            cursor.set_position(cursor.position() + color_map_size as u64);
        }

        // Parse image data
        let pixels = Self::parse_image_data(&mut cursor, &header)?;

        Ok(Self { header, pixels })
    }

    fn parse_header(cursor: &mut Cursor<&[u8]>) -> Result<TgaHeader> {
        let mut header_bytes = [0u8; 18];
        cursor
            .read_exact(&mut header_bytes)
            .map_err(|e| AssetError::ParseError(format!("Failed to read TGA header: {}", e)))?;

        Ok(TgaHeader {
            id_length: header_bytes[0],
            color_map_type: header_bytes[1],
            image_type: header_bytes[2],
            color_map_origin: u16::from_le_bytes([header_bytes[3], header_bytes[4]]),
            color_map_length: u16::from_le_bytes([header_bytes[5], header_bytes[6]]),
            color_map_depth: header_bytes[7],
            x_origin: u16::from_le_bytes([header_bytes[8], header_bytes[9]]),
            y_origin: u16::from_le_bytes([header_bytes[10], header_bytes[11]]),
            width: u16::from_le_bytes([header_bytes[12], header_bytes[13]]),
            height: u16::from_le_bytes([header_bytes[14], header_bytes[15]]),
            bits_per_pixel: header_bytes[16],
            image_descriptor: header_bytes[17],
        })
    }

    fn parse_image_data(cursor: &mut Cursor<&[u8]>, header: &TgaHeader) -> Result<Vec<u8>> {
        let pixel_count = (header.width as usize) * (header.height as usize);
        let bytes_per_pixel = (header.bits_per_pixel as usize + 7) / 8;

        match header.image_type {
            2 => {
                // Uncompressed RGB
                let expected_size = pixel_count * bytes_per_pixel;
                let mut pixels = vec![0u8; expected_size];
                cursor.read_exact(&mut pixels).map_err(|e| {
                    AssetError::ParseError(format!("Failed to read pixel data: {}", e))
                })?;
                Ok(pixels)
            }
            10 => {
                // RLE compressed RGB
                Self::decompress_rle(cursor, pixel_count, bytes_per_pixel)
            }
            _ => Err(AssetError::InvalidFormat(format!(
                "Unsupported TGA image type: {}",
                header.image_type
            ))),
        }
    }

    fn decompress_rle(
        cursor: &mut Cursor<&[u8]>,
        pixel_count: usize,
        bytes_per_pixel: usize,
    ) -> Result<Vec<u8>> {
        let mut pixels = Vec::with_capacity(pixel_count * bytes_per_pixel);
        let mut pixel_buffer = vec![0u8; bytes_per_pixel];

        while pixels.len() < pixel_count * bytes_per_pixel {
            // Read packet header
            let mut packet_header = [0u8; 1];
            cursor.read_exact(&mut packet_header).map_err(|e| {
                AssetError::ParseError(format!("Failed to read RLE packet header: {}", e))
            })?;

            let packet_type = packet_header[0] & 0x80;
            let packet_length = ((packet_header[0] & 0x7F) as usize) + 1;

            if packet_type == 0x80 {
                // RLE packet: repeat one pixel
                cursor.read_exact(&mut pixel_buffer).map_err(|e| {
                    AssetError::ParseError(format!("Failed to read RLE pixel: {}", e))
                })?;

                for _ in 0..packet_length {
                    pixels.extend_from_slice(&pixel_buffer);
                }
            } else {
                // Raw packet: read multiple pixels
                for _ in 0..packet_length {
                    cursor.read_exact(&mut pixel_buffer).map_err(|e| {
                        AssetError::ParseError(format!("Failed to read raw pixel: {}", e))
                    })?;
                    pixels.extend_from_slice(&pixel_buffer);
                }
            }
        }

        Ok(pixels)
    }

    /// Get image width in pixels.
    pub fn width(&self) -> u16 {
        self.header.width
    }

    /// Get image height in pixels.
    pub fn height(&self) -> u16 {
        self.header.height
    }

    /// Get bits per pixel (8, 16, 24, or 32).
    pub fn bits_per_pixel(&self) -> u8 {
        self.header.bits_per_pixel
    }

    /// Check if image has alpha channel.
    pub fn has_alpha(&self) -> bool {
        self.header.bits_per_pixel == 32 || self.header.bits_per_pixel == 16
    }

    /// Check if image is vertically flipped (origin at top).
    pub fn is_origin_top(&self) -> bool {
        (self.header.image_descriptor & 0x20) != 0
    }

    /// Convert image to RGBA8 format.
    ///
    /// Returns raw RGBA pixel data (8 bits per channel, 4 channels).
    /// Handles BGR/BGRA to RGB/RGBA conversion automatically.
    pub fn to_rgba(&self) -> Result<Vec<u8>> {
        let pixel_count = (self.header.width as usize) * (self.header.height as usize);
        let mut rgba = vec![0u8; pixel_count * 4];

        match self.header.bits_per_pixel {
            24 => {
                // BGR -> RGBA
                for i in 0..pixel_count {
                    let src_offset = i * 3;
                    let dst_offset = i * 4;
                    rgba[dst_offset] = self.pixels[src_offset + 2]; // R
                    rgba[dst_offset + 1] = self.pixels[src_offset + 1]; // G
                    rgba[dst_offset + 2] = self.pixels[src_offset]; // B
                    rgba[dst_offset + 3] = 255; // A
                }
            }
            32 => {
                // BGRA -> RGBA
                for i in 0..pixel_count {
                    let src_offset = i * 4;
                    let dst_offset = i * 4;
                    rgba[dst_offset] = self.pixels[src_offset + 2]; // R
                    rgba[dst_offset + 1] = self.pixels[src_offset + 1]; // G
                    rgba[dst_offset + 2] = self.pixels[src_offset]; // B
                    rgba[dst_offset + 3] = self.pixels[src_offset + 3]; // A
                }
            }
            8 => {
                // Grayscale -> RGBA
                for i in 0..pixel_count {
                    let gray = self.pixels[i];
                    let dst_offset = i * 4;
                    rgba[dst_offset] = gray;
                    rgba[dst_offset + 1] = gray;
                    rgba[dst_offset + 2] = gray;
                    rgba[dst_offset + 3] = 255;
                }
            }
            _ => {
                return Err(AssetError::InvalidFormat(format!(
                    "Unsupported bits per pixel for RGBA conversion: {}",
                    self.header.bits_per_pixel
                )));
            }
        }

        // Flip vertically if origin is at bottom (default TGA behavior)
        if !self.is_origin_top() {
            let width = self.header.width as usize;
            let height = self.header.height as usize;
            let row_size = width * 4;

            for y in 0..height / 2 {
                let top_offset = y * row_size;
                let bottom_offset = (height - 1 - y) * row_size;

                for x in 0..row_size {
                    rgba.swap(top_offset + x, bottom_offset + x);
                }
            }
        }

        Ok(rgba)
    }

    /// Get raw pixel data (in original format, BGR/BGRA).
    pub fn raw_pixels(&self) -> &[u8] {
        &self.pixels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tga_header() {
        // Minimal 24-bit uncompressed TGA header
        let mut data = vec![
            0, // ID length
            0, // Color map type (none)
            2, // Image type (uncompressed RGB)
            0, 0, // Color map origin
            0, 0, // Color map length
            0, // Color map depth
            0, 0, // X origin
            0, 0, // Y origin
            2, 0, // Width = 2
            2, 0,  // Height = 2
            24, // Bits per pixel
            0,  // Image descriptor
        ];

        // Add 2x2 pixel data (12 bytes, BGR format)
        data.extend_from_slice(&[
            255, 0, 0, // Pixel 0 (Blue)
            0, 255, 0, // Pixel 1 (Green)
            0, 0, 255, // Pixel 2 (Red)
            255, 255, 255, // Pixel 3 (White)
        ]);

        let result = TgaImage::parse(&data);
        assert!(result.is_ok(), "Failed to parse TGA: {:?}", result.err());

        let tga = result.unwrap();
        assert_eq!(tga.width(), 2);
        assert_eq!(tga.height(), 2);
        assert_eq!(tga.bits_per_pixel(), 24);
        assert!(!tga.has_alpha());
    }

    #[test]
    fn test_tga_to_rgba() {
        let mut data = vec![
            0, // ID length
            0, // Color map type (none)
            2, // Image type (uncompressed RGB)
            0, 0, // Color map origin
            0, 0, // Color map length
            0, // Color map depth
            0, 0, // X origin
            0, 0, // Y origin
            1, 0, // Width = 1
            1, 0,  // Height = 1
            24, // Bits per pixel
            0,  // Image descriptor
        ];

        // Single blue pixel (BGR format)
        data.extend_from_slice(&[255, 0, 0]);

        let tga = TgaImage::parse(&data).unwrap();
        let rgba = tga.to_rgba().unwrap();

        // Should convert to RGBA: [0, 0, 255, 255] (red, green, blue, alpha)
        assert_eq!(rgba, vec![0, 0, 255, 255]);
    }
}
