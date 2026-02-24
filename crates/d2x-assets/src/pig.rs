//! PIG texture/bitmap file format parser
//!
//! PIG (Parallax Image Group) files contain texture and bitmap data for Descent 1 and 2.
//! Bitmaps are stored with RLE compression and use 8-bit indexed color (paletted).
//!
//! File format:
//! - Header: 8 bytes (signature "PPIG" + version 2)
//! - Bitmap count: 4 bytes
//! - Bitmap headers: N * 17 bytes
//! - Bitmap data: RLE-compressed pixel data
//!
//! Corresponds to: `include/piggy.h`, `gameio/piggy.cpp`, `2d/rle.cpp`

use crate::error::{AssetError, Result};
use std::collections::HashMap;
use std::io::{Cursor, Read};

/// PIG file signature "PPIG" (0x47495050 little-endian)
const PIG_SIGNATURE: u32 = 0x47495050;
/// PIG file version
const PIG_VERSION: i32 = 2;

/// Bitmap header size in bytes (same for D1 and D2)
const BITMAP_HEADER_SIZE: usize = 17;

/// RLE code marker (top 3 bits set: 0xE0)
const RLE_CODE: u8 = 0xE0;
/// Mask for count in RLE byte (bottom 5 bits: 0x1F)
const RLE_COUNT_MASK: u8 = 0x1F;

/// Bitmap flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitmapFlags {
    bits: u8,
}

impl BitmapFlags {
    /// Bitmap has transparent pixels (color index 255)
    pub const TRANSPARENT: u8 = 0x01;
    /// Super transparency (color index 254)
    pub const SUPER_TRANSPARENT: u8 = 0x02;
    /// Bitmap ignores lighting
    pub const NO_LIGHTING: u8 = 0x04;
    /// Bitmap data is RLE compressed
    pub const RLE: u8 = 0x08;
    /// Large RLE compressed bitmap
    pub const RLE_BIG: u8 = 0x10;

    pub fn new(bits: u8) -> Self {
        Self { bits }
    }

    pub fn is_transparent(self) -> bool {
        (self.bits & Self::TRANSPARENT) != 0
    }

    pub fn is_super_transparent(self) -> bool {
        (self.bits & Self::SUPER_TRANSPARENT) != 0
    }

    pub fn no_lighting(self) -> bool {
        (self.bits & Self::NO_LIGHTING) != 0
    }

    pub fn is_rle(self) -> bool {
        (self.bits & Self::RLE) != 0
    }

    pub fn is_rle_big(self) -> bool {
        (self.bits & Self::RLE_BIG) != 0
    }
}

/// Bitmap header from PIG file
#[derive(Debug, Clone)]
pub struct BitmapHeader {
    /// Bitmap name (up to 8 characters, null-padded)
    pub name: String,
    /// Animation flags (bits 0-5: frame num, bit 6: ABM flag)
    pub dflags: u8,
    /// Bitmap width in pixels
    pub width: u16,
    /// Bitmap height in pixels
    pub height: u16,
    /// Bitmap flags (transparent, RLE, etc.)
    pub flags: BitmapFlags,
    /// Average color index (for minimap)
    pub avg_color: u8,
    /// Offset from start of bitmap data section
    pub offset: i32,
}

impl BitmapHeader {
    /// Get animation frame number (bits 0-5 of dflags)
    pub fn frame_number(&self) -> u8 {
        self.dflags & 0x3F
    }

    /// Check if this is an animated bitmap (bit 6 of dflags)
    pub fn is_animated(&self) -> bool {
        (self.dflags & 0x40) != 0
    }

    /// Parse D2 bitmap header (17 bytes with wh_extra field)
    fn parse_d2(data: &[u8]) -> Result<Self> {
        if data.len() < BITMAP_HEADER_SIZE {
            return Err(AssetError::InvalidPigFormat(
                "Bitmap header too short".to_string(),
            ));
        }

        // Read name (8 bytes, null-padded)
        let name_bytes = &data[0..8];
        let name = String::from_utf8_lossy(name_bytes)
            .trim_end_matches('\0')
            .to_string();

        let dflags = data[8];
        let width_lo = data[9] as u16;
        let height_lo = data[10] as u16;
        let wh_extra = data[11];
        let flags = BitmapFlags::new(data[12]);
        let avg_color = data[13];

        // Offset is little-endian i32
        let offset = i32::from_le_bytes([data[14], data[15], data[16], data[17]]);

        // Calculate actual width and height (12 bits each)
        let width = width_lo | (((wh_extra & 0x0F) as u16) << 8);
        let height = height_lo | (((wh_extra & 0xF0) as u16) << 4);

        Ok(Self {
            name,
            dflags,
            width,
            height,
            flags,
            avg_color,
            offset,
        })
    }

    /// Parse D1 bitmap header (17 bytes, no wh_extra field)
    fn parse_d1(data: &[u8]) -> Result<Self> {
        if data.len() < BITMAP_HEADER_SIZE {
            return Err(AssetError::InvalidPigFormat(
                "Bitmap header too short".to_string(),
            ));
        }

        let name_bytes = &data[0..8];
        let name = String::from_utf8_lossy(name_bytes)
            .trim_end_matches('\0')
            .to_string();

        let dflags = data[8];
        let width = data[9] as u16; // Full width (max 255)
        let height = data[10] as u16; // Full height (max 255)
        let flags = BitmapFlags::new(data[11]);
        let avg_color = data[12];

        let offset = i32::from_le_bytes([data[13], data[14], data[15], data[16]]);

        Ok(Self {
            name,
            dflags,
            width,
            height,
            flags,
            avg_color,
            offset,
        })
    }
}

/// Bitmap data (8-bit indexed color)
#[derive(Debug, Clone)]
pub struct BitmapData {
    /// Indexed pixel data (one byte per pixel)
    pub pixels: Vec<u8>,
    /// Width in pixels
    pub width: u16,
    /// Height in pixels
    pub height: u16,
}

impl BitmapData {
    /// Create new bitmap data
    pub fn new(width: u16, height: u16, pixels: Vec<u8>) -> Result<Self> {
        let expected_size = (width as usize) * (height as usize);
        if pixels.len() != expected_size {
            return Err(AssetError::InvalidPigFormat(format!(
                "Bitmap pixel data size mismatch: expected {}, got {}",
                expected_size,
                pixels.len()
            )));
        }
        Ok(Self {
            pixels,
            width,
            height,
        })
    }

    /// Convert indexed pixels to RGBA using a palette
    ///
    /// Palette should be 256 RGB triplets (768 bytes total).
    /// Color index 255 becomes fully transparent.
    pub fn to_rgba(&self, palette: &[u8]) -> Result<Vec<u8>> {
        if palette.len() != 768 {
            return Err(AssetError::InvalidPigFormat(format!(
                "Palette must be 768 bytes (256 RGB triplets), got {}",
                palette.len()
            )));
        }

        let mut rgba = Vec::with_capacity(self.pixels.len() * 4);
        for &index in &self.pixels {
            if index == 255 {
                // Transparent pixel
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let i = (index as usize) * 3;
                rgba.push(palette[i]); // R
                rgba.push(palette[i + 1]); // G
                rgba.push(palette[i + 2]); // B
                rgba.push(255); // A
            }
        }
        Ok(rgba)
    }
}

/// PIG file parser
pub struct PigFile {
    /// All bitmap headers indexed by name (case-insensitive)
    headers: HashMap<String, BitmapHeader>,
    /// Offset where bitmap data section starts
    data_start: usize,
    /// Raw file data
    data: Vec<u8>,
    /// Whether this is a Descent 1 PIG file
    is_d1: bool,
}

impl PigFile {
    /// Parse a PIG file from raw bytes
    ///
    /// The `is_d1` parameter determines whether to parse D1 or D2 bitmap headers.
    /// D1 headers have no wh_extra field, limiting dimensions to 255x255.
    pub fn parse(data: Vec<u8>, is_d1: bool) -> Result<Self> {
        let mut cursor = Cursor::new(&data);

        // Read and verify signature
        let mut sig_bytes = [0u8; 4];
        cursor.read_exact(&mut sig_bytes)?;
        let signature = u32::from_le_bytes(sig_bytes);

        if signature != PIG_SIGNATURE {
            return Err(AssetError::InvalidPigFormat(format!(
                "Invalid PIG signature: expected 0x{:08X}, got 0x{:08X}",
                PIG_SIGNATURE, signature
            )));
        }

        // Read and verify version
        let mut ver_bytes = [0u8; 4];
        cursor.read_exact(&mut ver_bytes)?;
        let version = i32::from_le_bytes(ver_bytes);

        if version != PIG_VERSION {
            return Err(AssetError::InvalidPigFormat(format!(
                "Unsupported PIG version: expected {}, got {}",
                PIG_VERSION, version
            )));
        }

        // Read bitmap count
        let mut count_bytes = [0u8; 4];
        cursor.read_exact(&mut count_bytes)?;
        let num_bitmaps = i32::from_le_bytes(count_bytes);

        if num_bitmaps < 0 {
            return Err(AssetError::InvalidPigFormat(format!(
                "Invalid bitmap count: {}",
                num_bitmaps
            )));
        }

        // Calculate data section start
        let data_start = 12 + (num_bitmaps as usize * BITMAP_HEADER_SIZE);

        // Read all bitmap headers
        let mut headers = HashMap::new();
        for _ in 0..num_bitmaps {
            let mut header_data = [0u8; BITMAP_HEADER_SIZE];
            cursor.read_exact(&mut header_data)?;

            let header = if is_d1 {
                BitmapHeader::parse_d1(&header_data)?
            } else {
                BitmapHeader::parse_d2(&header_data)?
            };

            // Store with lowercase key for case-insensitive lookup
            headers.insert(header.name.to_lowercase(), header);
        }

        Ok(Self {
            headers,
            data_start,
            data,
            is_d1,
        })
    }

    /// Get all bitmap headers
    pub fn headers(&self) -> impl Iterator<Item = &BitmapHeader> {
        self.headers.values()
    }

    /// Find a bitmap header by name (case-insensitive)
    pub fn find_bitmap(&self, name: &str) -> Option<&BitmapHeader> {
        self.headers.get(&name.to_lowercase())
    }

    /// Load bitmap data by name
    pub fn load_bitmap(&self, name: &str) -> Result<BitmapData> {
        let header = self
            .find_bitmap(name)
            .ok_or_else(|| AssetError::NotFound(format!("Bitmap not found: {}", name)))?;

        self.load_bitmap_by_header(header)
    }

    /// Load bitmap data using a header
    pub fn load_bitmap_by_header(&self, header: &BitmapHeader) -> Result<BitmapData> {
        let offset = self.data_start + header.offset as usize;

        if offset >= self.data.len() {
            return Err(AssetError::InvalidPigFormat(format!(
                "Bitmap offset {} out of bounds (file size: {})",
                offset,
                self.data.len()
            )));
        }

        let pixels = if header.flags.is_rle() {
            // RLE compressed
            rle_decompress(&self.data[offset..], header.width, header.height)?
        } else {
            // Uncompressed
            let size = (header.width as usize) * (header.height as usize);
            if offset + size > self.data.len() {
                return Err(AssetError::InvalidPigFormat(format!(
                    "Bitmap data extends beyond file: offset={}, size={}, file_size={}",
                    offset,
                    size,
                    self.data.len()
                )));
            }
            self.data[offset..offset + size].to_vec()
        };

        BitmapData::new(header.width, header.height, pixels)
    }

    /// Get the number of bitmaps in this file
    pub fn bitmap_count(&self) -> usize {
        self.headers.len()
    }

    /// Check if this is a Descent 1 PIG file
    pub fn is_descent1(&self) -> bool {
        self.is_d1
    }
}

/// Decompress RLE-encoded bitmap data
///
/// RLE format:
/// - Bytes < 0xE0: literal pixel value
/// - Bytes >= 0xE0: RLE code
///   - Count in bottom 5 bits (0-31)
///   - If count == 0: end marker
///   - Otherwise: next byte is color, repeat 'count' times
fn rle_decompress(data: &[u8], width: u16, height: u16) -> Result<Vec<u8>> {
    let expected_size = (width as usize) * (height as usize);
    let mut output = Vec::with_capacity(expected_size);
    let mut i = 0;

    while i < data.len() {
        let byte = data[i];
        i += 1;

        if (byte & RLE_CODE) != RLE_CODE {
            // Literal byte
            output.push(byte);
        } else {
            // RLE sequence
            let count = byte & RLE_COUNT_MASK;

            if count == 0 {
                // End marker
                break;
            }

            if i >= data.len() {
                return Err(AssetError::InvalidPigFormat(
                    "RLE data truncated: missing color byte".to_string(),
                ));
            }

            let color = data[i];
            i += 1;

            // Repeat color 'count' times
            output.extend(std::iter::repeat(color).take(count as usize));
        }

        // Safety check: don't exceed expected size
        if output.len() > expected_size {
            return Err(AssetError::InvalidPigFormat(format!(
                "RLE decompression produced too much data: expected {}, got {}",
                expected_size,
                output.len()
            )));
        }
    }

    // Verify we got the expected amount of data
    if output.len() != expected_size {
        return Err(AssetError::InvalidPigFormat(format!(
            "RLE decompression size mismatch: expected {}, got {}",
            expected_size,
            output.len()
        )));
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rle_decompress_literal() {
        // Simple literal bytes (no RLE codes)
        let data = vec![1, 2, 3, 4, 0xE0]; // End marker
        let result = rle_decompress(&data, 2, 2).unwrap();
        assert_eq!(result, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_rle_decompress_run() {
        // RLE: 5 times color 42
        let data = vec![0xE5, 42, 0xE0]; // count=5, color=42, end
        let result = rle_decompress(&data, 5, 1).unwrap();
        assert_eq!(result, vec![42, 42, 42, 42, 42]);
    }

    #[test]
    fn test_rle_decompress_mixed() {
        // Mixed: literal 1, run of 3x color 2, literal 3
        let data = vec![1, 0xE3, 2, 3, 0xE0]; // 1, run(3,2), 3, end
        let result = rle_decompress(&data, 5, 1).unwrap();
        assert_eq!(result, vec![1, 2, 2, 2, 3]);
    }

    #[test]
    fn test_bitmap_flags() {
        let flags = BitmapFlags::new(BitmapFlags::TRANSPARENT | BitmapFlags::RLE);
        assert!(flags.is_transparent());
        assert!(flags.is_rle());
        assert!(!flags.is_super_transparent());
    }

    #[test]
    fn test_bitmap_header_animation() {
        let header = BitmapHeader {
            name: "test".to_string(),
            dflags: 0x45, // frame 5, animated
            width: 64,
            height: 64,
            flags: BitmapFlags::new(0),
            avg_color: 0,
            offset: 0,
        };
        assert_eq!(header.frame_number(), 5);
        assert!(header.is_animated());
    }
}
