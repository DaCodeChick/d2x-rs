//! PCX (PC Paintbrush) image format parser.
//!
//! PCX is a raster graphics file format developed by ZSoft Corporation.
//! It was widely used in Descent 1 & 2 for briefing screens, ending screens,
//! and other full-screen images.
//!
//! ## Format Overview
//!
//! PCX files consist of:
//! 1. Header (128 bytes) - image metadata
//! 2. Image data (RLE compressed scanlines)
//! 3. Optional palette (768 bytes at end for 256-color images)
//!
//! ## Supported Formats
//!
//! - 8-bit indexed color (256 colors with palette)
//! - 24-bit RGB (3 color planes)
//! - RLE compression (default)
//!
//! ## Example
//!
//! ```no_run
//! use descent_core::pcx::PcxImage;
//!
//! let data = std::fs::read("brief1b.pcx").unwrap();
//! let pcx = PcxImage::parse(&data).unwrap();
//!
//! println!("{}x{} image, {} bits per pixel",
//!     pcx.width(), pcx.height(), pcx.bits_per_pixel());
//!
//! // Convert to RGBA
//! let rgba = pcx.to_rgba().unwrap();
//!
//! // Convert to TGA
//! let tga = pcx.to_tga().unwrap();
//! std::fs::write("brief1b.tga", tga).unwrap();
//! ```

use crate::error::{AssetError, Result};
use std::io::{Cursor, Read, Write};

/// PCX image data.
#[derive(Debug, Clone)]
pub struct PcxImage {
    header: PcxHeader,
    pixels: Vec<u8>,
    palette: Option<Vec<u8>>, // 768 bytes (256 * RGB)
}

#[derive(Debug, Clone)]
struct PcxHeader {
    manufacturer: u8,
    #[allow(dead_code)]
    version: u8,
    encoding: u8,
    bits_per_pixel: u8,
    x_min: u16,
    y_min: u16,
    x_max: u16,
    y_max: u16,
    #[allow(dead_code)]
    h_dpi: u16,
    #[allow(dead_code)]
    v_dpi: u16,
    #[allow(dead_code)]
    ega_palette: [u8; 48],
    #[allow(dead_code)]
    reserved: u8,
    num_planes: u8,
    bytes_per_line: u16,
    #[allow(dead_code)]
    palette_info: u16,
    #[allow(dead_code)]
    h_screen_size: u16,
    #[allow(dead_code)]
    v_screen_size: u16,
}

impl PcxImage {
    /// Parse a PCX image from binary data.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw PCX file data
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File format is invalid
    /// - Header is malformed
    /// - Image data is corrupted
    /// - Unsupported format
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 128 {
            return Err(AssetError::ParseError(
                "PCX file too small for header".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);
        let header = Self::parse_header(&mut cursor)?;

        // Validate header
        if header.manufacturer != 0x0A {
            return Err(AssetError::InvalidFormat(format!(
                "Invalid PCX manufacturer byte: 0x{:02X}",
                header.manufacturer
            )));
        }

        if header.encoding != 1 {
            return Err(AssetError::InvalidFormat(format!(
                "Unsupported PCX encoding: {}",
                header.encoding
            )));
        }

        // Calculate image dimensions
        let width = (header.x_max - header.x_min + 1) as usize;
        let height = (header.y_max - header.y_min + 1) as usize;

        // Decompress image data
        let pixels = Self::decompress_rle(&mut cursor, &header, width, height)?;

        // Check for 256-color palette at end of file
        let palette = if header.bits_per_pixel == 8 && header.num_planes == 1 {
            // Palette marker is 0x0C, followed by 768 bytes (256 colors * 3 channels)
            if data.len() >= 769 && data[data.len() - 769] == 0x0C {
                let palette_start = data.len() - 768;
                Some(data[palette_start..].to_vec())
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            header,
            pixels,
            palette,
        })
    }

    fn parse_header(cursor: &mut Cursor<&[u8]>) -> Result<PcxHeader> {
        let mut header_bytes = [0u8; 128];
        cursor
            .read_exact(&mut header_bytes)
            .map_err(|e| AssetError::ParseError(format!("Failed to read PCX header: {}", e)))?;

        Ok(PcxHeader {
            manufacturer: header_bytes[0],
            version: header_bytes[1],
            encoding: header_bytes[2],
            bits_per_pixel: header_bytes[3],
            x_min: u16::from_le_bytes([header_bytes[4], header_bytes[5]]),
            y_min: u16::from_le_bytes([header_bytes[6], header_bytes[7]]),
            x_max: u16::from_le_bytes([header_bytes[8], header_bytes[9]]),
            y_max: u16::from_le_bytes([header_bytes[10], header_bytes[11]]),
            h_dpi: u16::from_le_bytes([header_bytes[12], header_bytes[13]]),
            v_dpi: u16::from_le_bytes([header_bytes[14], header_bytes[15]]),
            ega_palette: {
                let mut pal = [0u8; 48];
                pal.copy_from_slice(&header_bytes[16..64]);
                pal
            },
            reserved: header_bytes[64],
            num_planes: header_bytes[65],
            bytes_per_line: u16::from_le_bytes([header_bytes[66], header_bytes[67]]),
            palette_info: u16::from_le_bytes([header_bytes[68], header_bytes[69]]),
            h_screen_size: u16::from_le_bytes([header_bytes[70], header_bytes[71]]),
            v_screen_size: u16::from_le_bytes([header_bytes[72], header_bytes[73]]),
        })
    }

    fn decompress_rle(
        cursor: &mut Cursor<&[u8]>,
        header: &PcxHeader,
        width: usize,
        height: usize,
    ) -> Result<Vec<u8>> {
        let bytes_per_line = header.bytes_per_line as usize;
        let num_planes = header.num_planes as usize;
        let total_bytes = bytes_per_line * num_planes * height;

        let mut pixels = Vec::with_capacity(total_bytes);
        let mut scanline = vec![0u8; bytes_per_line * num_planes];

        for _ in 0..height {
            // Decompress one scanline
            let mut pos = 0;

            while pos < scanline.len() {
                let mut byte = [0u8; 1];
                if cursor.read_exact(&mut byte).is_err() {
                    return Err(AssetError::ParseError(
                        "Unexpected end of PCX data".to_string(),
                    ));
                }

                let value = byte[0];

                if (value & 0xC0) == 0xC0 {
                    // RLE packet: repeat count in lower 6 bits
                    let count = (value & 0x3F) as usize;

                    if cursor.read_exact(&mut byte).is_err() {
                        return Err(AssetError::ParseError(
                            "Unexpected end of PCX data".to_string(),
                        ));
                    }

                    let pixel = byte[0];

                    for i in 0..count {
                        if pos + i < scanline.len() {
                            scanline[pos + i] = pixel;
                        }
                    }

                    pos += count;
                } else {
                    // Raw pixel
                    if pos < scanline.len() {
                        scanline[pos] = value;
                    }
                    pos += 1;
                }
            }

            // Convert planar format to interleaved if needed
            if num_planes == 1 {
                // 8-bit indexed - already interleaved
                pixels.extend_from_slice(&scanline[..width]);
            } else if num_planes == 3 {
                // 24-bit RGB - planes stored sequentially (RRRRGGGGBBBB)
                for x in 0..width {
                    pixels.push(scanline[x]); // R
                    pixels.push(scanline[bytes_per_line + x]); // G
                    pixels.push(scanline[bytes_per_line * 2 + x]); // B
                }
            } else {
                return Err(AssetError::InvalidFormat(format!(
                    "Unsupported number of planes: {}",
                    num_planes
                )));
            }
        }

        Ok(pixels)
    }

    /// Get image width in pixels.
    pub fn width(&self) -> u16 {
        self.header.x_max - self.header.x_min + 1
    }

    /// Get image height in pixels.
    pub fn height(&self) -> u16 {
        self.header.y_max - self.header.y_min + 1
    }

    /// Get bits per pixel (8 or 24).
    pub fn bits_per_pixel(&self) -> u8 {
        self.header.bits_per_pixel * self.header.num_planes
    }

    /// Check if image is indexed (8-bit with palette).
    pub fn is_indexed(&self) -> bool {
        self.header.bits_per_pixel == 8 && self.header.num_planes == 1
    }

    /// Convert image to RGBA8 format.
    ///
    /// Returns raw RGBA pixel data (8 bits per channel, 4 channels).
    pub fn to_rgba(&self) -> Result<Vec<u8>> {
        let width = self.width() as usize;
        let height = self.height() as usize;
        let mut rgba = vec![0u8; width * height * 4];

        if self.is_indexed() {
            // 8-bit indexed with palette
            let palette = self.palette.as_ref().ok_or_else(|| {
                AssetError::InvalidFormat("No palette found for indexed image".to_string())
            })?;

            for i in 0..width * height {
                let index = self.pixels[i] as usize;
                let pal_offset = index * 3;

                if pal_offset + 2 < palette.len() {
                    rgba[i * 4] = palette[pal_offset]; // R
                    rgba[i * 4 + 1] = palette[pal_offset + 1]; // G
                    rgba[i * 4 + 2] = palette[pal_offset + 2]; // B
                    rgba[i * 4 + 3] = 255; // A
                }
            }
        } else if self.header.num_planes == 3 {
            // 24-bit RGB
            for i in 0..width * height {
                rgba[i * 4] = self.pixels[i * 3]; // R
                rgba[i * 4 + 1] = self.pixels[i * 3 + 1]; // G
                rgba[i * 4 + 2] = self.pixels[i * 3 + 2]; // B
                rgba[i * 4 + 3] = 255; // A
            }
        } else {
            return Err(AssetError::InvalidFormat(format!(
                "Unsupported PCX format for RGBA conversion: {} bpp, {} planes",
                self.header.bits_per_pixel, self.header.num_planes
            )));
        }

        Ok(rgba)
    }

    /// Convert PCX image to TGA format.
    ///
    /// Returns raw TGA file data that can be written directly to disk.
    pub fn to_tga(&self) -> Result<Vec<u8>> {
        let width = self.width();
        let height = self.height();
        let rgba = self.to_rgba()?;

        let mut tga = Vec::new();

        // TGA header (18 bytes) - uncompressed RGBA
        tga.write_all(&[
            0, // ID length
            0, // Color map type (no palette)
            2, // Image type (uncompressed RGB)
            0, 0, // Color map origin
            0, 0, // Color map length
            0, // Color map depth
            0, 0, // X origin
            0, 0, // Y origin
        ])
        .map_err(|e| AssetError::ParseError(format!("Failed to write TGA header: {}", e)))?;

        tga.write_all(&width.to_le_bytes())
            .map_err(|e| AssetError::ParseError(format!("Failed to write TGA width: {}", e)))?;
        tga.write_all(&height.to_le_bytes())
            .map_err(|e| AssetError::ParseError(format!("Failed to write TGA height: {}", e)))?;

        tga.write_all(&[
            32, // Bits per pixel (32 = BGRA)
            8,  // Image descriptor (8 = origin at top-left, 8 bits alpha)
        ])
        .map_err(|e| AssetError::ParseError(format!("Failed to write TGA descriptor: {}", e)))?;

        // Convert RGBA to BGRA for TGA
        for i in 0..(width as usize * height as usize) {
            let r = rgba[i * 4];
            let g = rgba[i * 4 + 1];
            let b = rgba[i * 4 + 2];
            let a = rgba[i * 4 + 3];

            tga.write_all(&[b, g, r, a])
                .map_err(|e| AssetError::ParseError(format!("Failed to write TGA pixel: {}", e)))?;
        }

        Ok(tga)
    }

    /// Get raw pixel data (in original format).
    pub fn raw_pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Get palette data if available (256 colors * 3 channels = 768 bytes).
    pub fn palette(&self) -> Option<&[u8]> {
        self.palette.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pcx_8bit() -> Vec<u8> {
        let mut data = vec![0u8; 128];

        // PCX header for 2x2 8-bit indexed image
        data[0] = 0x0A; // Manufacturer
        data[1] = 5; // Version
        data[2] = 1; // RLE encoding
        data[3] = 8; // Bits per pixel
        data[4..6].copy_from_slice(&0u16.to_le_bytes()); // x_min
        data[6..8].copy_from_slice(&0u16.to_le_bytes()); // y_min
        data[8..10].copy_from_slice(&1u16.to_le_bytes()); // x_max = 1 (width 2)
        data[10..12].copy_from_slice(&1u16.to_le_bytes()); // y_max = 1 (height 2)
        data[65] = 1; // num_planes
        data[66..68].copy_from_slice(&2u16.to_le_bytes()); // bytes_per_line

        // RLE compressed image data (2x2 pixels)
        data.push(0xC2); // Repeat 2 times
        data.push(0); // Pixel value 0
        data.push(0xC2); // Repeat 2 times
        data.push(1); // Pixel value 1

        // Palette marker and palette
        data.push(0x0C); // Palette marker

        // 256-color palette (768 bytes)
        for i in 0..256 {
            data.push(i as u8); // R
            data.push(i as u8); // G
            data.push(i as u8); // B
        }

        data
    }

    #[test]
    fn test_parse_pcx_8bit() {
        let data = create_test_pcx_8bit();
        let result = PcxImage::parse(&data);
        assert!(result.is_ok(), "Failed to parse PCX: {:?}", result.err());

        let pcx = result.unwrap();
        assert_eq!(pcx.width(), 2);
        assert_eq!(pcx.height(), 2);
        assert_eq!(pcx.bits_per_pixel(), 8);
        assert!(pcx.is_indexed());
        assert!(pcx.palette().is_some());
    }

    #[test]
    fn test_pcx_to_rgba() {
        let data = create_test_pcx_8bit();
        let pcx = PcxImage::parse(&data).unwrap();
        let rgba = pcx.to_rgba().unwrap();

        // 2x2 image = 16 bytes RGBA
        assert_eq!(rgba.len(), 16);

        // First pixel: index 0 -> RGB(0,0,0), alpha 255
        assert_eq!(&rgba[0..4], &[0, 0, 0, 255]);

        // Second pixel: index 0 -> RGB(0,0,0), alpha 255
        assert_eq!(&rgba[4..8], &[0, 0, 0, 255]);
    }

    #[test]
    fn test_pcx_to_tga() {
        let data = create_test_pcx_8bit();
        let pcx = PcxImage::parse(&data).unwrap();
        let tga = pcx.to_tga().unwrap();

        // TGA file should have:
        // - 18 byte header
        // - 2x2 pixels * 4 bytes (BGRA) = 16 bytes
        assert_eq!(tga.len(), 18 + 16);

        // Check TGA signature
        assert_eq!(tga[2], 2); // Uncompressed RGB
        assert_eq!(tga[16], 32); // 32 bpp
    }

    #[test]
    fn test_invalid_pcx() {
        // Too small
        let data = vec![0u8; 10];
        assert!(PcxImage::parse(&data).is_err());

        // Invalid manufacturer byte
        let mut data = vec![0u8; 128];
        data[0] = 0xFF;
        assert!(PcxImage::parse(&data).is_err());
    }
}
