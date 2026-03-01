//! IFF (Interchange File Format) parser for Descent.
//!
//! This module implements parsing of IFF files, specifically the ILBM (Interleaved Bitmap)
//! and PBM (Planar Bitmap) formats used in Descent for briefing screens and other images.
//! These files typically have a `.bbm` extension in Descent.
//!
//! # Format Overview
//!
//! IFF is a chunk-based format originally from Commodore Amiga. Each chunk has:
//! - 4-byte chunk ID (e.g., "FORM", "BMHD", "BODY")
//! - 4-byte length (big-endian)
//! - Chunk data
//! - Optional padding byte if length is odd
//!
//! # Supported Chunks
//!
//! - **FORM**: Container chunk, must be first
//! - **ILBM/PBM**: Form type identifier
//! - **BMHD**: Bitmap header (width, height, depth, compression, etc.)
//! - **CMAP**: Color map (palette)
//! - **BODY**: Bitmap data (may be RLE compressed)
//!
//! # Example
//!
//! ```no_run
//! use descent_core::iff::IffFile;
//!
//! let data = std::fs::read("briefing.bbm")?;
//! let iff = IffFile::parse(&data)?;
//!
//! println!("Size: {}x{}", iff.width(), iff.height());
//! println!("Bit planes: {}", iff.bit_planes());
//! println!("Compression: {:?}", iff.compression());
//!
//! // Get decompressed bitmap data
//! let pixels = iff.bitmap_data();
//!
//! // Get palette if present
//! if let Some(palette) = iff.palette() {
//!     println!("Palette entries: {}", palette.len() / 3);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::error::{AssetError, Result};
use crate::io::ReadExt;
use std::io::{Cursor, Read};

/// IFF chunk identifier (4-byte signature)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkId(pub [u8; 4]);

impl ChunkId {
    pub const FORM: ChunkId = ChunkId(*b"FORM");
    pub const ILBM: ChunkId = ChunkId(*b"ILBM");
    pub const PBM: ChunkId = ChunkId(*b"PBM ");
    pub const BMHD: ChunkId = ChunkId(*b"BMHD");
    pub const CMAP: ChunkId = ChunkId(*b"CMAP");
    pub const BODY: ChunkId = ChunkId(*b"BODY");
    pub const TINY: ChunkId = ChunkId(*b"TINY");
    pub const ANHD: ChunkId = ChunkId(*b"ANHD");
    pub const DLTA: ChunkId = ChunkId(*b"DLTA");

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap_or("????")
    }
}

impl From<[u8; 4]> for ChunkId {
    fn from(bytes: [u8; 4]) -> Self {
        ChunkId(bytes)
    }
}

/// Bitmap type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitmapType {
    /// Planar bitmap (8-bit indexed)
    Pbm,
    /// Interleaved bitmap (bitplanes)
    Ilbm,
}

/// Compression method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    /// No compression
    None = 0,
    /// ByteRun1 RLE compression
    ByteRun1 = 1,
}

/// Masking type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Masking {
    /// No masking
    None = 0,
    /// Has mask plane
    HasMask = 1,
    /// Has transparent color
    TransparentColor = 2,
}

/// Bitmap header (BMHD chunk)
#[derive(Debug, Clone)]
pub struct BitmapHeader {
    /// Image width in pixels
    pub width: u16,
    /// Image height in pixels
    pub height: u16,
    /// X position (usually 0)
    pub x: i16,
    /// Y position (usually 0)
    pub y: i16,
    /// Number of bit planes
    pub bit_planes: u8,
    /// Masking type
    pub masking: Masking,
    /// Compression method
    pub compression: Compression,
    /// Transparent color index (if masking == TransparentColor)
    pub transparent_color: u16,
    /// X aspect ratio (usually 5)
    pub x_aspect: u8,
    /// Y aspect ratio (usually 6)
    pub y_aspect: u8,
    /// Page width
    pub page_width: u16,
    /// Page height
    pub page_height: u16,
}

/// Parsed IFF file
pub struct IffFile {
    /// Bitmap type (PBM or ILBM)
    bitmap_type: BitmapType,
    /// Bitmap header
    header: BitmapHeader,
    /// Color palette (RGB triplets, 256 colors max)
    palette: Option<Vec<u8>>,
    /// Decompressed bitmap data
    bitmap_data: Vec<u8>,
}

impl IffFile {
    /// Parse an IFF file from raw bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw IFF file data
    ///
    /// # Returns
    ///
    /// Parsed IFF file with decompressed bitmap data
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use descent_core::iff::IffFile;
    /// let data = std::fs::read("briefing.bbm")?;
    /// let iff = IffFile::parse(&data)?;
    /// println!("Image size: {}x{}", iff.width(), iff.height());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);

        // Read FORM chunk
        let form_id = read_chunk_id(&mut cursor)?;
        if form_id != ChunkId::FORM {
            return Err(AssetError::InvalidFormat(format!(
                "Not an IFF file: expected FORM, got {}",
                form_id.as_str()
            )));
        }

        let form_length = cursor.read_u32_be()?;
        let form_end = cursor.position() + form_length as u64;

        // Read form type (ILBM or PBM)
        let form_type = read_chunk_id(&mut cursor)?;
        let bitmap_type = match form_type {
            ChunkId::ILBM => BitmapType::Ilbm,
            ChunkId::PBM => BitmapType::Pbm,
            _ => {
                return Err(AssetError::InvalidFormat(format!(
                    "Unknown IFF form type: {}",
                    form_type.as_str()
                )))
            }
        };

        let mut header: Option<BitmapHeader> = None;
        let mut palette: Option<Vec<u8>> = None;
        let mut compressed_body: Option<Vec<u8>> = None;

        // Parse chunks
        while cursor.position() < form_end {
            let chunk_id = read_chunk_id(&mut cursor)?;
            let chunk_len = cursor.read_u32_be()? as usize;

            match chunk_id {
                ChunkId::BMHD => {
                    header = Some(parse_bmhd(&mut cursor)?);
                }
                ChunkId::CMAP => {
                    palette = Some(read_chunk_data(&mut cursor, chunk_len)?);
                }
                ChunkId::BODY => {
                    compressed_body = Some(read_chunk_data(&mut cursor, chunk_len)?);
                }
                _ => {
                    // Skip unknown chunks
                    skip_chunk(&mut cursor, chunk_len)?;
                }
            }

            // Chunks must be word-aligned (pad byte if odd length)
            if chunk_len & 1 != 0 {
                cursor.read_u8()?;
            }
        }

        let header = header
            .ok_or_else(|| AssetError::InvalidFormat("IFF file missing BMHD chunk".to_string()))?;

        let compressed_body = compressed_body
            .ok_or_else(|| AssetError::InvalidFormat("IFF file missing BODY chunk".to_string()))?;

        // Decompress bitmap data
        let bitmap_data = decompress_body(&header, &compressed_body, bitmap_type)?;

        Ok(IffFile {
            bitmap_type,
            header,
            palette,
            bitmap_data,
        })
    }

    /// Get the bitmap type (PBM or ILBM)
    pub fn bitmap_type(&self) -> BitmapType {
        self.bitmap_type
    }

    /// Get the image width in pixels
    pub fn width(&self) -> u16 {
        self.header.width
    }

    /// Get the image height in pixels
    pub fn height(&self) -> u16 {
        self.header.height
    }

    /// Get the number of bit planes
    pub fn bit_planes(&self) -> u8 {
        self.header.bit_planes
    }

    /// Get the compression method
    pub fn compression(&self) -> Compression {
        self.header.compression
    }

    /// Get the masking type
    pub fn masking(&self) -> Masking {
        self.header.masking
    }

    /// Get the transparent color index (if masking == TransparentColor)
    pub fn transparent_color(&self) -> Option<u16> {
        if matches!(self.header.masking, Masking::TransparentColor) {
            Some(self.header.transparent_color)
        } else {
            None
        }
    }

    /// Get the color palette as RGB triplets (if present)
    ///
    /// Returns a byte array where each 3 bytes represents one RGB color.
    pub fn palette(&self) -> Option<&[u8]> {
        self.palette.as_deref()
    }

    /// Get the decompressed bitmap data
    ///
    /// For PBM: linear 8-bit indexed pixel data (width * height bytes)
    /// For ILBM: planar data that needs to be converted to chunky format
    pub fn bitmap_data(&self) -> &[u8] {
        &self.bitmap_data
    }

    /// Convert ILBM planar data to chunky 8-bit indexed format
    ///
    /// For PBM images, this returns the data as-is.
    /// For ILBM images, this converts from bitplanes to pixel values.
    pub fn to_chunky(&self) -> Vec<u8> {
        match self.bitmap_type {
            BitmapType::Pbm => self.bitmap_data.clone(),
            BitmapType::Ilbm => convert_ilbm_to_chunky(
                &self.bitmap_data,
                self.header.width,
                self.header.height,
                self.header.bit_planes,
            ),
        }
    }
}

/// Read a 4-byte chunk ID (big-endian)
fn read_chunk_id(cursor: &mut Cursor<&[u8]>) -> Result<ChunkId> {
    let mut id = [0u8; 4];
    cursor.read_exact(&mut id)?;
    Ok(ChunkId(id))
}

/// Read chunk data
fn read_chunk_data(cursor: &mut Cursor<&[u8]>, len: usize) -> Result<Vec<u8>> {
    let mut data = vec![0u8; len];
    cursor.read_exact(&mut data)?;
    Ok(data)
}

/// Skip a chunk
fn skip_chunk(cursor: &mut Cursor<&[u8]>, len: usize) -> Result<()> {
    cursor.skip_bytes(len)?;
    Ok(())
}

/// Parse BMHD (bitmap header) chunk
fn parse_bmhd(cursor: &mut Cursor<&[u8]>) -> Result<BitmapHeader> {
    let width = cursor.read_u16_be()?;
    let height = cursor.read_u16_be()?;
    let x = cursor.read_i16_be()?;
    let y = cursor.read_i16_be()?;
    let bit_planes = cursor.read_u8()?;
    let masking_byte = cursor.read_u8()?;
    let compression_byte = cursor.read_u8()?;
    let _pad = cursor.read_u8()?; // padding byte
    let transparent_color = cursor.read_u16_be()?;
    let x_aspect = cursor.read_u8()?;
    let y_aspect = cursor.read_u8()?;
    let page_width = cursor.read_u16_be()?;
    let page_height = cursor.read_u16_be()?;

    let masking = match masking_byte {
        0 => Masking::None,
        1 => Masking::HasMask,
        2 => Masking::TransparentColor,
        _ => {
            return Err(AssetError::InvalidFormat(format!(
                "Unknown masking type: {}",
                masking_byte
            )))
        }
    };

    let compression = match compression_byte {
        0 => Compression::None,
        1 => Compression::ByteRun1,
        _ => {
            return Err(AssetError::InvalidFormat(format!(
                "Unknown compression type: {}",
                compression_byte
            )))
        }
    };

    Ok(BitmapHeader {
        width,
        height,
        x,
        y,
        bit_planes,
        masking,
        compression,
        transparent_color,
        x_aspect,
        y_aspect,
        page_width,
        page_height,
    })
}

/// Decompress BODY chunk data
fn decompress_body(
    header: &BitmapHeader,
    compressed: &[u8],
    bitmap_type: BitmapType,
) -> Result<Vec<u8>> {
    let (width, depth) = match bitmap_type {
        BitmapType::Pbm => (header.width as usize, 1),
        BitmapType::Ilbm => ((header.width as usize + 7) / 8, header.bit_planes as usize),
    };

    let row_size = width * depth;
    let total_size = row_size * header.height as usize;

    match header.compression {
        Compression::None => decompress_uncompressed(compressed, header, width, depth),
        Compression::ByteRun1 => decompress_byterun1(compressed, header, width, depth, total_size),
    }
}

/// Decompress uncompressed data (just copy with mask handling)
fn decompress_uncompressed(
    data: &[u8],
    header: &BitmapHeader,
    width: usize,
    depth: usize,
) -> Result<Vec<u8>> {
    let row_size = width * depth;
    let total_size = row_size * header.height as usize;
    let mut output = Vec::with_capacity(total_size);
    let mut pos = 0;

    for _ in 0..header.height {
        let row_data = data
            .get(pos..pos + row_size)
            .ok_or_else(|| AssetError::InvalidFormat("Truncated IFF body data".to_string()))?;
        output.extend_from_slice(row_data);
        pos += row_size;

        // Skip mask data if present
        if matches!(header.masking, Masking::HasMask) {
            pos += width;
        }

        // Skip padding byte for odd width
        if header.width & 1 != 0 {
            pos += 1;
        }
    }

    Ok(output)
}

/// Decompress ByteRun1 RLE compressed data
fn decompress_byterun1(
    data: &[u8],
    header: &BitmapHeader,
    width: usize,
    depth: usize,
    total_size: usize,
) -> Result<Vec<u8>> {
    let mut output = Vec::with_capacity(total_size);
    let mut pos = 0;
    let mut wid_cnt = width;
    let mut plane = 0;
    let end_cnt = if width & 1 != 0 { -1isize } else { 0isize };

    while pos < data.len() && output.len() < total_size {
        if wid_cnt as isize == end_cnt {
            wid_cnt = width;
            plane += 1;
            if (matches!(header.masking, Masking::HasMask) && plane == depth + 1)
                || (!matches!(header.masking, Masking::HasMask) && plane == depth)
            {
                plane = 0;
            }
        }

        let n = data[pos] as i8;
        pos += 1;

        if n >= 0 {
            // Copy next n+1 bytes literally
            let count = (n as usize) + 1;
            wid_cnt = wid_cnt.saturating_sub(count);

            let mut actual_count = count;
            if wid_cnt == usize::MAX {
                // wid_cnt wrapped, was -1
                actual_count -= 1;
                wid_cnt = 0;
            }

            if plane == depth {
                // Skip mask data
                pos += actual_count;
            } else {
                let bytes = data
                    .get(pos..pos + actual_count)
                    .ok_or_else(|| AssetError::InvalidFormat("Truncated RLE data".to_string()))?;
                output.extend_from_slice(bytes);
                pos += actual_count;
            }

            if actual_count != count {
                pos += 1; // skip padding
            }
        } else if n > -128 {
            // Repeat next byte -n+1 times
            let count = ((-n) as usize) + 1;
            let byte = data
                .get(pos)
                .copied()
                .ok_or_else(|| AssetError::InvalidFormat("Truncated RLE data".to_string()))?;
            pos += 1;

            wid_cnt = wid_cnt.saturating_sub(count);
            let mut actual_count = count;
            if wid_cnt == usize::MAX {
                actual_count -= 1;
                wid_cnt = 0;
            }

            if plane != depth {
                // Not mask data
                output.resize(output.len() + actual_count, byte);
            }
        }
        // n == -128 is a no-op
    }

    if output.len() != total_size {
        return Err(AssetError::InvalidFormat(format!(
            "IFF decompression size mismatch: expected {}, got {}",
            total_size,
            output.len()
        )));
    }

    Ok(output)
}

/// Convert ILBM planar format to chunky 8-bit indexed format
fn convert_ilbm_to_chunky(planar: &[u8], width: u16, height: u16, bit_planes: u8) -> Vec<u8> {
    let width = width as usize;
    let height = height as usize;
    let planes = bit_planes as usize;
    let bytes_per_row = (width + 7) / 8;

    let mut chunky = vec![0u8; width * height];

    for y in 0..height {
        for x in 0..width {
            let mut pixel = 0u8;
            let byte_x = x / 8;
            let bit_x = 7 - (x % 8);

            for plane in 0..planes {
                let offset = (y * planes * bytes_per_row) + (plane * bytes_per_row) + byte_x;
                if let Some(&byte) = planar.get(offset) {
                    let bit = (byte >> bit_x) & 1;
                    pixel |= bit << plane;
                }
            }

            chunky[y * width + x] = pixel;
        }
    }

    chunky
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_iff_pbm() -> Vec<u8> {
        let mut data = Vec::new();

        // Calculate FORM content size:
        // "PBM " = 4 bytes
        // BMHD chunk = 8 (header) + 20 (data) = 28 bytes
        // CMAP chunk = 8 (header) + 6 (data) = 14 bytes
        // BODY chunk = 8 (header) + 16 (data) = 24 bytes
        // Total = 4 + 28 + 14 + 24 = 70 bytes

        // FORM chunk
        data.extend_from_slice(b"FORM");
        data.extend_from_slice(&70u32.to_be_bytes()); // form length

        // PBM form type
        data.extend_from_slice(b"PBM ");

        // BMHD chunk
        data.extend_from_slice(b"BMHD");
        data.extend_from_slice(&20u32.to_be_bytes()); // chunk length
        data.extend_from_slice(&4u16.to_be_bytes()); // width
        data.extend_from_slice(&4u16.to_be_bytes()); // height
        data.extend_from_slice(&0i16.to_be_bytes()); // x
        data.extend_from_slice(&0i16.to_be_bytes()); // y
        data.push(8); // bit planes
        data.push(0); // masking
        data.push(0); // compression
        data.push(0); // pad
        data.extend_from_slice(&0u16.to_be_bytes()); // transparent color
        data.push(5); // x aspect
        data.push(6); // y aspect
        data.extend_from_slice(&4u16.to_be_bytes()); // page width
        data.extend_from_slice(&4u16.to_be_bytes()); // page height

        // CMAP chunk (palette)
        data.extend_from_slice(b"CMAP");
        data.extend_from_slice(&6u32.to_be_bytes()); // 2 colors * 3 bytes
        data.extend_from_slice(&[0, 0, 0]); // color 0: black
        data.extend_from_slice(&[255, 255, 255]); // color 1: white

        // BODY chunk (uncompressed 4x4 pixels)
        data.extend_from_slice(b"BODY");
        data.extend_from_slice(&16u32.to_be_bytes());
        data.extend_from_slice(&[
            0, 1, 0, 1, // row 1
            1, 0, 1, 0, // row 2
            0, 1, 0, 1, // row 3
            1, 0, 1, 0, // row 4
        ]);

        data
    }

    #[test]
    fn test_parse_pbm() {
        let data = create_test_iff_pbm();
        let iff = IffFile::parse(&data).unwrap();

        assert_eq!(iff.width(), 4);
        assert_eq!(iff.height(), 4);
        assert_eq!(iff.bit_planes(), 8);
        assert_eq!(iff.bitmap_type(), BitmapType::Pbm);
        assert_eq!(iff.compression(), Compression::None);
        assert_eq!(iff.bitmap_data().len(), 16);
    }

    #[test]
    fn test_invalid_signature() {
        let data = b"JUNK";
        let result = IffFile::parse(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_chunk_id_display() {
        assert_eq!(ChunkId::FORM.as_str(), "FORM");
        assert_eq!(ChunkId::ILBM.as_str(), "ILBM");
        assert_eq!(ChunkId::BMHD.as_str(), "BMHD");
    }

    #[test]
    fn test_palette() {
        let data = create_test_iff_pbm();
        let iff = IffFile::parse(&data).unwrap();

        let palette = iff.palette().unwrap();
        assert_eq!(palette.len(), 6); // 2 colors * 3 bytes

        // Check first color (black)
        assert_eq!(palette[0], 0);
        assert_eq!(palette[1], 0);
        assert_eq!(palette[2], 0);

        // Check second color (white)
        assert_eq!(palette[3], 255);
        assert_eq!(palette[4], 255);
        assert_eq!(palette[5], 255);
    }
}
